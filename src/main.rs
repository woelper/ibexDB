#![feature(proc_macro_hygiene, decl_macro)]


#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;
// #[macro_use] extern crate json;


#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde;

use serde::{Deserialize, Serialize};
extern crate clap;
use clap::{Arg, App, SubCommand};
use rocket_contrib::json::Json;
// use std::io::prelude::*;
use std::fs::File;
// use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use std::collections::HashMap;
use std::io::BufWriter;
use std::io::BufReader;
// use rocket::response::NamedFile;

// use serde_json::from_str;
// use serde_json::to_string;

#[derive(Serialize, Deserialize, Debug)]
struct Kv {
    key: String,
    value: String
}

#[derive(Serialize, Deserialize, Debug)]
struct IbexConf {
    database: String,
    snapshot_interval: u64
}


impl Default for IbexConf {
    fn default() -> IbexConf {
        IbexConf {
            database: String::from("db"),
            snapshot_interval: 10,
        }
    }
}


lazy_static! {
    static ref OBJECT: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

// send out the stored object
#[get("/<key>")]
fn get(key: String) -> String {

    if let Ok(locked_obj) = OBJECT.lock(){
        if let Some(k) = locked_obj.get(&key){
            return k.clone();
        }
    }

    String::from("{}")

}

//set the storage TODO: implement
#[get("/<obj>")]
fn set(obj: String)  {

}



#[post("/add", format = "application/json", data = "<kv>")]
fn post_kv(kv: Json<Kv>) {
    if let Ok(mut locked_obj) = OBJECT.lock(){
        locked_obj.insert(kv.key.clone(), kv.value.clone());
    }
}



// periodically dump the database
fn disk_committer(dbfile: String, interval: u64) {
    thread::spawn(move || {
        let moved_dbfile = dbfile.clone();
        loop {
            thread::sleep(Duration::from_secs(interval));
            match OBJECT.lock() {
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

fn read_db(dbfile: &String) {    
 
    if let Ok(file) = File::open(dbfile) {
        let reader = BufReader::new(file);
        if let Ok(db) = serde_json::from_reader(reader) {
            if let Ok(mut data) = OBJECT.lock(){
                *data = db;
            }

        }
    }

}

fn main() {

    let matches = App::new("ibexDB")
        .version("1.0")
        .author("Johann Woelper. <woelper@gmail.com>")
        .arg(Arg::with_name("config")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
            .get_matches();

   
    let conf = match File::open(matches.value_of("config").unwrap_or("ibex.conf")) {
        Ok(ibexconf) => {
            let reader = BufReader::new(ibexconf);
            serde_json::from_reader(reader).unwrap_or_default()
        }
        Err(_e) => {
            println!("Could not open a config file, using default.");
            IbexConf::default()
            }
    };


    // Load from db
    read_db(&conf.database);
    //start the committer
    disk_committer(conf.database.clone(), conf.snapshot_interval);

    rocket::ignite()
    .mount("/", routes![post_kv])
    .mount("/get", routes![get])
    .launch();
}