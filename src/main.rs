#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;
// #[macro_use] extern crate json;


#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde;

// use serde::{Deserialize, Serialize};
extern crate clap;
use clap::{Arg, App, SubCommand};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use log::{info, trace, warn};
use simple_logger;

// distributed db sync
mod herd;
mod http_interface;
mod persistence;


// use serde_json::from_str;
// use serde_json::to_string;


#[derive(Serialize, Deserialize, Debug)]
pub struct IbexConf {
    database: String,
    snapshot_interval: u64,
    sync_interval: u64,
    herd: Vec<String>
}

impl Default for IbexConf {
    fn default() -> IbexConf {
        IbexConf {
            database: String::from("db"),
            snapshot_interval: 10,
            sync_interval: 20,
            herd: vec![]
        }
    }
}


fn main() {
    // simple_logger::init().unwrap();

    info!("ibexDB is starting.");


    let matches = App::new("ibexDB")
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
            info!("Could not open a config file, creating default.");
            let writer = BufWriter::new(File::create("ibex.conf").unwrap());
            serde_json::to_writer_pretty(writer, &IbexConf::default()).unwrap();
            IbexConf::default()
            }
    };

    println!("=== Configuration:\n\tCommit interval: {}s\n\tHosts to sync: {:?}", conf.snapshot_interval, conf.herd);

    // Load from db
    persistence::disk_reader(&conf.database);
    // start the committer
    persistence::disk_committer(conf.database.clone(), conf.snapshot_interval);
    // start the interface
    herd::init(&conf);
    http_interface::start();

 
}