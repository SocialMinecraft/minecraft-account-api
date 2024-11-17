mod proto;
mod util;
mod state;
mod token_to_account;

use std::env;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use crate::state::State;

fn get_listen() -> Result<String> {
    match env::var("LISTEN") {
        Ok(value) => Ok(value),
        Err(e) => {
            Err(anyhow::anyhow!("Couldn't read LISTEN environment variable: {}", e))
        },
    }
}

#[actix_web::main]
async fn main() -> Result<()> {

    // get the app name, used for group and such
    let app_name = match util::get_app_name() {
        Some(name) => name,
        None => { return Err(anyhow::anyhow!("Could not  determine application name.")); },
    };

    // get listen
    let listen = get_listen()?;

    // Setup logging
    util::setup_logging(app_name.as_str());

    // connect to nats
    let nc = util::connect_to_nats().await?;

    HttpServer::new( move || {
        App::new()
            .app_data(web::Data::new(State::new(nc.clone())))
            //.service(get_endpoint::get_endpoint)
            //.service(update_endpoint::update_endpoint)
    })
        .bind(listen)?
        .run()
        .await?;

    Ok(())
}
