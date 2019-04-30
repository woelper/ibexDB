use super::persistence::{Value, DB, UNSYNCED};
use super::IbexConf;

use log::{info, trace, warn};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime}; // 0.6.5


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SyncBucket {
    pub unsynced_data: HashMap<String, Value>,
    pub herd: Vec<String>,
}


pub fn receive(bucket: &SyncBucket) -> Option<String> {
    // println!("got {:?} to sync", bucket);
    if let Ok(mut db) = DB.lock() {
        for (key, value) in &bucket.unsynced_data {
            println!("{}={:?}", key, value);
            db.insert(key.clone(), value.clone());
        }
        return Some(String::from("OK"));
    }
    return None;
}

pub fn send(hosts: &Vec<String>) {
    if let Ok(unsynced) = UNSYNCED.lock() {

     
        let mut shuffled_hosts = hosts.clone();
        shuffled_hosts.shuffle(&mut thread_rng());

        for (i, host) in shuffled_hosts.iter().enumerate() {

            let mut other_hosts = shuffled_hosts.clone();
            other_hosts.remove(i);
            let bucket = SyncBucket {
                unsynced_data: unsynced.clone(),
                herd: other_hosts,
            };
            let url = format!("http://{}/sync", host);

            if let Ok(_res) = reqwest::Client::new()
                .post(&url)
                .json(&bucket)
                .send()
            {
                println!("Sent to {}", url);
                break;
            } else {
                println!("Host unreachable {}", url);
            }   
        }
    }
}

pub fn sync_service(interval: u64, herd: Vec<String>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(interval));

        send(&herd);
    });
}

pub fn init(conf: &IbexConf) {
    sync_service(20, conf.herd.clone());
}