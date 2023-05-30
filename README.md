# rsa_rt_messager
## Client-Side Protocol Description
### Step 1: Connection Request

The client initiates the communication by sending a connection request to the server. This can be a simple message indicating that the client wishes to establish a secure communication channel.

```
Client -> Server: "Connection Request"
```

Step 2: Receive Server's RSA Public Key

Upon receiving the connection request, the server will respond by sending its RSA public key. The client needs to receive this key and store it for the next step.

markdown

Server -> Client: "RSA Public Key"

Step 3: Generate and Send AES Key

After receiving the server's RSA public key, the client will generate a random AES key for symmetric encryption. The client will then encrypt this AES key using the server's RSA public key. This encrypted AES key is then sent back to the server.

markdown

Client -> Server: "AES Key (encrypted with Server's RSA public key)"

Step 4: Wait for Acknowledgment

The client then waits for an acknowledgment from the server that the AES key has been successfully decrypted and is ready for use in symmetric encryption.

markdown

Server -> Client: "AES Key Decryption Acknowledgment"

Step 5: Communication Using AES Encryption

Once the acknowledgment is received, the client can start sending messages encrypted with the shared AES key. Similarly, it can decrypt received messages with the same AES key. This process continues as long as the secure communication session is active.

markdown

Client -> Server: "AES Encrypted Message"
Server -> Client: "AES Encrypted Message"

Note: The client should be prepared to handle any errors during this process, such as failure to connect to the server, failure to receive the server's RSA public key, failure to generate or encrypt the AES key, or failure to receive the acknowledgment from the server. Proper error handling mechanisms should be in place to ensure secure and reliable communication.
