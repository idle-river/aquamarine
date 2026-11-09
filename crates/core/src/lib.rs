use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufWriter,
    sync::{Mutex, RwLock},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LogLine<'a, 'b> {
    Set { key: &'a str, value: &'b str },
    Delete { key: &'a str },
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
        });

        let data = &mut *self.data.write().map_err(|e| e.to_string())?;

        data.insert(key, value);

        Ok(())
    }
}
