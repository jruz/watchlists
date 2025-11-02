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

lint-allows:
  #!/usr/bin/env bash
  echo "Checking for allow(clippy) attributes in source code..."
  if rg --type rust '#\[allow\(clippy' src/; then
    echo ""
    echo "❌ Error: Found allow(clippy) attributes in source code"
    echo "Please refactor the code to avoid using clippy allows"
    exit 1
  else
    echo "✅ No allow(clippy) attributes found"
  fi

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
  just lint-allows
  nix develop --command cargo clippy --all-targets --all-features -- -D warnings
  nix develop --command cargo nextest run

outdated: clear
  nix develop --command cargo upgrade --dry-run

update: clear
  nix develop --command cargo upgrade
  just test

upgrade: clear
  nix develop --command cargo upgrade --incompatible
  just test
