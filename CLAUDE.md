# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust CLI tool that generates TradingView watchlist files by fetching ticker data from various sources (cryptocurrency exchanges, Interactive Brokers positions, and ETF holdings). Each command outputs a `.txt` file to the `dist/` directory formatted for import into TradingView watchlists.

## Development Commands

This project uses `just` as a task runner. Key commands:

```bash
just run <command>           # Run the CLI with arguments (e.g., just run binance)
just watch                   # Auto-reload on changes with clippy checks
just lint                    # Run clippy with pedantic warnings
just test                    # Run unit tests (excludes integration tests)
just test-integration        # Run integration tests with output
just watch-integration       # Auto-run integration tests on changes
```

### Running the CLI

```bash
cargo run -- woo --perp --spot    # Fetch WOO perpetual and spot markets
cargo run -- binance              # Fetch Binance spot markets
cargo run -- kucoin               # Fetch KuCoin spot markets
cargo run -- ibkr                 # Fetch Interactive Brokers positions (requires TWS running on 127.0.0.1:7496)
cargo run -- stock-analysis <ETF> # Scrape ETF holdings from stockanalysis.com (e.g., spy, qqq)
```

## Architecture

The codebase follows a modular exchange provider pattern:

- `src/main.rs`: CLI entrypoint using `clap` with subcommands for each data source
- `src/exchanges/`: Each exchange module implements a data fetching function that returns `Vec<String>` of ticker symbols
  - `binance.rs`, `kucoin.rs`, `woo.rs`: Public API calls to crypto exchanges
  - `ibkr.rs`: Interactive Brokers API integration via `ibapi` crate, returns structured `Tickers` with stocks and options
  - `stockanalysis.rs`: Web scraping ETF holdings using `scraper`
- `src/utils.rs`: File writing utilities that output to `dist/<name>.txt`

### Exchange-Specific Details

**Crypto exchanges** filter for USDT-quoted pairs and exclude stablecoins. Output format: `EXCHANGE:SYMBOL` (e.g., `BINANCE:BTCUSDT`)

**IBKR** requires TWS (Trader Workstation) running locally on port 7496. It maps exchange codes (SMART→NYSE, ISLAND→NASDAQ, etc.) and filters out zero positions. Outputs separate files for stocks and options.

**StockAnalysis** scrapes ETF holdings pages and returns raw ticker symbols without exchange prefix.

### Output Files

All output written to `dist/` directory as TradingView-compatible watchlist files with naming convention:
- Crypto: `- C - <EXCHANGE>-<TYPE>.txt`
- Stocks: `- Positions - Stocks.txt`
- Options: `- Positions - Options.txt`
- ETFs: `- E - <ETF>.txt`

Each file contains one ticker per line in the format `EXCHANGE:TICKER` (e.g., `BINANCE:BTCUSDT` or `NYSE:AAPL`).

## Testing Strategy

The project uses fixture-based testing with a separate fixture generation tool.

### Tests
All tests use fixtures from `tests/fixtures/`:
- Test business logic with real API response data
- Test filtering, parsing, and formatting logic
- Fast, offline, deterministic
- Run with: `just test`

### Fixture Management

**Fixtures are separate from tests:**

Fixtures stored in `tests/fixtures/`:
- `binance_response.json` - Binance API response
- `kucoin_response.json` - KuCoin API response
- `woo_response.json` - WOO API response
- `stockanalysis_spy.html` - StockAnalysis HTML page

**Generating Fixtures:**
```bash
just generate-fixtures   # Runs examples/generate_fixtures.rs
```
This fetches fresh data from all APIs and saves to `tests/fixtures/`.

**Running Tests:**
```bash
just test               # Runs all tests with cached fixtures
cargo test              # Same as above
cargo test <test_name>  # Run specific test
```

### Test Organization

Each exchange module (`src/exchanges/*.rs`) follows this pattern:
- **Data fetching functions** - Make HTTP/API calls (private)
- **Processing functions** - Pure functions that transform data (tested)
- **Public API functions** - Combine fetching + processing (used in production)

Tests load fixtures and validate the full pipeline matches production usage.

### CI/CD Workflow

The GitHub Actions workflow:
1. Generates fresh fixtures from live APIs (`just generate-fixtures`)
2. Runs all tests with the fresh data (`just test`)
3. Fails if APIs have changed in breaking ways

**Workflow:**
- **Local development**: Fast iteration with cached fixtures (`just test`)
- **CI/CD**: Validates against fresh data on every commit
- **When CI fails**: API/website structure changed → update your parsing code

## Rust Toolchain

Project uses Rust 1.82.0 (specified in `rust-toolchain` file).
