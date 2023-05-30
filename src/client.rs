use crate::key_gen::*;
use crate::rsa::*;
use num_bigint::BigUint;
use serde_json;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use std::io::Write;
use std::net::TcpStream;

pub fn send_connection_request(server_addr: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
    match TcpStream::connect(server_addr) {
        Ok(mut stream) => {
            println!("Successfully connected to server at {}", server_addr);
            let msg = b"Connection Request";
            stream.write(msg)?;
            println!("Sent connection request, awaiting reply...");
            Ok(stream)
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
            Err(e.into())
        }
    }
}

fn receive_server_public_key(
    mut stream: TcpStream,
) -> Result<(TcpStream, PublicKey), Box<dyn std::error::Error>> {
    let mut data = vec![0; 1024];
    let mut buffer = Vec::new();

    loop {
        match stream.read(&mut data) {
            Ok(size) => {
                println!("Size of data: {}, Data: {:?}", size, data);
                if size == 1 || size == 0 {
                    // No more data to read
                    break;
                }
                buffer.extend_from_slice(&data[..size]);
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    let text = serde_json::from_str(String::from_utf8(buffer).unwrap().as_str()).unwrap();
    println!("Text: {:?}", text);

    // Assuming gen_keys() function generates a pair of keys and returns the public key
    Ok((stream, text))
}

use aes::Aes128;
use rand::Rng;

fn generate_aes_key() -> [u8; 16] {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; 16];
    rng.fill(&mut key);
    key
}

// Client side
fn encrypt_and_send_aes_key(
    pub_key: PublicKey,
    aes_key: [u8; 16],
    mut stream: &TcpStream,
) -> Result<&TcpStream, Box<dyn std::error::Error>> {
    let encrypted_aes_key = rsa_encrypt_bytes(&pub_key, aes_key);

    // Send the encrypted AES key
    stream.write(&encrypted_aes_key)?;
    stream.flush()?;

    Ok(stream)
}

fn wait_for_acknowledgement(
    mut stream: &TcpStream,
) -> Result<&TcpStream, Box<dyn std::error::Error>> {
    let mut buffer = [0; 128]; // Define buffer to hold incoming data
    let mut ack_message = String::new();

    loop {
        let nbytes = stream.read(&mut buffer)?;
        if nbytes == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "No acknowledgment received from the server",
            )));
        }
        ack_message.push_str(std::str::from_utf8(&buffer[..nbytes])?);
        if ack_message.contains("ACK") {
            // "ACK" is the acknowledgment message
            break;
        }
    }
    println!("Acknowledgement received from the server");
    Ok(stream)
}
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use requestty::Question;
use std::error::Error;
use std::io::Read;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

pub fn start_communication(
    mut stream: &TcpStream,
    aes_key: [u8; 16],
) -> Result<(), Box<dyn Error>> {
    loop {
        // Get user input
        let questions = vec![Question::input("message")
            .message("Enter a message")
            .validate(|input, _| {
                if input.is_empty() {
                    Err(String::from("This field cannot be empty"))
                } else {
                    Ok(())
                }
            })
            .build()];

        let answers = requestty::prompt(questions).unwrap();

        let message: String = answers
            .get("message")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string()
            .clone();

        let message = message.as_bytes().to_vec();

        // Generate random IV
        let mut iv = [0u8; 16];
        rand::thread_rng().fill(&mut iv);

        // Create cipher for encryption
        let cipher = Aes128Cbc::new_from_slices(&aes_key, &iv).unwrap();

        // Encrypt message
        let mut cipher_text = cipher.encrypt_vec(&message);

        // Prefix the IV to our message. This is a common technique as the IV does not need to be kept secret.
        let mut data_to_send = iv.to_vec();
        data_to_send.append(&mut cipher_text);

        // Write data to the TCP stream
        stream.write_all(&data_to_send)?;

        // Prepare to read from stream
        let mut read_buffer = vec![0u8; 1024];
        let read_size = stream.read(&mut read_buffer)?;

        // Separate IV and actual data
        let received_iv = &read_buffer[0..16];
        let received_data = &read_buffer[16..read_size];

        // Create cipher for decryption
        let cipher = Aes128Cbc::new_from_slices(&aes_key, received_iv).unwrap();

        // Decrypt received data
        let decrypted_data = cipher.decrypt_vec(&received_data)?;

        // Display decrypted data
        println!(
            "Received message: {}",
            String::from_utf8_lossy(&decrypted_data)
        );
    }

    Ok(())
}

pub fn start() {
    // Here we chain our operations together. Each step depends on the success of the previous one.
    let result = send_connection_request("localhost:8000");
    let (stream, pub_key) = receive_server_public_key(result.unwrap()).unwrap();
    let aes_key = generate_aes_key();
    let stream = encrypt_and_send_aes_key(pub_key, aes_key.clone(), &stream).unwrap();
    let stream = wait_for_acknowledgement(&stream).unwrap();
    start_communication(stream, aes_key).unwrap();
}
