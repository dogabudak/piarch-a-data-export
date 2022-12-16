#[macro_use]
extern crate rocket;

use std::env;
use dotenv::dotenv;
use once_cell::sync::OnceCell;
use mongodb::{bson, bson::{Document, doc}, options::{ClientOptions}, sync::{Client, Database}};
use serde::{Deserialize, Serialize};

static MONGODB: OnceCell<Database> = OnceCell::new();

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

pub fn initialize_database(connection_string: String) {
    if MONGODB.get().is_some() {
        return;
    }
    if let Ok(client_options) = ClientOptions::parse(connection_string) {
        if let Ok(client) = Client::with_options(client_options) {
            let _ = MONGODB.set(client.database("piarka"));
        }
    }
}

#[get("/")]
async fn hello() -> &'static str {
    let database = MONGODB.get().unwrap();
    let mut wtr = csv::WriterBuilder::new()
        // TODO check the exported csv
        .delimiter(b'\t')
        .from_path("./output.csv").unwrap();
    wtr.write_record(&["username", "password"]);
    let collection = database.collection::<Document>("users");
    let cursor = match collection.find(None, None) {
        Ok(cursor) => cursor,
        // TODO return empty cursor
        Err(_) => return "Err"// return vec![],
    };
    // TODO this should be another thread maybe
    for doc in cursor {
        let userDoc = match doc {
            Ok(user) => {
                User {
                    username: user.get("username").unwrap().to_string(),
                    password: user.get("password").unwrap().to_string(),
                }
            }
            Err(_) => return "Finito"
        };
        wtr.write_record(&[userDoc.username, userDoc.password]);
    };
    wtr.flush();
    // TODO this should return 200 ok
    "Hello world"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();
    let connection_string = match env::var("MONGODB") {
        Ok(v) => v,
        Err(e) => panic!("{}", e)
    };
    initialize_database(connection_string);
    let _rocket = rocket::build()
        .mount("/get-user-location-records", routes![hello])
        .launch()
        .await?;

    Ok(())
}