use crate::{elevation, journal::Journal, model::*, registry};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub scan: ScanResult,
    pub history: Vec<HistoryItem>,
    pub storage_path: String,
}
pub struct Service {
    pub journal: Journal,
}
impl Service {
    pub fn data(&self) -> AppData {
        let mut scan = registry::scan();
        let (history, warnings) = self.journal.list();
        scan.warnings.extend(warnings);
        AppData {
            scan,
            history: history.into_iter().map(Into::into).collect(),
            storage_path: self.journal.directory.to_string_lossy().into_owned(),
        }
    }
    pub fn toggle(&self, id: &str, enabled: bool) -> Result<(), String> {
        let target = Target::parse(id)?;
        let scanned = registry::scan();
        let item = scanned
            .entries
            .iter()
            .find(|e| e.id == id)
            .ok_or("菜单已不存在，请重新扫描。")?;
        if !item.manageable {
            return Err(item.reason.clone());
        }
        let before = registry::current(&target)?.ok_or("菜单已不存在。")?;
        registry::eligibility(&target, &before)?;
        if registry::legacy_value(&before).is_none() == enabled {
            return Err("菜单状态已经变化，请重新扫描。".into());
        }
        let desired = if enabled {
            None
        } else {
            Some(ValueData::string(""))
        };
        let after = registry::set_legacy_snapshot(&before, desired.clone());
        let mut record = Record {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now(),
            label: item.name.clone(),
            target: target.clone(),
            operation: if enabled { "enable" } else { "disable" }.into(),
            before: Some(before.clone()),
            after: Some(after),
            status: "prepared".into(),
            error: None,
            restored_at: None,
        };
        self.journal.save(&record)?;
        let result = elevation::execute(&LegacyPlan {
            target,
            expected: before,
            value: desired,
        })
        .map(|_| ());
        self.finish(&mut record, result)
    }
    fn finish(&self, record: &mut Record, result: Result<(), String>) -> Result<(), String> {
        record.status = if result.is_ok() { "applied" } else { "failed" }.into();
        record.error = result.as_ref().err().cloned();
        self.journal.save(record)?;
        result
    }
    pub fn create(&self, menu: NewMenu) -> Result<(), String> {
        let after = registry::custom_snapshot(&menu)?;
        let target = Target {
            hive: "CU".into(),
            location: menu.scope,
            verb: format!("RightClickManager_{}", uuid::Uuid::new_v4()),
        };
        let mut record = Record {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now(),
            label: menu.name.trim().into(),
            target: target.clone(),
            operation: "create".into(),
            before: None,
            after: Some(after.clone()),
            status: "prepared".into(),
            error: None,
            restored_at: None,
        };
        self.journal.save(&record)?;
        self.finish(&mut record, registry::create_custom(&target, &after))
    }
    pub fn restore(&self, id: &str) -> Result<(), String> {
        let mut record = self.journal.load(id)?;
        if record.status == "restored" {
            return Err("这次操作已经恢复。".into());
        }
        let (records, warnings) = self.journal.list();
        if !warnings.is_empty() {
            return Err("部分操作历史无法读取，请先检查恢复记录文件。".into());
        }
        if records.iter().any(|later| {
            later.target == record.target
                && later.timestamp > record.timestamp
                && later.status != "restored"
        }) {
            return Err("这个菜单还有后续操作，请先恢复最新的一次修改。".into());
        }
        let current = registry::current(&record.target)?;
        if current == record.before {
            record.status = "restored".into();
            record.restored_at = Some(now());
            record.error = None;
            return self.journal.save(&record);
        }
        // A crash during creation may leave a subset of the intended new key.
        let partial_create = record.operation == "create"
            && ["prepared", "failed", "restore_pending"].contains(&record.status.as_str())
            && current
                .as_ref()
                .zip(record.after.as_ref())
                .is_some_and(|(actual, expected)| is_subset(actual, expected));
        if current != record.after && !partial_create {
            return Err(
                "菜单已被其他软件或后续操作修改，无法直接覆盖。请先恢复后续操作，或检查注册位置。"
                    .into(),
            );
        }
        record.status = "restore_pending".into();
        record.error = None;
        self.journal.save(&record)?;
        let result = if record.operation == "create" {
            registry::remove_custom(&record.target, current.as_ref().ok_or("菜单已经不存在。")?)
        } else {
            let before = record.before.as_ref().ok_or("恢复记录缺少原始状态。")?;
            let after = record.after.as_ref().ok_or("恢复记录缺少修改后状态。")?;
            if registry::set_legacy_snapshot(after, registry::legacy_value(before)) != *before {
                return Err("恢复记录的修改范围不符合要求。".into());
            }
            elevation::execute(&LegacyPlan {
                target: record.target.clone(),
                expected: after.clone(),
                value: registry::legacy_value(before),
            })
            .map(|_| ())
        };
        if let Err(error) = result {
            record.error = Some(error.clone());
            self.journal.save(&record)?;
            return Err(error);
        }
        record.status = "restored".into();
        record.restored_at = Some(now());
        self.journal.save(&record)
    }
}
fn is_subset(actual: &KeySnapshot, expected: &KeySnapshot) -> bool {
    actual
        .values
        .iter()
        .all(|(k, v)| expected.values.get(k) == Some(v))
        && actual.children.iter().all(|(k, child)| {
            expected
                .children
                .get(k)
                .is_some_and(|e| is_subset(child, e))
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_creation_cannot_hide_external_changes() {
        let mut expected = KeySnapshot::default();
        expected
            .values
            .insert("RightClickManager".into(), ValueData::string("1"));
        assert!(is_subset(&KeySnapshot::default(), &expected));
        let mut actual = expected.clone();
        actual
            .values
            .insert("external".into(), ValueData::string("changed"));
        assert!(!is_subset(&actual, &expected));
    }
    #[test]
    fn custom_create_disable_restore_and_undo_roundtrip() {
        let directory =
            std::env::temp_dir().join(format!("rcm-service-test-{}", uuid::Uuid::new_v4()));
        let service = Service {
            journal: Journal::new(&directory).unwrap(),
        };
        let program =
            std::path::Path::new(&std::env::var("WINDIR").unwrap()).join("System32\\notepad.exe");
        service
            .create(NewMenu {
                name: "右键管理自动验证（临时）".into(),
                scope: "background".into(),
                program: program.to_string_lossy().into_owned(),
                arguments: "\"{path}\"".into(),
            })
            .unwrap();
        let created = service
            .journal
            .list()
            .0
            .into_iter()
            .find(|r| r.operation == "create")
            .unwrap();
        let initial = registry::current(&created.target).unwrap().unwrap();
        assert!(registry::command(&initial).ends_with("\"%V\""));
        assert!(registry::scan()
            .entries
            .iter()
            .any(|e| e.id == created.target.id() && e.manageable));
        service.toggle(&created.target.id(), false).unwrap();
        let disabled = service
            .journal
            .list()
            .0
            .into_iter()
            .find(|r| r.operation == "disable")
            .unwrap();
        assert!(
            registry::legacy_value(&registry::current(&created.target).unwrap().unwrap()).is_some()
        );
        assert!(
            service.restore(&created.id).is_err(),
            "must not delete a subsequently changed menu"
        );
        service.toggle(&created.target.id(), true).unwrap();
        let enabled = service
            .journal
            .list()
            .0
            .into_iter()
            .find(|r| r.operation == "enable")
            .unwrap();
        assert!(
            service.restore(&created.id).is_err(),
            "same registry state still must respect later operations"
        );
        service.restore(&enabled.id).unwrap();
        service.restore(&disabled.id).unwrap();
        assert_eq!(registry::current(&created.target).unwrap(), Some(initial));
        service.restore(&created.id).unwrap();
        assert!(registry::current(&created.target).unwrap().is_none());
        for record in service.journal.list().0 {
            std::fs::remove_file(
                service
                    .journal
                    .directory
                    .join(format!("{}.json", record.id)),
            )
            .unwrap();
        }
        std::fs::remove_dir(&service.journal.directory).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
