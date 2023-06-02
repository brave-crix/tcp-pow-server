## Problem
Design and implement “Word of Wisdom” tcp server.

- TCP server should be protected from DDOS attacks with the [Proof of Work](https://en.wikipedia.org/wiki/Proof_of_work), the challenge-response protocol should be used.
- The choice of the POW algorithm should be explained.
- After Proof Of Work verification, the server should send one of the quotes from “Word of wisdom” book or any other collection of the quotes.
- Docker file should be provided both for the server and for the client that solves the POW challenge

## Hashcash
I used Hashcash as a proof of work algorithm
[Hashcash](https://en.wikipedia.org/wiki/Hashcash)

I think Hashcash is more simple to implement than others like - `Merkle Tree`

It uses `version:zero_count:timestamp:resource::rand:counter` as an input of hash

Hashcash is validated when the `leading zero count` is enough.
I used `SHA256` to create hash.
## Challenge-response
Following `Challenge-response` protocol, I defined the message type like below
1. RequestService - Ask server to send initial hashcash,
2. Challenge - Send initial hashcash to client,
3. Response - Send valid hashcash to server,
4. GrantService - Send word of wisdom to client,
5. InvalidHashcash -  Reject the Challenge,
POW automatically 
##  How to build and run without docker
```
cargo test
cargo build --release
```
And then run server first
```
./target/release/server
```
You can add zero count by running, ZERO_COUNT determines the complexity in the client machine
```
ZERO_COUNT=4 ./target/release/server
```
Execute this command to run clients
```
./target/release/client
```
You can add max tries 
```
MAX_TRIES=100000 ./target/release/client
```
##  Docker Image
- I've created only one image named as `tcp_pow_server` for both of `server` and `client`.
- The `server` and `client` will use the same image but different bash command.
- The `server` service automatically start right after the image is built and it restarts when it's faild to run.
- The client service doesn't start automatically and tit can be run manually
- The `server` will bind any incoming ip addresses and he client will connect to the server address directly.
- As env variables, we can set `ZERO_COUNT` for `server` and `MAX_TRIES` for `client`
##  How to run with docker

```
sudo docker-compose up -d
sudo docker-compose run client
```
To change the server environment, you should run
```
sudo ZERO_COUNT=3 docker-compose up -d
```
To add the client variable, you should run
```
sudo MAX_TRIES=100000 docker-compose run client
```




