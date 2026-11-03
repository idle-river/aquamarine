use std::path::PathBuf;

use aquamarine::KVStore;

fn main() {
    let store = KVStore::new(PathBuf::from("./aquamarine.log"));

    for i in 0..=100 {
        let res = store.get(format!("{}", i));
        if let Some(res) = res {
            println!("{res}");
        } else {
            println!("None");
        }
    }
}
