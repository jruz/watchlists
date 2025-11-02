set dotenv-load

clear:
  -clear

tmux:
  tmuxinator local

run +ARGS="help": clear
  nix develop --command cargo run {{ARGS}}

watch: clear
  nix develop --command cargo watch --quiet --clear --exec "clippy -- -W clippy::pedantic && cargo run --quiet"

lint:
  nix develop --command cargo clippy --all-targets --all-features -- -D warnings

fmt:
  nix develop --command cargo fmt --all -- --check

fmt-fix:
  nix develop --command cargo fmt --all

test: clear
  nix develop --command cargo nextest run

generate-fixtures: clear
  nix develop --command cargo run --bin generate_fixtures

check: clear
  nix develop --command cargo fmt --all -- --check
  nix develop --command cargo clippy --all-targets --all-features -- -D warnings
  nix develop --command cargo run --bin generate_fixtures
  nix develop --command cargo nextest run

outdated: clear
  nix develop --command cargo upgrade --dry-run

update: clear
  nix develop --command cargo upgrade
  just test

upgrade: clear
  nix develop --command cargo upgrade --incompatible
  just test
