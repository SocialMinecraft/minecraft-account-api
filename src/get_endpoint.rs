use actix_web::{get, web, HttpResponse, Responder};
use async_nats::Client;
use protobuf::{Message};
use serde::{ Serialize};
use tracing::{error, info};
use crate::proto::account::Account;
use crate::proto::minecraft_account::MinecraftAccount;
use crate::proto::minecraft_account_list::{ListMinecraftAccountsRequest, ListMinecraftAccountsResponse};
use crate::proto::stats::Stats;
use crate::proto::stats_get::{GetStats, GetStatsResponse};
use crate::state::State;
use crate::token_to_account::token_to_account;

#[derive(Serialize)]
struct ResponseAccountServer {
    name: String,
    playtime_sec: i32,
    deaths: i32,
}

#[derive(Serialize)]
struct ResponseAccount {
    username: String,
    uuid: String,
    playtime_sec: i32,
    deaths: i32,
    servers: Vec<ResponseAccountServer>,
}

#[derive(Serialize)]
struct Response {
    accounts: Vec<ResponseAccount>,
}

#[get("/api/{token}/minecraft_accounts")]
pub async fn get_endpoint(ctx: web::Data<State>, token: web::Path<String>) -> impl Responder {

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

    // get minecraft accounts
    let mc_accounts = match get_minecraft_accounts_legacy(ctx.nc.clone(), &account).await {
        Ok(mc_accounts) => mc_accounts,
        Err(e) => {
            error!("Error looking up Minecraft accounts:  {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    // get account stats
    let uuids = mc_accounts.iter().map(|a| a.minecraft_uuid.clone()).collect();
    let stats = match get_minecraft_account_stats(ctx.nc.clone(), uuids).await {
        Ok(stats) => stats,
        Err(e) => {
            error!("Error getting stats:  {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    // build response
    let mut resp = Response {
        accounts: vec![]
    };
    for account in mc_accounts {
        let mut x = ResponseAccount {
            uuid: account.minecraft_uuid.clone(),
            username: account.minecraft_username.clone(),
            playtime_sec: 0,
            deaths: 0,
            servers: vec![]
        };
        let uuid =
        for s in &stats {
            if !s.minecraft_uuid.eq(&account.minecraft_uuid) {
                info!("{} <-> {}", s.minecraft_uuid, account.minecraft_uuid);
                continue;
            }
            info!("party");

            let ras = ResponseAccountServer {
                name: s.server.clone(),
                playtime_sec: s.playtime.unwrap_or(0)/20,
                deaths: s.deaths.unwrap_or(0),
            };
            x.deaths = x.deaths + ras.deaths;
            x.playtime_sec = x.playtime_sec + ras.playtime_sec;
            x.servers.push(ras);
        };
        resp.accounts.push(x);
    }

    // response
    HttpResponse::Ok().json(resp)
}

async fn get_minecraft_accounts_legacy(nc: Client, account: &Account) -> anyhow::Result<Vec<MinecraftAccount>> {

    if account.discord_id.is_none() {
        return Ok(vec![]);
    }

    let mut msg = ListMinecraftAccountsRequest::new();
    msg.user_id = account.clone().discord_id.unwrap();
    let encoded: Vec<u8> = msg.write_to_bytes()?;
    let result = nc.request("accounts.minecraft.list", encoded.into()).await?;
    let response = ListMinecraftAccountsResponse::parse_from_bytes(&result.payload)?;

    Ok(response.accounts)
}

async fn get_minecraft_account_stats(nc: Client, uuid: Vec<String>) -> anyhow::Result<Vec<Stats>> {

    let mut msg = GetStats::new();
    msg.minecraft_ids = uuid;
    let encoded: Vec<u8> = msg.write_to_bytes()?;
    let result = nc.request("stats.get", encoded.into()).await?;
    let response = GetStatsResponse::parse_from_bytes(&result.payload)?;

    Ok(response.stats)
}