// src/messager.rs
//use futures::{SinkExt, StreamExt};
use crate::key_gen::*;
use crate::rsa::*;
use requestty::{prompt, Question};
//use std::io::Write;
//use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use num_bigint::{BigInt, BigUint, RandBigInt, Sign::Plus, ToBigUint};
use std::io::{self, Read};

#[tokio::main]
pub async fn messager_start() -> Result<(), Box<dyn Error>> {
    // Ask for the server's address and username
    let questions = vec![
        Question::input("address")
            .message("What is the server's address?")
            .default("localhost:8000")
            .validate(|input, _| {
                if input.is_empty() {
                    Err(String::from("This field cannot be empty"))
                } else {
                    Ok(())
                }
            })
            .build(),
        Question::input("username")
            .message("What is your username?")
            .default("lowband")
            .validate(|input, _| {
                if input.is_empty() {
                    Err(String::from("This field cannot be empty"))
                } else {
                    Ok(())
                }
            })
            .build(),
    ];
    let answers = prompt(questions).unwrap();

    let username: String = answers
        .get("username")
        .unwrap()
        .as_string()
        .unwrap()
        .to_string()
        .clone();

    // Get the address from the user input
    let address: String = answers
        .get("address")
        .unwrap()
        .as_string()
        .unwrap()
        .to_string()
        .clone();

    // Start the server listening on the given address

    //let server = start_server(&address, &priv_key);

    // Add a delay before starting the client
    //tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Then start the client
    let client = connect_to_server();

    // Run both the server and client concurrently
    tokio::try_join!(client)?;

    Ok(())
}

use crate::types::{EncryptAbleMessage, RsaKey};
use std::error::Error;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream; // Assuming you have types.rs in the same crate

async fn connect_to_server() -> Result<(), Box<dyn Error>> {
    // Change the IP address and port to match your server's
    let mut stream = TcpStream::connect("127.0.0.1:9000").await?;

    // Read RSA keys from files
    let rsa_key = RsaKey::from_files(Path::new("public_key.txt"), Path::new("private_key.txt"))?;

    let message = EncryptAbleMessage::new(rsa_key.public.clone(), "Hello, Server!".to_owned());

    // Send public key to the server
    let key_string = rsa_key.public.clone().to_string();
    stream.write_all(key_string.as_bytes()).await?;

    // Read server response (server's challenge)
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    let encrypted_secret = BigUint::from_bytes_le(&buf[..n]);

    // Decrypt the secret with private key and send it back to the server
    let decrypted_secret = rsa_decrypt_biguint(&rsa_key.private, encrypted_secret);
    stream
        .write_all(decrypted_secret.to_string().as_bytes())
        .await?;

    // Read server response (verification result)
    let n = stream.read(&mut buf).await?;
    let server_response = String::from_utf8_lossy(&buf[..n]);
    println!("Server response: {}", server_response);

    // Check if verification failed
    if server_response != "VERIFIED" {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Verification failed",
        )));
    }

    // Encrypt the message and send to server
    let encrypted_message = message.encrypt(rsa_key.public);
    for encrypted_part in encrypted_message {
        let encrypted_string = encrypted_part.to_string();
        stream.write_all(encrypted_string.as_bytes()).await?;
    }

    Ok(())
}

use crate::types::PrivateRSAKey;

fn rsa_decrypt_biguint(private_key: &PrivateRSAKey, cipher_text: BigUint) -> BigUint {
    cipher_text.modpow(&private_key.private_d, &private_key.private_phi_n)
}
