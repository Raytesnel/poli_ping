# PoliPing
Sends push notifications about political motions. 
Vote with them and find your best match when a new voting is expected.


## Installation (localhost)
## Prerequisites

Make sure you have installed:

- Rust (https://www.rust-lang.org/tools/install)
- Cargo (comes with Rust)
- sqlx-cli:
  `cargo install sqlx-cli --no-default-features --features sqlite`
- Dioxus CLI:
  `cargo install dioxus-cli`
- Clone repo:
  `git clone https://github.com/Raytesnel/poli_ping.git`

## Quickstart
### setup
```shell
git clone https://github.com/Raytesnel/poli_ping.git
cd poli_ping

cp .env.example .env
# update LLM_KEY in .env

sqlx database create
sqlx migrate run

cargo run
```
### run backend
```shell
cargo run -p backend
```

### run frontend
```shell
cd frontend
dx serve --web
```

