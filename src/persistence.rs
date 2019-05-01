
use std::io::BufWriter;
use std::time::{Duration, SystemTime};
use std::io::BufReader;
use log::{info, trace, warn};
use std::thread;
use std::fs::File;
extern crate lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;

use super::herd;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub timestamp: SystemTime,
    pub value: String
}

lazy_static! {
    pub static ref DB: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
    pub static ref UNSYNCED: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());


    // pub static ref UNSYNCED: Mutex<herd::SyncBucket> = Mutex::new(herd::SyncBucket::default());
}

// periodically dump the database
pub fn disk_committer(dbfile: String, interval: u64) {
    thread::spawn(move || {
        let moved_dbfile = dbfile.clone();
        loop {
            thread::sleep(Duration::from_secs(interval));
            if let Ok(obj) = DB.lock() {
                if let Ok(f) = File::create(&moved_dbfile) {
                    if let Ok(_r) = serde_json::to_writer(BufWriter::new(f), &*obj) {
                        info!("Disk snapshot complete.")
                    }    
                }
            }
            if let Ok(obj) = UNSYNCED.lock() {
                if let Ok(f) = File::create(format!("{}-unsynced", moved_dbfile)) {
                    if let Ok(_r) = serde_json::to_writer(BufWriter::new(f), &*obj) {
                        info!("Disk snapshot complete.")
                    }    
                }
            }

        }
    });
}

pub fn make_db_unsynced() {
    if let Ok(db) = DB.lock() {
        if let Ok(mut us) = UNSYNCED.lock(){
            // *us.extend(db);

            us.extend(db.clone());

            // for (key, value) in *db {
            //     // us.insert(key, value);

            // }
        }
    }
}

pub fn disk_reader(dbfile: &String) {    
 
// format!("{}-unsynced", moved_dbfile)

    if let Ok(file) = File::open(dbfile) {
        let reader = BufReader::new(file);
        if let Ok(db) = serde_json::from_reader(reader) {
            if let Ok(mut data) = DB.lock(){
                *data = db;
            }
        }
    }
    // Load unsynced files
    if let Ok(file) = File::open(format!("{}-unsynced", dbfile)) {
        let reader = BufReader::new(file);
        if let Ok(db) = serde_json::from_reader(reader) {
            if let Ok(mut data) = UNSYNCED.lock(){
                *data = db;
            }
        }
    }
}