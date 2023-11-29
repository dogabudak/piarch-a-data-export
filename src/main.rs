#[macro_use]
extern crate rocket;

use std::env;
use dotenv::dotenv;
use once_cell::sync::OnceCell;
use mongodb::{bson, bson::{Document, doc}, options::{ClientOptions}, sync::{Client, Database}};
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;
use rocket::http::Status;

static MONGODB: OnceCell<Database> = OnceCell::new();

#[derive(Deserr)]
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
async fn hello() -> Status {
    let database = MONGODB.get().unwrap();
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path("./output.csv").unwrap();
    wtr.write_record(&["username", "password"]);
    let collection = database.collection::<Document>("users");
    let cursor = match collection.find(None, None) {
        Ok(cursor) => cursor,
        Err(_) => return Status::NotAcceptable
    };
    // TODO this should be another thread maybe
    for doc in cursor {
        let user_doc = match doc {
            Ok(user) => {
                User {
                    username: user.get("username").unwrap().to_string(),
                    password: user.get("password").unwrap().to_string(),
                }
            }
            Err(_) => return Status::NotAcceptable
        };
        wtr.write_record(&[user_doc.username, user_doc.password]);
    };
    wtr.flush();
    Status::Accepted
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
