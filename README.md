# tuiper
A terminal user-interface game for practicing typing and racing against typists. 

## How to run
1. Install Rust
2. ``git clone https://github.com/GopeshSingal/tuiper.git``
3. You can either play as a guest online or have an account to track progress
    * Play as guest: ``cargo run --release -p tuiper``
    * Make account / sign in: ``cargo run --release -p tuiper -- --user <your_user> --password <your_password>`` First time logging in will create the account permanently. 

## Features
- Solo practicing mode featuring Time and Words mode
- Competitive real-time multiplayer between two players with an Elo system
- Customizable aesthetic configuration

## Feature TODO before first release
- [ ] Support multiple languages and use self-sourced english language
- [ ] Enhance aesthetic configuration with cursor options, more color control, etc.
- [ ] Beautify application
- [ ] Store user race data for analysis

## Inspirations
- ttyper by Max Niederman
- monkeytype by Jack Miodec
