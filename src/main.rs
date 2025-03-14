#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;

mod api;
mod error;
mod models;
mod rpc;

use dotenv::dotenv;
use rocket::fs::{FileServer, relative};
use rocket::serde::json::Json;

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    env_logger::init();

    rocket::build()
        .mount("/", routes![api::transactions::get_wallet_transactions])
        .mount("/static", FileServer::from(relative!("static")))
}
