#![allow(non_snake_case)]

use clap::{Parser, ValueEnum};
use azure_identity::DefaultAzureCredentialBuilder;
use azure_security_keyvault::prelude::*;
use dioxus::prelude::*;
use std::{fmt, sync::Arc};
use std::fmt::Formatter;



// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
struct GenerateApiKeyError(String);

type GenerateAPiKeyResult<T> = Result<T, GenerateApiKeyError>;

#[derive(Debug, Clone, Copy,ValueEnum)]
enum Mode {
    Console,
    GUI,
    Unknown,
}
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Console => write!(f,"console"),
            Mode::GUI => write!(f,"gui"),
            Mode::Unknown => write!(f,"unknown"),
        }
    }
}
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Application ID (ex. AP1234)
    #[arg(short, long)]
    app_id: String,

    /// Azure KeyVault URLs (ex. https://xxx.vault.azure.net/,https://yyy.vault.azure.net/ )
    #[arg(short, long,value_parser, num_args = 1.., value_delimiter = ',')]
    key_vault_urls: Vec<String>,

    /// API Key Size , Default is 32 characters
    #[arg(short, long, default_value_t = 32)]
    size: usize,

    ///Mode , Default is console
    #[arg(short, long, value_enum, default_value_t = Mode::Console)]
    mode: Mode,
}

fn generate_api_key(key_len : usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    //const API_KEY_LEN: usize = 32;
    let mut rng = rand::thread_rng();
    let api_key: String = (0..key_len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    api_key
}
async fn generate_and_save_api_key(app_id : String,
                                   mut kv_urls: Vec<String>,
                                   key_size : usize) -> GenerateAPiKeyResult<bool>{

    if kv_urls.len() <= 0 {
        return Err(GenerateApiKeyError(String::new()));
    }

    let creds = Arc::new(
        DefaultAzureCredentialBuilder::new()
            .exclude_managed_identity_credential()
            .build(),
    );

    while let Some(key_vault_url) = kv_urls.pop(){
        let client = SecretClient::new(&key_vault_url, creds.clone());
        match client {
            Ok(client) => {
                let api_key = generate_api_key(key_size);
                println!("api key : {}",&api_key);
                println!("saving to : {}",&key_vault_url);

                let secret_name = format!("{}-api-key",app_id);
                client.set(&secret_name, api_key).await.expect("Set Failed");
                let _secret = client.get(secret_name).await.expect("Get Failed");
                //assert_eq!(secret.value, api_key.clone());
            }
            Err(_) => {
                println!("Error");
            }
        }
    }
    Ok(true)
}
// define a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            "Hello, world!"
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    //println!("{}", generate_api_key(args.size));
    match args.mode {
        Mode::Console => {
            //println!("{}", generate_api_key(args.size));
           let result =  generate_and_save_api_key(args.app_id,
            args.key_vault_urls,
            args.size);
             match result.await {
                 Ok(_) => {

                 }
                 Err(_) => {
                 }
             }
        }
        Mode::GUI => {
            // launch the dioxus app in a webview
            dioxus_desktop::launch(App);
        }
        Mode::Unknown => {

        }
    }
    Ok(())
}