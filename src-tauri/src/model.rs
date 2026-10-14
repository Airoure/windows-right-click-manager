use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub hive: String,
    pub location: String,
    pub verb: String,
}

impl Target {
    pub fn id(&self) -> String {
        format!("{}|{}|shell|{}", self.hive, self.location, self.verb)
    }
    pub fn parse(id: &str) -> Result<Self, String> {
        let parts: Vec<_> = id.split('|').collect();
        if parts.len() != 4 || parts[2] != "shell" {
            return Err("这个菜单类型暂不支持修改。".into());
        }
        let target = Self {
            hive: parts[0].into(),
            location: parts[1].into(),
            verb: parts[3].into(),
        };
        target.validate()?;
        Ok(target)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !["CU", "LM"].contains(&self.hive.as_str())
            || location_root(&self.location).is_none()
            || self.verb.is_empty()
            || self.verb.len() > 240
            || self
                .verb
                .chars()
                .any(|c| ['\\', '/', '\0', '|'].contains(&c))
        {
            return Err("菜单目标不在允许的管理范围内。".into());
        }
        Ok(())
    }
    pub fn relative_path(&self) -> Result<String, String> {
        self.validate()?;
        Ok(format!(
            "Software\\Classes\\{}\\shell\\{}",
            location_root(&self.location).unwrap(),
            self.verb
        ))
    }
    pub fn display_path(&self) -> String {
        format!(
            "{}\\{}",
            if self.hive == "CU" {
                "HKEY_CURRENT_USER"
            } else {
                "HKEY_LOCAL_MACHINE"
            },
            self.relative_path().unwrap_or_default()
        )
    }
}

pub const LOCATIONS: [&str; 6] = [
    "desktop",
    "file",
    "filesystem",
    "folder",
    "background",
    "drive",
];
pub fn location_root(location: &str) -> Option<&'static str> {
    match location {
        "desktop" => Some("DesktopBackground"),
        "file" => Some("*"),
        "filesystem" => Some("AllFilesystemObjects"),
        "folder" => Some("Directory"),
        "background" => Some("Directory\\Background"),
        "drive" => Some("Drive"),
        _ => None,
    }
}
pub fn scope(location: &str) -> &str {
    if location == "filesystem" {
        "filesystem"
    } else {
        location
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValueData {
    pub kind: u32,
    pub bytes: Vec<u8>,
}
impl ValueData {
    pub fn string(value: &str) -> Self {
        Self {
            kind: 1,
            bytes: value
                .encode_utf16()
                .chain(std::iter::once(0))
                .flat_map(u16::to_le_bytes)
                .collect(),
        }
    }
    pub fn text(&self) -> Option<String> {
        if ![1, 2].contains(&self.kind) || self.bytes.len() % 2 != 0 {
            return None;
        }
        let chars: Vec<_> = self
            .bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        Some(
            String::from_utf16_lossy(&chars)
                .trim_end_matches('\0')
                .to_owned(),
        )
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeySnapshot {
    pub values: BTreeMap<String, ValueData>,
    pub children: BTreeMap<String, KeySnapshot>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuEntry {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub source: String,
    pub kind: String,
    pub enabled: bool,
    pub manageable: bool,
    pub reason: String,
    pub command: String,
    pub registry_path: String,
    pub owner: String,
    pub requires_admin: bool,
    pub extended: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub entries: Vec<MenuEntry>,
    pub warnings: Vec<String>,
    pub scanned_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: String,
    pub timestamp: u64,
    pub label: String,
    pub target: Target,
    pub operation: String,
    pub before: Option<KeySnapshot>,
    pub after: Option<KeySnapshot>,
    pub status: String,
    pub error: Option<String>,
    pub restored_at: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: String,
    pub timestamp: u64,
    pub label: String,
    pub scope: String,
    pub operation: String,
    pub status: String,
    pub error: Option<String>,
    pub can_restore: bool,
    pub requires_admin: bool,
}
impl From<Record> for HistoryItem {
    fn from(r: Record) -> Self {
        Self {
            id: r.id,
            timestamp: r.timestamp,
            label: r.label,
            scope: r.target.location,
            operation: r.operation,
            can_restore: matches!(
                r.status.as_str(),
                "applied" | "prepared" | "failed" | "restore_pending"
            ),
            status: r.status,
            error: r.error,
            requires_admin: r.target.hive == "LM",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewMenu {
    pub name: String,
    pub scope: String,
    pub program: String,
    pub arguments: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyPlan {
    pub target: Target,
    pub expected: KeySnapshot,
    pub value: Option<ValueData>,
}

pub fn now() -> u64 {
    static LAST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let wall = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let previous = LAST
        .fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |last| Some(wall.max(last.saturating_add(1))),
        )
        .unwrap();
    wall.max(previous.saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_targets_outside_menu_roots() {
        assert!(Target::parse("CU|background|shell|example").is_ok());
        for id in [
            "CU|background|shell|x\\command",
            "CU|CLSID|shell|x",
            "LM|file|shellex|x",
            "XX|file|shell|x",
            "CU|folder|shell|",
        ] {
            assert!(Target::parse(id).is_err(), "{id}");
        }
    }
    #[test]
    fn registry_strings_round_trip_unicode() {
        let value = ValueData::string("中文 C:\\Program Files\\编辑器.exe");
        assert_eq!(
            value.text().as_deref(),
            Some("中文 C:\\Program Files\\编辑器.exe")
        );
        assert_eq!(ValueData::string("").bytes, vec![0, 0]);
    }
}
