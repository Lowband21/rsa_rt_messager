// src/messager.rs
use futures::{SinkExt, StreamExt};
use requestty::{prompt, Question};
use std::io::{self, Write};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    // Ask for the server's address and username
    let questions = vec![
        Question::input("address")
            .message("What is the server's address?")
            .default("localhost:3012")
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

    // Connect to the server
    let (ws_stream, _) = connect_async(answers.get("address").unwrap().as_string().unwrap())
        .await
        .expect("Failed to connect");

    println!("Successfully connected");

    let username: String = answers
        .get("username")
        .unwrap()
        .as_string()
        .unwrap()
        .to_string()
        .clone();

    let (mut write, mut read) = ws_stream.split();

    // Send messages to the server
    let write_handle = tokio::spawn(async move {
        loop {
            let mut input = String::new();
            let _ = io::stdout().flush();
            io::stdin().read_line(&mut input).unwrap();

            let message = format!("{}: {}", username, input.trim_end());
            write.send(Message::Text(message)).await.unwrap();
        }
    });

    // Receive messages from the server
    let read_handle = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => println!("{}", text),
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                Ok(_) => {}
                Err(e) => eprintln!("error reading message: {}", e),
            }
        }
    });

    tokio::try_join!(read_handle, write_handle).unwrap();
}
