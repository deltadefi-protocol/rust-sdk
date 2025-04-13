use deltadefi::{accounts::GetOperationKeyResponse, DeltaDeFi};
use dotenv::dotenv;
use std::env;

#[tokio::test]
async fn test_get_operation_key() {
    dotenv().ok();
    let api_key = env::var("DELTADEFI_API_KEY").unwrap();
    let deltadefi = DeltaDeFi::new(api_key, whisky::Network::Preprod, None);

    let response = deltadefi.accounts.get_operation_key().await;
    match response {
        Ok(GetOperationKeyResponse {
            encrypted_operation_key,
            operation_key_hash,
        }) => {
            println!("Encrypted Operation Key: {}", encrypted_operation_key);
            println!("Operation Key Hash: {}", operation_key_hash);
        }
        Err(e) => {
            panic!("Failed to get operation key: {:?}", e);
        }
    }
}
