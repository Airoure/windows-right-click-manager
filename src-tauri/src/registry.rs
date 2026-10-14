use crate::model::*;
use std::collections::BTreeMap;
use winreg::{enums::*, RegKey, RegValue};

fn err(e: impl std::fmt::Display) -> String {
    format!("注册表操作失败：{e}")
}
fn flush(key: &RegKey) -> Result<(), String> {
    let code = unsafe { windows_sys::Win32::System::Registry::RegFlushKey(key.raw_handle()) };
    if code == 0 {
        Ok(())
    } else {
        Err(err(std::io::Error::from_raw_os_error(code as i32)))
    }
}
pub fn root(hive: &str) -> Result<RegKey, String> {
    match hive {
        "CU" => Ok(RegKey::predef(HKEY_CURRENT_USER)),
        "LM" => Ok(RegKey::predef(HKEY_LOCAL_MACHINE)),
        _ => Err("不支持的注册表来源。".into()),
    }
}
pub fn open(target: &Target, writable: bool) -> Result<RegKey, String> {
    root(&target.hive)?
        .open_subkey_with_flags(
            target.relative_path()?,
            KEY_READ | KEY_WOW64_64KEY | if writable { KEY_SET_VALUE } else { 0 },
        )
        .map_err(err)
}

fn snapshot_depth(key: &RegKey, depth: usize, total: &mut usize) -> Result<KeySnapshot, String> {
    if depth > 12 {
        return Err("菜单结构过深，已停止修改。".into());
    }
    let mut result = KeySnapshot::default();
    for pair in key.enum_values() {
        let (name, value) = pair.map_err(err)?;
        *total += value.bytes.len() + name.len();
        if *total > 1_000_000 {
            return Err("菜单数据过大，已停止修改。".into());
        }
        result.values.insert(
            name,
            ValueData {
                kind: value.vtype as u32,
                bytes: value.bytes,
            },
        );
    }
    for name in key.enum_keys() {
        let name = name.map_err(err)?;
        *total += name.len() + 1;
        if *total > 1_000_000 {
            return Err("菜单数据过大，已停止修改。".into());
        }
        let child = key
            .open_subkey_with_flags(&name, KEY_READ | KEY_WOW64_64KEY)
            .map_err(err)?;
        result
            .children
            .insert(name, snapshot_depth(&child, depth + 1, total)?);
    }
    Ok(result)
}
pub fn snapshot(key: &RegKey) -> Result<KeySnapshot, String> {
    snapshot_depth(key, 0, &mut 0)
}
pub fn current(target: &Target) -> Result<Option<KeySnapshot>, String> {
    let path = target.relative_path()?;
    match root(&target.hive)?.open_subkey_with_flags(path, KEY_READ | KEY_WOW64_64KEY) {
        Ok(key) => snapshot(&key).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(err(e)),
    }
}
fn value<'a>(s: &'a KeySnapshot, name: &str) -> Option<&'a ValueData> {
    s.values
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value)
}
fn text(s: &KeySnapshot, name: &str) -> String {
    value(s, name).and_then(ValueData::text).unwrap_or_default()
}
fn has(s: &KeySnapshot, name: &str) -> bool {
    value(s, name).is_some()
}
pub fn legacy_value(s: &KeySnapshot) -> Option<ValueData> {
    value(s, "LegacyDisable").cloned()
}
pub fn set_legacy_snapshot(s: &KeySnapshot, v: Option<ValueData>) -> KeySnapshot {
    let mut next = s.clone();
    let name = next
        .values
        .keys()
        .find(|k| k.eq_ignore_ascii_case("LegacyDisable"))
        .cloned()
        .unwrap_or_else(|| "LegacyDisable".into());
    if let Some(v) = v {
        next.values.insert(name, v);
    } else {
        next.values.remove(&name);
    }
    next
}
pub fn command(s: &KeySnapshot) -> String {
    s.children
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("command"))
        .map(|(_, key)| text(key, ""))
        .unwrap_or_default()
}
pub fn eligibility(target: &Target, s: &KeySnapshot) -> Result<(), String> {
    target.validate()?;
    let name = target.verb.to_ascii_lowercase();
    if name.starts_with("windows.")
        || [
            "open",
            "explore",
            "runas",
            "runasuser",
            "print",
            "printto",
            "find",
            "properties",
            "pintohome",
            "pintostartscreen",
            "cmd",
            "powershell",
            "new",
            "opennewwindow",
            "opennewprocess",
        ]
        .contains(&name.as_str())
    {
        return Err("系统基础菜单，第一版仅提供查看。".into());
    }
    if has(s, "ExplorerCommandHandler")
        || has(s, "DelegateExecute")
        || has(s, "SubCommands")
        || has(s, "ExtendedSubCommandsKey")
        || has(s, "CommandStateHandler")
        || has(s, "ProgrammaticAccessOnly")
        || has(s, "AppliesTo")
    {
        return Err("这个菜单使用动态命令、条件或子菜单，第一版仅提供查看。".into());
    }
    let cmd = s
        .children
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("command"))
        .map(|(_, key)| key);
    if cmd.is_none()
        || command(s).trim().is_empty()
        || cmd.is_some_and(|key| has(key, "DelegateExecute") || has(key, "ExplorerCommandHandler"))
    {
        return Err("没有可识别的普通执行命令，第一版仅提供查看。".into());
    }
    Ok(())
}
fn raw(value: &ValueData) -> Result<RegValue, String> {
    let vtype = match value.kind {
        0 => REG_NONE,
        1 => REG_SZ,
        2 => REG_EXPAND_SZ,
        3 => REG_BINARY,
        4 => REG_DWORD,
        5 => REG_DWORD_BIG_ENDIAN,
        6 => REG_LINK,
        7 => REG_MULTI_SZ,
        8 => REG_RESOURCE_LIST,
        9 => REG_FULL_RESOURCE_DESCRIPTOR,
        10 => REG_RESOURCE_REQUIREMENTS_LIST,
        11 => REG_QWORD,
        _ => return Err("不支持的注册表值类型。".into()),
    };
    if value.bytes.len() > 65536 {
        return Err("恢复值超出允许大小。".into());
    }
    Ok(RegValue {
        vtype,
        bytes: value.bytes.clone(),
    })
}
pub fn apply_legacy(plan: &LegacyPlan) -> Result<KeySnapshot, String> {
    let key = open(&plan.target, true)?;
    let after = apply_legacy_key(&key, plan)?;
    notify_shell();
    Ok(after)
}
fn apply_legacy_key(key: &RegKey, plan: &LegacyPlan) -> Result<KeySnapshot, String> {
    let before = snapshot(&key)?;
    if before != plan.expected {
        return Err("菜单已被其他软件修改，请重新扫描后再操作。".into());
    }
    eligibility(&plan.target, &before)?;
    let name = before
        .values
        .keys()
        .find(|name| name.eq_ignore_ascii_case("LegacyDisable"))
        .cloned()
        .unwrap_or_else(|| "LegacyDisable".into());
    if let Some(value) = &plan.value {
        key.set_raw_value(&name, &raw(value)?).map_err(err)?;
    } else if has(&before, "LegacyDisable") {
        key.delete_value(&name).map_err(err)?;
    }
    flush(key)?;
    let after = snapshot(&key)?;
    if after != set_legacy_snapshot(&before, plan.value.clone()) {
        return Err("写入后检测到额外变化。恢复记录已保留，请查看操作历史。".into());
    }
    Ok(after)
}
fn source(cmd: &str, fallback: &str) -> String {
    let low = cmd.to_lowercase();
    for (needle, name) in [
        ("git", "Git"),
        ("deveco", "DevEco Studio"),
        ("warp", "Warp"),
        ("workbuddy", "WorkBuddy"),
        ("code.exe", "Visual Studio Code"),
        ("notepad++", "Notepad++"),
        ("7-zip", "7-Zip"),
    ] {
        if low.contains(needle) {
            return name.into();
        }
    }
    if cmd.is_empty() {
        return fallback.into();
    }
    let exe = if let Some(rest) = cmd.strip_prefix('"') {
        rest.split('"').next().unwrap_or(rest)
    } else {
        cmd.split_whitespace().next().unwrap_or(cmd)
    };
    std::path::Path::new(exe)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback.into())
}
fn display_label(label: &str) -> Option<String> {
    if label.is_empty() {
        return None;
    }
    if !label.starts_with('@') {
        return Some(label.to_owned());
    }
    let input: Vec<u16> = label.encode_utf16().chain(Some(0)).collect();
    let mut output = vec![0u16; 1024];
    let result = unsafe {
        windows_sys::Win32::UI::Shell::SHLoadIndirectString(
            input.as_ptr(),
            output.as_mut_ptr(),
            output.len() as u32,
            std::ptr::null(),
        )
    };
    if result < 0 {
        return None;
    }
    let end = output.iter().position(|c| *c == 0).unwrap_or(output.len());
    if end == 0 {
        None
    } else {
        Some(String::from_utf16_lossy(&output[..end]))
    }
}
fn extension_source(clsid: &str) -> String {
    if uuid::Uuid::parse_str(clsid.trim_matches(['{', '}'])).is_err() {
        return "来源未确认".into();
    }
    let classes = RegKey::predef(HKEY_CLASSES_ROOT);
    let path = format!("CLSID\\{clsid}\\InprocServer32");
    classes
        .open_subkey_with_flags(path, KEY_READ | KEY_WOW64_64KEY)
        .ok()
        .and_then(|key| key.get_value::<String, _>("").ok())
        .map(|dll| source(&dll, "来源未确认"))
        .unwrap_or_else(|| "来源未确认".into())
}
pub fn scan() -> ScanResult {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    for hive in ["CU", "LM"] {
        for location in LOCATIONS {
            let base = format!("Software\\Classes\\{}", location_root(location).unwrap());
            for kind in ["shell", "shellex\\ContextMenuHandlers"] {
                let path = format!("{base}\\{kind}");
                let parent = match root(hive)
                    .unwrap()
                    .open_subkey_with_flags(&path, KEY_READ | KEY_WOW64_64KEY)
                {
                    Ok(key) => key,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => {
                        warnings.push(format!("无法读取 {hive}\\{path}：{e}"));
                        continue;
                    }
                };
                for name in parent.enum_keys() {
                    let name = match name {
                        Ok(name) => name,
                        Err(e) => {
                            warnings.push(format!("枚举菜单失败：{e}"));
                            continue;
                        }
                    };
                    let result = (|| -> Result<MenuEntry, String> {
                        let key = parent
                            .open_subkey_with_flags(&name, KEY_READ | KEY_WOW64_64KEY)
                            .map_err(err)?;
                        let s = snapshot(&key)?;
                        let is_static = kind == "shell";
                        let target = Target {
                            hive: hive.into(),
                            location: location.into(),
                            verb: name.clone(),
                        };
                        let cmd = if is_static { command(&s) } else { text(&s, "") };
                        let label = [text(&s, "MUIVerb"), text(&s, "")]
                            .into_iter()
                            .find_map(|label| display_label(&label))
                            .unwrap_or_else(|| name.clone());
                        let allowed = if is_static {
                            eligibility(&target, &s)
                        } else {
                            Err("程序扩展可能生成多个菜单项，第一版仅提供查看。".into())
                        };
                        let system = allowed
                            .as_ref()
                            .err()
                            .is_some_and(|reason| reason.starts_with("系统基础"));
                        Ok(MenuEntry {
                            id: if is_static {
                                target.id()
                            } else {
                                format!("{hive}|{location}|extension|{name}")
                            },
                            name: label,
                            scope: scope(location).into(),
                            source: if text(&s, "RightClickManager") == "1" {
                                "自定义命令".into()
                            } else if !is_static {
                                extension_source(&cmd)
                            } else {
                                source(&cmd, if system { "Windows" } else { "来源未确认" })
                            },
                            kind: if system {
                                "system"
                            } else if is_static {
                                "static"
                            } else {
                                "extension"
                            }
                            .into(),
                            enabled: !has(&s, "LegacyDisable")
                                && !has(&s, "ProgrammaticAccessOnly"),
                            manageable: allowed.is_ok(),
                            reason: allowed.err().unwrap_or_default(),
                            command: cmd,
                            registry_path: if is_static {
                                target.display_path()
                            } else {
                                format!(
                                    "{}\\{path}\\{name}",
                                    if hive == "CU" {
                                        "HKEY_CURRENT_USER"
                                    } else {
                                        "HKEY_LOCAL_MACHINE"
                                    }
                                )
                            },
                            owner: if hive == "CU" {
                                "当前用户"
                            } else {
                                "所有用户"
                            }
                            .into(),
                            requires_admin: hive == "LM",
                            extended: has(&s, "Extended"),
                        })
                    })();
                    match result {
                        Ok(entry) => entries.push(entry),
                        Err(e) => warnings.push(format!("{name}：{e}")),
                    }
                }
            }
        }
    }
    let user_paths: std::collections::HashSet<_> = entries
        .iter()
        .filter(|e| e.id.starts_with("CU|"))
        .map(|e| e.id[3..].to_lowercase())
        .collect();
    for entry in &mut entries {
        if entry.id.starts_with("LM|") && user_paths.contains(&entry.id[3..].to_lowercase()) {
            entry.manageable = false;
            entry.reason =
                "存在同位置的用户级注册，系统级记录可能被覆盖，第一版仅提供查看。".into();
        }
    }
    entries.sort_by(|a, b| {
        a.scope
            .cmp(&b.scope)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    ScanResult {
        entries,
        warnings,
        scanned_at: now(),
    }
}
pub fn custom_snapshot(menu: &NewMenu) -> Result<KeySnapshot, String> {
    if !LOCATIONS.contains(&menu.scope.as_str()) || menu.scope == "filesystem" {
        return Err("请选择支持的菜单位置。".into());
    }
    if menu.name.trim().is_empty()
        || menu.name.len() > 240
        || menu.name.chars().any(char::is_control)
    {
        return Err("菜单名称不能为空，且不能包含换行等控制字符。".into());
    }
    let path = std::path::Path::new(menu.program.trim());
    if !path.is_absolute()
        || !path.is_file()
        || menu.program.contains('"')
        || menu.program.chars().any(char::is_control)
    {
        return Err("请选择实际存在的程序或 PowerShell 脚本的绝对路径。".into());
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_lowercase();
    if !["exe", "ps1"].contains(&ext.as_str()) {
        return Err("第一版支持 .exe 程序和 .ps1 脚本。".into());
    }
    if menu.arguments.len() > 4096 || menu.arguments.chars().any(char::is_control) {
        return Err("参数过长或包含控制字符。".into());
    }
    let token = if menu.scope == "background" {
        "%V"
    } else {
        "%1"
    };
    let args = if menu.arguments.trim().is_empty() {
        format!("\"{token}\"")
    } else {
        menu.arguments.replace("{path}", token)
    };
    let cmd = if ext == "exe" {
        format!("\"{}\" {args}", path.display())
    } else {
        format!(
            "powershell.exe -NoProfile -File \"{}\" {args}",
            path.display()
        )
    };
    let mut result = KeySnapshot::default();
    result
        .values
        .insert("".into(), ValueData::string(menu.name.trim()));
    result
        .values
        .insert("RightClickManager".into(), ValueData::string("1"));
    if ext == "exe" {
        result.values.insert(
            "Icon".into(),
            ValueData::string(&format!("\"{}\"", path.display())),
        );
    }
    result.children.insert(
        "command".into(),
        KeySnapshot {
            values: BTreeMap::from([("".into(), ValueData::string(&cmd))]),
            children: BTreeMap::new(),
        },
    );
    Ok(result)
}
fn write_snapshot(key: &RegKey, s: &KeySnapshot) -> Result<(), String> {
    for (name, value) in &s.values {
        key.set_raw_value(name, &raw(value)?).map_err(err)?;
    }
    for (name, child) in &s.children {
        let (subkey, _) = key.create_subkey(name).map_err(err)?;
        write_snapshot(&subkey, child)?;
    }
    flush(key)
}
pub fn create_custom(target: &Target, s: &KeySnapshot) -> Result<(), String> {
    if target.hive != "CU" || !target.verb.starts_with("RightClickManager_") {
        return Err("只能添加当前用户的应用自有菜单。".into());
    }
    if current(target)?.is_some() {
        return Err("菜单已存在，未覆盖原始数据。".into());
    }
    let (key, disposition) = root("CU")?
        .create_subkey_with_flags(
            target.relative_path()?,
            KEY_READ | KEY_WRITE | KEY_WOW64_64KEY,
        )
        .map_err(err)?;
    if disposition != REG_CREATED_NEW_KEY {
        return Err("菜单创建冲突，未覆盖原始数据。".into());
    }
    write_snapshot(&key, s)?;
    if current(target)?.as_ref() != Some(s) {
        return Err("创建后校验失败，已保留恢复记录。".into());
    }
    notify_shell();
    Ok(())
}
pub fn remove_custom(target: &Target, expected: &KeySnapshot) -> Result<(), String> {
    if target.hive != "CU" || !target.verb.starts_with("RightClickManager_") {
        return Err("只能撤销本应用添加的菜单。".into());
    }
    if current(target)?.as_ref() != Some(expected) {
        return Err("菜单已被修改，不能直接撤销添加。".into());
    }
    let path = target.relative_path()?;
    let (parent_path, verb) = path.rsplit_once('\\').ok_or("无效的自定义菜单位置。")?;
    let parent = root("CU")?
        .open_subkey_with_flags(parent_path, KEY_READ | KEY_WRITE | KEY_WOW64_64KEY)
        .map_err(err)?;
    parent.delete_subkey_all(verb).map_err(err)?;
    flush(&parent)?;
    if current(target)?.is_some() {
        return Err("撤销后仍检测到菜单，请重新检查。".into());
    }
    notify_shell();
    Ok(())
}
pub fn notify_shell() {
    unsafe {
        windows_sys::Win32::UI::Shell::SHChangeNotify(
            0x08000000,
            0,
            std::ptr::null(),
            std::ptr::null(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_change_preserves_other_values_and_detects_conflict() {
        let mut s = KeySnapshot::default();
        s.values
            .insert("Icon".into(), ValueData::string("test.exe"));
        s.values.insert(
            "legacydisable".into(),
            ValueData {
                kind: 3,
                bytes: vec![1, 2, 3],
            },
        );
        let disabled = set_legacy_snapshot(&s, Some(ValueData::string("")));
        assert_eq!(disabled.values.len(), 2);
        assert_eq!(disabled.values["Icon"], s.values["Icon"]);
        assert_eq!(set_legacy_snapshot(&disabled, legacy_value(&s)), s);
    }
    #[test]
    fn isolated_registry_roundtrip_and_conflict() {
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let path = format!("Software\\RightClickManagerTests\\{}", uuid::Uuid::new_v4());
        let (key, _) = root.create_subkey(&path).unwrap();
        key.set_value("", &"中文示例").unwrap();
        let (cmd, _) = key.create_subkey("command").unwrap();
        cmd.set_value("", &"\"C:\\Program Files\\Example.exe\" \"%V\"")
            .unwrap();
        let before = snapshot(&key).unwrap();
        let target = Target {
            hive: "CU".into(),
            location: "background".into(),
            verb: "TestCommand".into(),
        };
        let plan = LegacyPlan {
            target: target.clone(),
            expected: before.clone(),
            value: Some(ValueData::string("")),
        };
        let disabled = apply_legacy_key(&key, &plan).unwrap();
        assert_eq!(
            disabled,
            set_legacy_snapshot(&before, Some(ValueData::string("")))
        );
        apply_legacy_key(
            &key,
            &LegacyPlan {
                target,
                expected: disabled,
                value: None,
            },
        )
        .unwrap();
        assert_eq!(snapshot(&key).unwrap(), before);
        key.set_value("Icon", &"external.exe").unwrap();
        assert_ne!(snapshot(&key).unwrap(), before);
        assert!(apply_legacy_key(&key, &plan).is_err());
        assert!(key.get_raw_value("LegacyDisable").is_err());
        drop(cmd);
        drop(key);
        root.delete_subkey_all(&path).unwrap();
    }
}
