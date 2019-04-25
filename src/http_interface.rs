use super::persistence;
use rocket_contrib::json::Json;
use std::time::{Duration, SystemTime};
use super::rocket;


#[derive(Serialize, Deserialize, Debug, Default)]
struct Kv {
    key: String,
    value: String
}




// send out the stored object
#[get("/<key>")]
fn get(key: String) -> String {

    if let Ok(locked_obj) = persistence::DB.lock(){
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
    if let Ok(mut locked_obj) = persistence::DB.lock(){
        locked_obj.insert(
            kv.key.clone(),
            persistence::Value{
                value: kv.value.clone(),
                timestamp:SystemTime::now()
            }
        );
    }
}


pub fn start() {
    rocket::ignite()
        .mount("/", routes![post_kv])
        .mount("/get", routes![get])
        .launch();

}




