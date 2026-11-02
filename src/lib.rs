use std::{
    collections::HashMap, fs::{ File, OpenOptions }, io::{ BufReader, BufWriter, Read, Write }, path::PathBuf, sync::{ Mutex, RwLock },
};
use rkyv::{Archive, Deserialize, Serialize, rancor::Error};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum Logline {
    Set { key: String, value: String },
    Delete { key: String },
}

#[derive(Debug)]
pub struct KVStore {
    data: RwLock<HashMap<String, String>>,
    wal_file: Mutex<BufWriter<File>>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl KVStore {
    pub fn new(wal_path: PathBuf) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(wal_path.clone())
            .expect("Failed to open WAL file");

        let wal_file = Mutex::new(BufWriter::new(file));

        let data = RwLock::new(HashMap::new());

        let store = KVStore { data, wal_file };

        store
    }

    fn append_to_wal(&self, log: &Logline) -> Result<()> {
        let mut lock = self
            .wal_file
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let bytes= rkyv::to_bytes::<Error>(log)?;

        lock.write_all(bytes.as_ref())?;

        Ok(())
    }
}
