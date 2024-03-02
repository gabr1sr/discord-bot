# Discord Bot
A multi-purpose selfhosted Discord bot.

## Goals
- Selfhosted so you don't need to trust other bots
- Provide everything a Discord server might need
- Users can verify the source code
- Multi-language support
- Highly configurable

## Getting Started
Here we will teach you how to setup your bot.

### Requirements
- [Rust](https://www.rust-lang.org/) (if you are on NixOS, just run `direnv allow`)
- [PostgreSQL](https://www.postgresql.org/)

### Environment Variables
Copy the `.env.example` file to `.env`:
```
cp .env.example .env
```

Open `.env` file and fill these values:
```
DISCORD_TOKEN=
DATABASE_HOST=
DATABASE_PORT=
DATABASE_USER=
DATABASE_PASSWORD=
```

### Build and Run
To build the whole project:
```sh
cargo build
```

To run the project:
```sh
cargo run
```
