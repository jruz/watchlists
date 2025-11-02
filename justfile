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

  # Find all allow(clippy::*) attributes
  ALLOWS=$(nix develop --command rg --type rust -o '#\[allow\(clippy::[a-z_]+\)\]' src/ || true)

  if [ -z "$ALLOWS" ]; then
    echo "✅ No clippy allow attributes found"
    exit 0
  fi

  # Check if any are not expect_used
  NON_EXPECT=$(echo "$ALLOWS" | grep -v 'expect_used' || true)
  if [ -n "$NON_EXPECT" ]; then
    echo ""
    echo "❌ Error: Found non-test clippy allows in source code:"
    echo "$NON_EXPECT"
    echo ""
    echo "Only #[allow(clippy::expect_used)] is permitted in test modules"
    exit 1
  fi

  # Check that all expect_used allows are in test modules
  # Look for lines where #[allow(clippy::expect_used)] is NOT preceded by #[cfg(test)]
  # We do this by checking if the context (-B 1) includes cfg(test)
  while IFS= read -r file; do
    CONTEXT=$(nix develop --command rg --type rust -B 1 '#\[allow\(clippy::expect_used\)\]' "$file" 2>/dev/null || true)
    if ! echo "$CONTEXT" | grep -q '#\[cfg(test)\]'; then
      echo ""
      echo "❌ Error: Found #[allow(clippy::expect_used)] not immediately after #[cfg(test)] in $file"
      exit 1
    fi
  done < <(nix develop --command rg --type rust -l '#\[allow\(clippy::expect_used\)\]' src/ || true)

  echo "✅ Only test-module expect_used allows found (permitted)"

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
  nix develop --command just lint-allows
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
