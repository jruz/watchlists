{
  description = "Watchlist - TradingView watchlist generator with Playwright support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable."1.91.0".default;

        playwright-browsers = pkgs.playwright-driver.browsers;

        chromiumExecutable = if pkgs.stdenv.isDarwin
          then "${playwright-browsers}/chromium-*/chrome-mac/Chromium.app/Contents/MacOS/Chromium"
          else "${playwright-browsers}/chromium-*/chrome-linux/chrome";
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo-edit
            cargo-nextest
            just
            playwright-driver.browsers
            nodejs
            openssl
            pkg-config
          ];

          shellHook = ''
            export PLAYWRIGHT_BROWSERS_PATH="${playwright-browsers}"
            export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
            export PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS=true
            export PLAYWRIGHT_NODEJS_PATH="${pkgs.nodejs}/bin/node"
            export PLAYWRIGHT_CHROMIUM_EXECUTABLE="$(echo ${chromiumExecutable})"
          '';
        };
      }
    );
}
