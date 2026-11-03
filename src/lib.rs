use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
    sync::{Mutex, RwLock},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Logline {
    Set { key: String, value: String },
    Delete { key: String },
}

pub struct KVStore {
    data: RwLock<HashMap<String, String>>,
    wal_file: Mutex<BufWriter<File>>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

impl KVStore {
    pub fn new<P: AsRef<Path> + Clone>(wal_path: P) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(wal_path.clone())
            .expect("Failed to open WAL file");

        let wal_file = Mutex::new(BufWriter::new(file));
        let data = Self::replay_wal(wal_path).unwrap();

        KVStore {
            data: RwLock::new(data),
            wal_file,
        }
    }

    fn append_to_wal(&self, log: &Logline) -> Result<()> {
        let mut lock = self
            .wal_file
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        postcard::to_io(log, &mut *lock)?;

        Ok(())
    }

    fn replay_wal<P: AsRef<Path> + Clone>(wal_path: P) -> Result<HashMap<String, String>> {
        if wal_path.as_ref().exists() {
            let file = File::open(wal_path)?;
            let mut map = HashMap::new();
            let mut reader = BufReader::new(file);
            let mut scratch = [0u8; 1024];

            while let Ok(msg) = postcard::from_io::<Logline, _>((&mut reader, &mut scratch)) {
                let (line, _) = msg;

                match line {
                    Logline::Set { key, value } => map.insert(key, value),
                    Logline::Delete { key } => map.remove(&key),
                };
            }

            Ok(map)
        } else {
            Ok(HashMap::new())
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.append_to_wal(&Logline::Set {
            key: key.clone(),
            value: value.clone(),
        })?;
        let mut map = self
            .data
            .write()
            .map_err(|e| std::io::Error::other(format!("RwLock poisoned: {e}")))?;

        map.insert(key, value);

        Ok(())
    }

    pub fn delete(&mut self, key: String) -> Result<()> {
        let mut map = self
            .data
            .write()
            .map_err(|e| std::io::Error::other(format!("RwLock poisoned: {e}")))?;

        map.remove(&key);

        Ok(())
    }

    pub fn get(&self, key: String) -> Option<String> {
        let Ok(map) = self.data.read() else {
            return None;
        };
        let res = map.get(&key)?;

        Some(res.clone())
    }
}
