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

### Environment Variables
Create a new `.env` file and inform the values:
```
DISCORD_TOKEN=
```

### Installing Dependencies
Open your terminal and type:
```sh
cargo install --locked
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
