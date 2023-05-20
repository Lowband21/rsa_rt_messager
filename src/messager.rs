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

    let server = start_server(&address, &priv_key);

    // Add a delay before starting the client
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Then start the client
    let client = start_client(&address, &username, &public_key);

    // Run both the server and client concurrently
    tokio::try_join!(server, client)?;

    Ok(())
}

use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn start_client(
    peer_addr: &str,
    username: &str,
    public_key: &PublicKey,
) -> Result<(), Box<dyn Error>> {
    // Set up a TCP stream
    let mut stream = match TcpStream::connect(peer_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect: {:?}", e);
            return Err(e.into());
        }
    };

    // Ask for input and then encrypt it using the public key
    let mut input = String::new();
    println!("Write your message: ");
    std::io::stdin().read_line(&mut input)?;
    let message = format!("{}: {}", username, input.trim());

    let encrypted_message = rsa_encrypt(public_key, &message);

    // Convert the encrypted message into bytes and send it over the network
    for block in &encrypted_message {
        let block_bytes = block.to_bytes_be();
        stream.write_all(&block_bytes).await?;
    }

    Ok(())
}

async fn start_server(local_addr: &str, priv_key: &PrivateKey) -> Result<(), Box<dyn Error>> {
    // Listen for incoming connections
    let listener = TcpListener::bind(local_addr).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        let priv_key = priv_key.clone();

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Convert the received bytes to a BigUint
                let message_as_big_uint = BigUint::from_bytes_be(&buf[..n]);

                // Decrypt the message
                let decrypted_message = rsa_decrypt(&priv_key, &vec![message_as_big_uint]);

                // Handle the case where the decrypted message isn't valid UTF-8
                let message = match decrypted_message {
                    Ok(msg) => msg,
                    Err(_) => "Received message isn't valid UTF-8".to_string(),
                };

                println!("Received: {}", message);

                // Echo the data back
                if let Err(e) = socket.write_all(message.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
