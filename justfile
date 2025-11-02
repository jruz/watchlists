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
  echo "Checking for new allow attributes in source code..."
  BASELINE=".allowed-lints-baseline.txt"
  CURRENT=$(mktemp)

  rg --type rust '#\[allow\(clippy' src/ --no-heading --no-filename 2>/dev/null | sort > "$CURRENT" || touch "$CURRENT"

  if [ ! -f "$BASELINE" ]; then
    echo "⚠️  Warning: No baseline file found at $BASELINE"
    echo "Creating baseline with current allows..."
    cp "$CURRENT" "$BASELINE"
    echo "✅ Baseline created"
    rm "$CURRENT"
    exit 0
  fi

  DIFF=$(diff "$BASELINE" "$CURRENT")

  if [ -n "$DIFF" ]; then
    echo "❌ Error: New allow(clippy) attributes detected!"
    echo ""
    echo "Differences from baseline:"
    diff "$BASELINE" "$CURRENT" || true
    echo ""
    echo "Please refactor the code to avoid using clippy allows"
    echo "If these allows are intentional and unavoidable, update the baseline:"
    echo "  rg --type rust '#\[allow\(clippy' src/ --no-heading --no-filename | sort > $BASELINE"
    rm "$CURRENT"
    exit 1
  else
    echo "✅ No new allow attributes detected"
  fi

  rm "$CURRENT"

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
