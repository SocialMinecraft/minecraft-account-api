use actix_web::{delete, web, HttpResponse, Responder};
use async_nats::Client;
use protobuf::{Message};
use serde::Serialize;
use tracing::{error, warn};
use uuid::Uuid;
use crate::proto::account::Account;
use crate::proto::minecraft_account_remove::RemoveMinecraftAccountRequest;
use crate::proto::minecraft_account_update::{ChangeMinecraftAccountResponse};
use crate::state::State;
use crate::token_to_account::token_to_account;

#[derive(Serialize)]
struct Response {
    error: Option<String>,
}

#[delete("/api/{token}/minecraft_accounts/{uuid}")]
pub async fn remove_endpoint(ctx: web::Data<State>, token: web::Path<String>, uuid: web::Path<String>) -> impl Responder {

    // validate uuid
    match Uuid::parse_str(uuid.as_str()) {
        Ok(_) => {}
        Err(e) => {
            warn!("Invalid UUID supplied: {}", e);
            return HttpResponse::BadRequest().body("Could not parse UUID");
        }
    }

    // get account
    let account = match token_to_account(ctx.nc.clone(), &token).await {
        Ok(account) => account,
        Err(e) => {
            error!("Error looking up account:  {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        },
    };
    if account.is_none() {
        return HttpResponse::NotFound().body("Account not found");
    }
    let account = account.unwrap();

    // Remove account
    let error = match remove_minecraft_accounts_legacy(ctx.nc.clone(), &account, uuid.as_str()).await {
        Ok(error) => error,
        Err(error) => {
            error!("Error removing account:  {}", error);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };


    // response
    HttpResponse::Ok().json(Response {
        error
    })
}

async fn remove_minecraft_accounts_legacy(nc: Client, account: &Account, uuid: &str) -> anyhow::Result<Option<String>> {

    if account.discord_id.is_none() {
        return Ok(None);
    }

    let mut msg = RemoveMinecraftAccountRequest::new();
    msg.user_id = account.discord_id.clone().unwrap(); // todo - switch to account id
    msg.minecraft_uuid = Some(uuid.to_string());
    let encoded: Vec<u8> = msg.write_to_bytes()?;
    let result = nc.request("accounts.minecraft.remove", encoded.into()).await?;
    let response = ChangeMinecraftAccountResponse::parse_from_bytes(&result.payload)?;

    Ok(response.error_message)
}