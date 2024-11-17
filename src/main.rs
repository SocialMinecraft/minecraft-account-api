mod proto;
mod util;

use anyhow::Result;
use async_nats::Client;
use proto::hello::Hello;
use protobuf::Message;
use tokio::task::JoinSet;
use tracing::info;

#[tracing::instrument]
async fn send_hello(nc: async_nats::Client, from: &str) -> Result<()> {

    // create test message
    let mut msg = Hello::new();
    msg.from = from.to_string();

    // Serialize the user to bytes
    let encoded: Vec<u8> = msg.write_to_bytes().unwrap();

    // send message
    let publisher_client = nc.clone();
    publisher_client.publish("hello", encoded.into()).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn handle_hello(nc: Client, msg: async_nats::Message) -> Result<()> {
    let decoded_msg = Hello::parse_from_bytes(&msg.payload)?;
    println!("Hello from: {}", decoded_msg.from);
    info!("Hello from: {}", decoded_msg.from);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {

    // get the app name, used for group and such
    let app_name = match util::get_app_name() {
        Some(name) => name,
        None => { return Err(anyhow::anyhow!("Could not  determine application name.")); },
    };

    // Setup logging
    util::setup_logging(app_name.as_str());

    // connect to db
    let _db = util::connect_to_database().await?;

    // connect to nats
    let nc = util::connect_to_nats().await?;

    let mut set = JoinSet::new();

    let _nc = nc.clone();
    set.spawn(async move {
        util::handle_requests(_nc, "hello", handle_hello).await.expect("hello");
    });
    /* Or * /
    set.spawn(async move {
        util::handle_requests(_nc, "vault.store", move|_nc, msg| {
            handle_hello(/ *[other params]* /, _nc, msg)
        }).await.expect("hello");
    });*/

    // send hello
    send_hello(nc.clone(), &app_name.to_string()).await?;

    set.join_all().await;
    Ok(())
}
