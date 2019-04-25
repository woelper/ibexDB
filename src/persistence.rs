
use std::io::BufWriter;
use std::time::{Duration, SystemTime};
use std::io::BufReader;

use std::thread;
use std::fs::File;
extern crate lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Value {
    pub timestamp: SystemTime,
    pub value: String
}

lazy_static! {
    pub static ref DB: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
}

// periodically dump the database
pub fn disk_committer(dbfile: String, interval: u64) {
    thread::spawn(move || {
        let moved_dbfile = dbfile.clone();
        loop {
            thread::sleep(Duration::from_secs(interval));
            match DB.lock() {
                Ok(obj) => {
                    if let Ok(f) = File::create(&moved_dbfile) {
                        if let Ok(_r) = serde_json::to_writer_pretty(BufWriter::new(f), &*obj) {
                            println!("Disk snapshot to <{}> done", &dbfile)
                        }    
                    }
                },
                _ => ()
            }
        }
    });
}


pub fn disk_reader(dbfile: &String) {    
 
    if let Ok(file) = File::open(dbfile) {
        let reader = BufReader::new(file);
        if let Ok(db) = serde_json::from_reader(reader) {
            if let Ok(mut data) = DB.lock(){
                *data = db;
            }
        }
    }
}