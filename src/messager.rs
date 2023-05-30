// src/messager.rs
//use futures::{SinkExt, StreamExt};
use crate::key_gen::*;
use crate::rsa::*;
use requestty::{prompt, Question};
//use std::io::Write;
//use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::types::{EncryptAbleMessage, Message, PublicRSAKey, RsaKey};
use bytes::BytesMut;
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

    println!("Sent public key to server...");

    // Read server response (server's challenge)
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    let encrypted_secret = BigUint::from_bytes_le(&buf[..n]);

    // Decrypt the secret with private key and send it back to the server
    let decrypted_secret = rsa_decrypt_biguint(&rsa_key.private, encrypted_secret);
    stream
        .write_all(decrypted_secret.to_string().as_bytes())
        .await?;

    println!("Decrypted nonce and sent it back to server...");

    /*
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
    let encrypted_message = message.encrypt(rsa_key.public.clone());
    for encrypted_part in encrypted_message {
        let encrypted_string = encrypted_part.to_string();
        stream.write_all(encrypted_string.as_bytes()).await?;
    }
    */

    println!("About to handle messages");

    let result = handle_messages(stream, rsa_key).await;
    result.unwrap();
    Ok(())
}
use crypto::aes::ecb_encryptor;
use crypto::aes::KeySize;
use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::symmetriccipher::BlockDecryptor;
use rand::Rng;

async fn handle_messages(mut stream: TcpStream, rsa_key: RsaKey) -> Result<(), Box<dyn Error>> {
    let mut buffer = BytesMut::new();
    let pub_key = PublicRSAKey::from_file(Path::new("public_key.txt")).unwrap();

    loop {
        // Here, we could add code to receive messages from the user interface,
        // encrypt them, and send them to the server.
        //let message_text = read_plaintext_message()?; // For now, read from stdin
        let question = vec![Question::input("message")
            .message(">")
            .validate(|input, _| {
                if input.is_empty() {
                    Err(String::from("This field cannot be empty"))
                } else {
                    Ok(())
                }
            })
            .build()];
        let answer = prompt(question).unwrap();

        let message_send: String = answer
            .get("message")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string()
            .clone();

        // Send message to the server
        stream
            .write_all(
                format!("{},{}-{}", pub_key.public_n, pub_key.public_e, message_send).as_bytes(),
            )
            .await?;

        // Receive and decrypt messages from the server
        let mut message = vec![0; 4096];
        let bytes_read = stream.read(&mut message).await?;

        // Truncate the message to the actual length
        message.truncate(bytes_read);

        let message_recieve = String::from_utf8(message.to_vec())?;

        // Print decrypted message to console
        println!("Received: {}", message_recieve);
    }
}
/*
async fn handle_messages(
    mut stream: TcpStream,
    rsa_key: RsaKey,
    other_public_key: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let mut rng = rand::thread_rng();

    // Generate a random AES key
    let aes_key: [u8; 32] = rng.gen();

    // Encrypt the AES key with the other client's public RSA key
    let rsa = Rsa::public_key_from_der(&other_public_key)?;
    let encrypted_aes_key = rsa.public_encrypt(&aes_key, openssl::rsa::Padding::PKCS1)?;

    // Send the encrypted AES key to the server
    stream.write_all(&encrypted_aes_key).await?;

    let mut buffer = BytesMut::new();

    loop {
        // Receive and decrypt messages from the server
        let mut encrypted_message = vec![0; 4096];
        let bytes_read = stream.read(&mut encrypted_message).await?;

        // Truncate the message to the actual length
        encrypted_message.truncate(bytes_read);

        // Decrypt the AES key with our private RSA key
        let aes_key = rsa_key.private_decrypt(&encrypted_aes_key, openssl::rsa::Padding::PKCS1)?;

        // Decrypt the message with AES
        let mut decryptor =
            ecb_encryptor(KeySize::KeySize256, &aes_key, crypto::blockmodes::NoPadding);
        let mut reader = crypto::buffer::RefReadBuffer::new(&encrypted_message);
        let mut buffer = [0; 4096];
        let mut writer = crypto::buffer::RefWriteBuffer::new(&mut buffer);
        let result = decryptor.decrypt(&mut reader, &mut writer, true);

        match result {
            Ok(BufferResult::BufferUnderflow) => {}
            Ok(BufferResult::BufferOverflow) => {
                return Err("unexpected buffer overflow".into());
            }
            Err(_) => {
                return Err("decryption failed".into());
            }
        }

        let decrypted_ciphertext = writer.take_read_buffer().take_remaining();
        // Convert bytes to string
        let message = String::from_utf8(decrypted_ciphertext.to_vec())?;

        // Print decrypted message to console
        println!("Received: {}", message);

        // Here, we could add code to receive messages from the user interface,
        // encrypt them, and send them to the server.
        let message_text = read_plaintext_message()?; // For now, read from stdin

        // Encrypt the message with AES
        let mut encryptor =
            ecb_encryptor(KeySize::KeySize256, &aes_key, crypto::blockmodes::NoPadding);
        let mut reader = crypto::buffer::RefReadBuffer::new(message_text.as_bytes());
        let mut buffer = [0; 4096];
        let mut writer = crypto::buffer::RefWriteBuffer::new(&mut buffer);
        let result = encryptor.encrypt(&mut reader, &mut writer, true);

        match result {
            Ok(BufferResult::BufferUnderflow) => {}
            Ok(BufferResult::BufferOverflow) => {
                return Err("unexpected buffer overflow".into());
            }
            Err(_) => {
                return Err("encryption failed".into());
            }
        }

        let encrypted_message = writer.take_read_buffer().take_remaining();

        // Send message to the server
        stream.write_all(encrypted_message).await?;
    }
}
*/

use crate::types::PrivateRSAKey;

fn rsa_decrypt_biguint(private_key: &PrivateRSAKey, cipher_text: BigUint) -> BigUint {
    cipher_text.modpow(&private_key.private_d, &private_key.private_phi_n)
}
