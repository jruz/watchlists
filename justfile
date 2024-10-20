set dotenv-load

clear:
  clear

tmux:
  tmuxinator local

run +ARGS="help": clear
  cargo run -q {{ARGS}}

watch: clear
  cargo watch --quiet --clear --exec "clippy -- -W clippy::pedantic && cargo run --quiet"

lint:
  cargo clippy -- -W clippy::pedantic

test: clear
  cargo nextest run -E 'not test(integration)'

test-integration: clear
  cargo nextest run -E 'test(integration)' --nocapture

watch-integration:
  cargo watch -x nextest run -E 'test(integration)'
