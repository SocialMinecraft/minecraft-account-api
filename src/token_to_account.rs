use protobuf::Message;
use async_nats::Client;
use tracing::info;
use crate::proto::account::Account;
use crate::proto::account_access_validate::{ValidateAccountAccessToken, ValidateAccountAccessTokenResponse};
use crate::proto::account_get::{GetAccount, GetAccountResponse};

pub async fn token_to_account(nc: Client, token: &str) -> anyhow::Result<Option<Account>> {

    // get account id
    let account_id = token_to_account_id(nc.clone(), token).await?;
    if account_id.is_none() {
        info!("No Account Found for token: {}", token);
        return Ok(None);
    }

    // get account
    let mut msg = GetAccount::new();
    msg.account_id = account_id.clone();
    let encoded: Vec<u8> = msg.write_to_bytes()?;
    let result = nc.request("accounts.get", encoded.into()).await?;
    let response = GetAccountResponse::parse_from_bytes(&result.payload)?;

    if response.account.is_none() {
        info!("No Account Found for id: {}", account_id.clone().unwrap());
        return Ok(None);
    }
    Ok(Some(response.account.unwrap()))
}

async fn token_to_account_id(nc: Client, token: &str) -> anyhow::Result<Option<String>> {

    // create message
    let mut msg = ValidateAccountAccessToken::new();
    msg.token = token.to_string();

    // Serialize the user to bytes
    let encoded: Vec<u8> = msg.write_to_bytes()?;

    // send message and get reply
    let result = nc.request("accounts.access.verify", encoded.into()).await?;
    let response = ValidateAccountAccessTokenResponse::parse_from_bytes(&result.payload)?;
    Ok(response.account_id)
}