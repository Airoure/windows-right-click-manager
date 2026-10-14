use crate::model::Record;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub struct Journal {
    pub directory: PathBuf,
}
impl Journal {
    pub fn new(app_data: &Path) -> Result<Self, String> {
        let directory = app_data.join("history");
        fs::create_dir_all(&directory).map_err(|e| format!("无法创建恢复记录目录：{e}"))?;
        Ok(Self { directory })
    }
    fn path(&self, id: &str) -> Result<PathBuf, String> {
        let id = uuid::Uuid::parse_str(id).map_err(|_| "无效的恢复记录标识。")?;
        Ok(self.directory.join(format!("{id}.json")))
    }
    pub fn save(&self, record: &Record) -> Result<(), String> {
        let final_path = self.path(&record.id)?;
        let tmp = self.directory.join(format!("{}.tmp", uuid::Uuid::new_v4()));
        let result = (|| -> Result<(), String> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp)
                .map_err(|e| e.to_string())?;
            file.write_all(&serde_json::to_vec_pretty(record).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            drop(file);
            fs::rename(&tmp, &final_path).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        result.map_err(|e| format!("恢复记录保存失败，操作已停止或需要重新检查历史：{e}"))
    }
    pub fn load(&self, id: &str) -> Result<Record, String> {
        let path = self.path(id)?;
        let bytes = fs::read(&path).map_err(|e| format!("无法读取恢复记录：{e}"))?;
        if bytes.len() > 16_000_000 {
            return Err("恢复记录超出大小限制。".into());
        }
        let record: Record = serde_json::from_slice(&bytes)
            .map_err(|e| format!("恢复记录损坏，未执行任何修改：{e}"))?;
        if record.id != id {
            return Err("恢复记录标识不匹配。".into());
        }
        record.target.validate()?;
        Ok(record)
    }
    pub fn list(&self) -> (Vec<Record>, Vec<String>) {
        let mut records = Vec::new();
        let mut warnings = Vec::new();
        let files = match fs::read_dir(&self.directory) {
            Ok(files) => files,
            Err(e) => return (records, vec![format!("无法读取历史目录：{e}")]),
        };
        for file in files {
            let file = match file {
                Ok(file) => file,
                Err(e) => {
                    warnings.push(format!("历史文件读取失败：{e}"));
                    continue;
                }
            };
            let path = file.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            match self.load(id) {
                Ok(record) => records.push(record),
                Err(e) => warnings.push(format!("记录 {id}：{e}")),
            }
        }
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then(b.id.cmp(&a.id)));
        (records, warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    #[test]
    fn durable_record_can_be_updated_without_losing_original_state() {
        let directory =
            std::env::temp_dir().join(format!("rcm-journal-test-{}", uuid::Uuid::new_v4()));
        let journal = Journal::new(&directory).unwrap();
        let mut record = Record {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now(),
            label: "测试".into(),
            target: Target {
                hive: "CU".into(),
                location: "background".into(),
                verb: "test".into(),
            },
            operation: "disable".into(),
            before: Some(KeySnapshot::default()),
            after: Some(KeySnapshot::default()),
            status: "prepared".into(),
            error: None,
            restored_at: None,
        };
        journal.save(&record).unwrap();
        record.status = "applied".into();
        journal.save(&record).unwrap();
        let loaded = journal.load(&record.id).unwrap();
        assert_eq!(loaded.status, "applied");
        assert_eq!(loaded.before, record.before);
        assert!(journal.load("../anything").is_err());
        std::fs::remove_file(journal.path(&record.id).unwrap()).unwrap();
        std::fs::remove_dir(&journal.directory).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
