set dotenv-load

clear:
  clear

tmux:
  tmuxinator local

run +ARGS="help": clear
  cargo run {{ARGS}}

watch: clear
  cargo watch --quiet --clear --exec "clippy -- -W clippy::pedantic && cargo run --quiet"

lint:
  cargo clippy -- -W clippy::pedantic

test: clear
  cargo nextest run

generate-fixtures: clear
  cargo run --example generate_fixtures
