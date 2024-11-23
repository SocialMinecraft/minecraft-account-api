use actix_web::{post, web, HttpResponse, Responder};
use async_nats::Client;
use protobuf::{Message};
use serde::{Deserialize, Serialize};
use tracing::{error};
use crate::proto::account::Account;
use crate::proto::minecraft_account_add::AddMinecraftAccountRequest;
use crate::proto::minecraft_account_update::ChangeMinecraftAccountResponse;
use crate::state::State;
use crate::token_to_account::token_to_account;

#[derive(Serialize)]
struct Response {
    error: Option<String>,
}

#[derive(Deserialize)]
struct Request {
    minecraft_name: String,
    minecraft_uuid: Option<String>,
}

#[post("/api/{token}/minecraft_accounts")]
pub async fn add_endpoint(ctx: web::Data<State>, path: web::Path<String>,  body: web::Json<Request>) -> impl Responder {

    let token = path.as_str();

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

    // Verify username is not empty
    if body.minecraft_name.is_empty() {
        return HttpResponse::BadRequest().json(Response{error:
            Some("Minecraft account name empty".to_string())
        });
    }

    // Add Account
    let result = match add_minecraft_account(ctx.nc.clone(), &account, &body.minecraft_uuid, &body.minecraft_name).await {
        Ok(error) => error,
        Err(e) => {
            error!("Error adding account:  {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };
    if result.is_some() {
        return HttpResponse::BadRequest().json(Response{error:
            result
        });
    }

    // Send Response
    HttpResponse::Ok().json(Response{error:None})
}

async fn add_minecraft_account(nc: Client, account: &Account, uuid: &Option<String>, username: &str) -> anyhow::Result<Option<String>> {

    if account.discord_id.is_none() {
        return Ok(None);
    }

    let mut msg = AddMinecraftAccountRequest::new();
    msg.user_id = Some(account.id.clone());
    msg.deprecated_discord_id = account.discord_id.clone();
    msg.minecraft_uuid = uuid.clone();
    msg.minecraft_username = username.to_string();
    msg.first_name = account.first_name.clone().unwrap_or("".to_string());
    let encoded: Vec<u8> = msg.write_to_bytes()?;
    let result = nc.request("accounts.minecraft.add", encoded.into()).await?;
    let response = ChangeMinecraftAccountResponse::parse_from_bytes(&result.payload)?;

    Ok(response.error_message)
}