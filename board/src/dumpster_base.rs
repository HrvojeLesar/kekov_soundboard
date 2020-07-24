use serde::{Serialize, Deserialize};
use std::fs::{read_to_string, write, read_dir};
use std::collections::hash_map::HashMap;
use std::sync::RwLock;

use crate::MAIN_DIR;

const DUMPSTER_BASE: &str = "dumpster_base.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct DumpsterBaseJson {
    pub full_file_name: String,
    pub without_extention: String,
    pub display_name: String,
}

pub struct RwLockedDumpster {
    pub dumpster_base_struct: RwLock<HashMap<String, DumpsterBaseJson>>,
}

pub fn read_db() -> HashMap<String, DumpsterBaseJson> {
    match read_to_string(DUMPSTER_BASE) {
        Ok(d) => serde_json::from_str::<HashMap<String, DumpsterBaseJson>>(&d).unwrap(),
        Err(_) => {
            let d = create_db_entries();
            write(DUMPSTER_BASE, serde_json::to_string(&d).unwrap()).unwrap();
            d
        }
    }
}

pub fn update_dumpster_db(hm: &mut HashMap<String, DumpsterBaseJson>) -> Result<(), std::io::Error> {
    write(DUMPSTER_BASE, serde_json::to_string(&hm).unwrap())
}

fn create_db_entries() -> HashMap<String, DumpsterBaseJson> {
    let mut dump: HashMap<String, DumpsterBaseJson> = HashMap::new();
    for file in std::fs::read_dir(MAIN_DIR).unwrap() {
        let file = file.unwrap();
        let file_without_extention = file.file_name().to_str().unwrap().split(".").collect::<Vec<&str>>()[0].to_owned();
        let full_file_name = String::from(file.file_name().to_str().unwrap());

        dump.insert(
            full_file_name.clone(),
            DumpsterBaseJson {
                full_file_name: full_file_name,
                without_extention: file_without_extention.clone(),
                display_name: file_without_extention,
            }
        );
    }
    dump
}