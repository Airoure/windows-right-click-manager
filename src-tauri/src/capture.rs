//! Observe a single Explorer popup session. Never invoke menu actions or inject into Explorer.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    io::Read,
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use windows::{
    core::{Interface, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HWND, LPARAM},
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
            Variant::VARIANT,
        },
        UI::{
            Accessibility::{AccessibleObjectFromWindow, IAccessible},
            WindowsAndMessaging::{
                EnumWindows, GetClassNameW, GetWindowThreadProcessId, IsWindowVisible, OBJID_CLIENT,
            },
        },
    },
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub name: String,
    pub shortcut: String,
    pub disabled: bool,
    pub separator: bool,
    pub has_submenu: bool,
    pub children: Vec<Item>,
    #[serde(skip)]
    active: bool,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub items: Vec<Item>,
    pub captured_at: u64,
    pub warnings: Vec<String>,
}

pub fn run_worker() -> Result<Capture, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut child = Command::new(exe)
        .arg("--capture-menu")
        .creation_flags(0x08000000)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let output = child.stdout.take().ok_or("无法连接菜单读取进程")?;
    // Drain concurrently: a large submenu must not fill the pipe and deadlock the worker.
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        output
            .take(2 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() < Duration::from_secs(25) => {
                std::thread::sleep(Duration::from_millis(100))
            }
            other => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err(match other {
                    Err(e) => e.to_string(),
                    _ => "读取超时。请重试，并在 20 秒内右键、展开需要的子菜单，再按 Esc 关闭。"
                        .into(),
                });
            }
        }
    }
    let bytes = reader
        .join()
        .map_err(|_| "菜单读取进程异常")?
        .map_err(|e| e.to_string())?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("菜单数据过大，已停止读取。".into());
    }
    serde_json::from_slice::<Result<Capture, String>>(&bytes)
        .map_err(|_| "无法读取系统菜单数据。".to_owned())?
}

unsafe extern "system" fn collect(hwnd: HWND, param: LPARAM) -> windows::core::BOOL {
    let output = &mut *(param.0 as *mut Vec<HWND>);
    let mut class = [0u16; 64];
    let len = GetClassNameW(hwnd, &mut class);
    if IsWindowVisible(hwnd).as_bool()
        && String::from_utf16_lossy(&class[..len.max(0) as usize]) == "#32768"
    {
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut path = [0u16; 32768];
            let mut size = path.len() as u32;
            let ok = QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(path.as_mut_ptr()),
                &mut size,
            )
            .is_ok();
            let _ = CloseHandle(process);
            if ok
                && String::from_utf16_lossy(&path[..size as usize])
                    .rsplit('\\')
                    .next()
                    .is_some_and(|s| s.eq_ignore_ascii_case("explorer.exe"))
            {
                output.push(hwnd);
            }
        }
    }
    true.into()
}
fn popups() -> Vec<HWND> {
    let mut windows = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut windows as *mut _ as isize));
    }
    windows
}
fn number(value: windows::core::Result<VARIANT>) -> i32 {
    value
        .ok()
        .and_then(|v| i32::try_from(&v).ok())
        .unwrap_or_default()
}
unsafe fn read_popup(hwnd: HWND) -> Result<(Vec<Item>, String), String> {
    let mut raw = std::ptr::null_mut();
    AccessibleObjectFromWindow(hwnd, OBJID_CLIENT.0 as u32, &IAccessible::IID, &mut raw)
        .map_err(|e| e.to_string())?;
    if raw.is_null() {
        return Err("系统未提供菜单内容。".into());
    }
    let accessible = IAccessible::from_raw(raw);
    let parent = accessible
        .accParent()
        .ok()
        .and_then(|p| p.cast::<IAccessible>().ok())
        .and_then(|p| p.get_accName(&VARIANT::from(0i32)).ok())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let count = accessible.accChildCount().map_err(|e| e.to_string())?;
    if count > 512 {
        return Err("菜单项目过多。".into());
    }
    let mut items = Vec::new();
    for index in 1..=count {
        let child = VARIANT::from(index);
        let state = number(accessible.get_accState(&child)) as u32;
        if state & 0x8000 != 0 {
            continue;
        } // STATE_SYSTEM_INVISIBLE
        let role = number(accessible.get_accRole(&child));
        let raw_name = accessible
            .get_accName(&child)
            .map(|s| s.to_string())
            .unwrap_or_default();
        if raw_name.len() > 8192 {
            return Err("菜单文字超出允许长度。".into());
        }
        let (name, shortcut) = raw_name.split_once('\t').unwrap_or((&raw_name, ""));
        let separator = role == 21;
        if !separator && name.is_empty() {
            continue;
        }
        items.push(Item {
            name: name.into(),
            shortcut: shortcut.into(),
            disabled: state & 1 != 0,
            separator,
            has_submenu: state & 0x40000000 != 0,
            children: Vec::new(),
            active: state & (4 | 128) != 0,
        });
    }
    if items.is_empty() {
        return Err("系统未提供可读取的菜单项。".into());
    }
    Ok((items, parent))
}
fn preserve_children(next: &mut [Item], old: &[Item]) {
    for (i, item) in next.iter_mut().enumerate() {
        if let Some(previous) = old
            .get(i)
            .filter(|p| p.name == item.name && p.separator == item.separator)
        {
            item.children = previous.children.clone();
            clear_active(&mut item.children);
            item.has_submenu |= previous.has_submenu;
        }
    }
}
fn clear_active(items: &mut [Item]) {
    for item in items {
        item.active = false;
        clear_active(&mut item.children);
    }
}
fn attach(items: &mut [Item], path: &[usize], mut children: Vec<Item>) {
    if let Some(item) = items.get_mut(path[0]) {
        if path.len() == 1 {
            preserve_children(&mut children, &item.children);
            item.children = children;
            item.has_submenu = true;
        } else {
            attach(&mut item.children, &path[1..], children);
        }
    }
}
fn candidates(items: &[Item], parent: &str, prefix: &[usize], result: &mut Vec<Vec<usize>>) {
    for (index, item) in items.iter().enumerate() {
        let mut path = prefix.to_vec();
        path.push(index);
        if !item.separator
            && ((!parent.is_empty() && item.name == parent) || (parent.is_empty() && item.active))
        {
            result.push(path.clone());
        }
        candidates(&item.children, parent, &path, result);
    }
}
pub fn observe() -> Result<Capture, String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
    }
    struct Com;
    impl Drop for Com {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }
    let _com = Com;
    if !popups().is_empty() {
        return Err("请先关闭已经打开的右键菜单，再点击读取。".into());
    }
    let start = Instant::now();
    let mut root: Option<HWND> = None;
    let mut items = Vec::new();
    let mut closed = None;
    let mut unlinked = HashSet::new();
    loop {
        let windows = popups();
        if let Some(handle) = root {
            if !windows.contains(&handle) {
                let gone = closed.get_or_insert_with(Instant::now);
                if gone.elapsed() > Duration::from_millis(350) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(80));
                continue;
            }
        }
        closed = None;
        if root.is_none() && !windows.is_empty() {
            // The first popup is the menu the user opened; subsequent popups are submenus.
            root = windows.last().copied();
        }
        if let Some(handle) = root {
            if let Ok((mut fresh, _)) = unsafe { read_popup(handle) } {
                preserve_children(&mut fresh, &items);
                items = fresh;
            }
            for popup in windows.iter().rev().filter(|w| **w != handle) {
                if let Ok((children, parent)) = unsafe { read_popup(*popup) } {
                    let key = popup.0 as isize;
                    let path = {
                        let mut found = Vec::new();
                        candidates(&items, &parent, &[], &mut found);
                        if found.is_empty() && !parent.is_empty() {
                            candidates(&items, "", &[], &mut found);
                        }
                        (found.len() == 1).then(|| found.remove(0))
                    };
                    if let Some(path) = path {
                        attach(&mut items, &path, children);
                        unlinked.remove(&key);
                    } else {
                        unlinked.insert(key);
                    }
                }
            }
        }
        if start.elapsed() > Duration::from_secs(20) {
            return Err(if root.is_none() {
                "没有读到菜单。请重试，在桌面或资源管理器中右键一次。"
            } else {
                "菜单仍未关闭。请按 Esc 关闭后重新读取。"
            }
            .into());
        }
        std::thread::sleep(Duration::from_millis(120));
    }
    if items.is_empty() {
        return Err("此菜单未提供可读取的内容，请换一个位置重试。".into());
    }
    Ok(Capture {
        items,
        captured_at: crate::model::now(),
        warnings: if unlinked.is_empty() {
            vec![]
        } else {
            vec!["部分子菜单无法确定归属，未合并。请重新读取并逐个展开。".into()]
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(name: &str) -> Item {
        Item {
            name: name.into(),
            shortcut: String::new(),
            disabled: false,
            separator: false,
            has_submenu: false,
            children: vec![],
            active: false,
        }
    }
    #[test]
    fn previous_submenu_focus_does_not_contaminate_next_popup() {
        let mut previous = item("桌面助手");
        let mut child = item("设置");
        child.active = true;
        previous.children.push(child);
        previous.has_submenu = true;
        let mut fresh = vec![item("桌面助手")];
        fresh[0].disabled = true;
        preserve_children(&mut fresh, &[previous]);
        assert!(fresh[0].disabled);
        assert!(fresh[0].has_submenu);
        assert_eq!(fresh[0].children[0].name, "设置");
        let mut owners = Vec::new();
        candidates(&fresh, "", &[], &mut owners);
        assert!(owners.is_empty());
        // A changed command at the same index must not inherit another command's submenu.
        let mut changed = vec![item("新建")];
        preserve_children(&mut changed, &fresh);
        assert!(changed[0].children.is_empty());
    }
    #[test]
    fn duplicate_captions_are_ambiguous_even_across_submenu_levels() {
        let mut root = item("查看");
        root.children.push(item("设置"));
        let items = vec![root, item("设置")];
        let mut owners = Vec::new();
        candidates(&items, "设置", &[], &mut owners);
        assert_eq!(owners, vec![vec![0, 0], vec![1]]);
        // observe only attaches when there is exactly one owner.
        assert_ne!(owners.len(), 1);
    }
}
