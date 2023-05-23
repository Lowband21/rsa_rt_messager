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
        Question::input("public_key_file")
            .message("What is the file name of your public key?")
            .default("public_key.txt")
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

    let public_key: PublicKey =
        read_public_key_from_file(answers.get("public_key_file").unwrap().as_string().unwrap())
            .unwrap();

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

    let priv_key = read_private_key_from_file("private_key.txt").unwrap();
    // Start the server listening on the given address

    //let server = start_server(&address, &priv_key);

    // Add a delay before starting the client
    //tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Then start the client
    let client = start_client(&address, public_key, &username, priv_key);

    // Run both the server and client concurrently
    tokio::try_join!(client)?;

    Ok(())
}

use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn start_client(
    server_addr: &str,
    pub_key: PublicKey,
    username: &str,
    priv_key: PrivateKey,
) -> Result<(), Box<dyn Error>> {
    println!("Starting client...");

    let mut stream = TcpStream::connect(server_addr).await?;
    println!("Connected to server...");

    let pub_key = Arc::new(pub_key);
    let priv_key = Arc::new(priv_key);

    // Read the server's public key
    let mut buf = [0; 1024];
    stream.read(&mut buf).await?;
    println!("Read data from server");

    println!("Buffer: {:?}", buf);
    let server_pub_key = PublicKey::from_bytes(&buf)?;

    // Send the client's public key
    stream.write_all(&pub_key.to_bytes().unwrap()).await?;

    println!("Exchanged public keys with the server");

    loop {
        let server_pub_key = Arc::new(&server_pub_key);
        let priv_key = Arc::clone(&priv_key);
        println!("In loop");

        // Read the nonce from the server
        let mut buf = [0; 1024];
        let n = match stream.read(&mut buf).await {
            Ok(n) => {
                println!("Received nonce from server...");
                n
            }
            Err(e) => {
                println!("Failed to read nonce from server: {}", e);
                return Err(e.into());
            }
        };
        let nonce = u64::from_be_bytes(buf[..n].try_into().unwrap());

        // Encrypt the nonce and send it back to the server
        let encrypted_nonce = rsa_encrypt(&*pub_key, &nonce.to_string());
        for block in &encrypted_nonce {
            match stream.write_all(&block.to_bytes_be()).await {
                Ok(_) => println!("Sent encrypted nonce..."),
                Err(e) => {
                    println!("Failed to send encrypted nonce: {}", e);
                    return Err(e.into());
                }
            }
        }

        /*
        // Encrypt and send the username
        let encrypted_username = rsa_encrypt(&pub_key, username);
        for block in &encrypted_username {
            match stream.write_all(&block.to_bytes_be()).await {
                Ok(_) => println!("Sent encrypted username..."),
                Err(e) => {
                    println!("Failed to send encrypted username: {}", e);
                    return Err(e.into());
                }
            }
        }

        // Read, encrypt and send the plaintext message
        let plaintext_message = read_plaintext_message()?;
        let encrypted_message = rsa_encrypt(&pub_key, &plaintext_message);
        for block in &encrypted_message {
            match stream.write_all(&block.to_bytes_be()).await {
                Ok(_) => println!("Sent encrypted message..."),
                Err(e) => {
                    println!("Failed to send encrypted message: {}", e);
                    return Err(e.into());
                }
            }
        }
        */

        // Read and print the server's response
        let mut buf = [0; 1024];
        let n = match stream.read(&mut buf).await {
            Ok(n) => {
                println!("Read {} bytes from server...", n);
                n
            }
            Err(e) => {
                println!("Failed to read from server: {}", e);
                return Err(e.into());
            }
        };

        let response_message = std::str::from_utf8(&buf[..n])?;
        println!("Response: {}", response_message);
    }
}
