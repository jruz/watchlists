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
  nix develop --command cargo clippy -- -W clippy::pedantic

test: clear
  #!/usr/bin/env bash
  if command -v nix &> /dev/null; then
    nix develop --command cargo nextest run
  else
    cargo nextest run
  fi

generate-fixtures: clear
  #!/usr/bin/env bash
  if command -v nix &> /dev/null; then
    nix develop --command cargo run --example generate_fixtures
  else
    cargo run --example generate_fixtures
  fi

outdated: clear
  nix develop --command cargo upgrade --dry-run

update: clear
  nix develop --command cargo upgrade
  just test

upgrade: clear
  nix develop --command cargo upgrade --incompatible
  just test
