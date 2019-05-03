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
            db.insert(key.clone(), value.clone());
        }
        info!("Received {} keys", &bucket.unsynced_data.len());
        return Some(String::from("OK"));
    }
    return None;
}

pub fn send(hosts: &Vec<String>) {
    if let Ok(mut unsynced) = UNSYNCED.lock() {

        if unsynced.is_empty() {
            return
        }
        let mut shuffled_hosts = hosts.clone();
        shuffled_hosts.shuffle(&mut thread_rng());

        for (i, host) in shuffled_hosts.iter().enumerate() {
            info!("Trying to sync {} keys to {}", unsynced.len(), host);
            let mut other_hosts = shuffled_hosts.clone();
            other_hosts.remove(i);
            let bucket = SyncBucket {
                unsynced_data: unsynced.clone(),
                herd: other_hosts,
            };
            let url = format!("http://{}/sync", host);

            match reqwest::Client::new()
                .post(&url)
                .json(&bucket)
                .send()
            {
                Ok(res) => {
                    info!("Data sent to {}", url);
                    unsynced.clear();
                    break;
                },
                Err(e) => warn!("Host unreachable {}", url)
            }  
        }
    } else {dbg!("Can't lock for sending");}
}

pub fn sync_service(interval: u64, herd: Vec<String>) {
    thread::spawn(move || loop {
        send(&herd);
        thread::sleep(Duration::from_secs(interval));
    });
}

pub fn init(conf: &IbexConf) {
    info!("Starting sync service");
    sync_service(conf.sync_interval, conf.herd.clone());
}