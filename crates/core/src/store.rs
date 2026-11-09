use std::{
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::PathBuf,
    sync::{Mutex, RwLock},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LogLine {
    Set {
        key: String,
        value: String,
        constant: bool,
    },
    Delete {
        key: String,
    },
}

pub struct KVStore {
    data: RwLock<HashMap<String, String>>,
    constants: RwLock<HashSet<String>>,
    wal_file: Mutex<BufWriter<File>>,
}

pub type Result<'a, T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'a>>;

impl KVStore {
    pub fn new(wal: PathBuf) -> Result<'static, Self> {
        let mut map = HashMap::new();
        let mut constants = HashSet::new();

        if wal.exists() {
            Self::replay_wal(wal.clone(), &mut map, &mut constants);
        }

        let file = OpenOptions::new().append(true).create(true).open(wal)?;
        let reader = BufWriter::new(file);

        Ok(Self {
            data: RwLock::new(HashMap::new()),
            constants: RwLock::new(HashSet::new()),
            wal_file: Mutex::new(reader),
        })
    }

    fn append_to_wal(&self, log: &LogLine) -> Result<'_, ()> {
        let file = &mut *self.wal_file.lock().map_err(|e| e.to_string())?;
        postcard::to_io(log, file)?;

        Ok(())
    }

    fn replay_wal(
        wal: PathBuf,
        map: &mut HashMap<String, String>,
        constants: &mut HashSet<String>,
    ) {
        let mut reader = BufReader::new(File::open(wal).unwrap());
        let mut buf: &mut [u8] = &mut [0u8; 1024];
        while let Ok((logline, (r, s))) =
            postcard::from_io::<LogLine, BufReader<File>>((reader, buf))
        {
            reader = r;
            buf = s;

            match logline {
                LogLine::Set {
                    key,
                    value,
                    constant,
                } => {
                    if constant {
                        map.insert(key.clone(), value);
                        constants.insert(key);
                    } else {
                        map.insert(key, value);
                    }
                }
                LogLine::Delete { key } => {
                    map.remove(&key);
                }
            }
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<'_, ()> {
        let _ = self.append_to_wal(&LogLine::Set {
            key: key.clone(),
            value: value.clone(),
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
            key: key.clone(),
            value: value.clone(),
            constant: true,
        });

        let data = &mut *self.data.write().map_err(|e| e.to_string())?;
        data.insert(key, value);

        Ok(())
    }

    pub fn delete(&mut self, key: String) -> Result<'_, ()> {
        let _ = self.append_to_wal(&LogLine::Delete { key: key.clone() });

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
