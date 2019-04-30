use super::persistence::{Value, DB, UNSYNCED, clear_unsynced};
use rocket_contrib::json::Json;
use std::time::{Duration, SystemTime};
use super::rocket;
use super::herd::{SyncBucket, receive};
use super::rocket::response::content;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Kv {
    key: String,
    value: String
}

// send out the stored object
#[get("/<key>")]
fn get(key: String) -> String {

    if let Ok(locked_obj) = DB.lock(){
        if let Some(k) = locked_obj.get(&key){
            return k.value.clone();
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
    let timestamp = SystemTime::now();

    if let Ok(mut locked_obj) = DB.lock(){
        locked_obj.insert(
            kv.key.clone(),
            Value{
                value: kv.value.clone(),
                timestamp: timestamp
            }
        );
    }
    if let Ok(mut locked_obj) = UNSYNCED.lock(){
        locked_obj.insert(
            kv.key.clone(),
            Value{
                value: kv.value.clone(),
                timestamp: timestamp
            }
        );
    }
}


#[post("/sync", format = "application/json", data = "<kv>")]
fn post_sync(kv: Json<SyncBucket>) -> content::Json<&'static str> {
    // println!("got {:?} to sync", kv.into_inner());
    match receive(&kv.into_inner()) {
        Some(_res) => {
            clear_unsynced();
            content::Json("{ 'status': 'ok' }")
            },
        None => content::Json("{ 'status': 'error' }")
    }
}


pub fn start() {
    rocket::ignite()
        .mount("/", routes![post_kv])
        .mount("/", routes![post_sync])
        .mount("/get", routes![get])
        .launch();
}




