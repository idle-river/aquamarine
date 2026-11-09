use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufWriter,
    sync::{Mutex, RwLock},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LogLine<'a, 'b> {
    Set {
        key: &'a str,
        value: &'b str,
        constant: bool,
    },
    Delete {
        key: &'a str,
    },
}

pub struct KVStore {
    data: RwLock<HashMap<String, String>>,
    constants: RwLock<HashSet<String>>,
    wal_file: Mutex<BufWriter<File>>,
}

pub type Result<'a, T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'a>>;

impl KVStore {
    pub fn new(wal: File) -> Self {
        // Add WAL replay
        let file = BufWriter::new(wal);

        Self {
            data: RwLock::new(HashMap::new()),
            constants: RwLock::new(HashSet::new()),
            wal_file: Mutex::new(file),
        }
    }

    fn append_to_wal(&self, log: &LogLine) -> Result<'_, ()> {
        let file = &mut *self.wal_file.lock().map_err(|e| e.to_string())?;
        postcard::to_io(log, file)?;

        Ok(())
    }

    pub fn set(&mut self, key: String, value: String) -> Result<'_, ()> {
        let _ = self.append_to_wal(&LogLine::Set {
            key: &key,
            value: &value,
            constant: false,
        });

        let data = &mut *self.data.write().map_err(|e| e.to_string())?;
        data.insert(key, value);

        Ok(())
    }

    pub fn set_const(&mut self, key: String, value: String) -> Result<'_, ()> {
        if let Ok(data) = self.constants.read() {
            if data.contains(&key) {
                return Err("key is a constant".into());
            }
        } else {
            return Err("failed to read constants".into());
        }

        let _ = self.append_to_wal(&LogLine::Set {
            key: &key,
            value: &value,
            constant: true,
        });

        let data = &mut *self.data.write().map_err(|e| e.to_string())?;
        data.insert(key, value);

        Ok(())
    }

    pub fn delete(&mut self, key: String) -> Result<'_, ()> {
        let _ = self.append_to_wal(&LogLine::Delete { key: &key });

        let data = &mut *self.data.write().map_err(|e| e.to_string())?;
        data.remove(&key);

        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Ok(data) = self.data.read() {
            data.get(key).cloned()
        } else {
            None
        }
    }
}
