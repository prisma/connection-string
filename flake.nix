{
  description = "ADO.net and JDBC Connection String Parser.";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      nix = pkgs.nixFlakes;
      inherit (pkgs) wasm-bindgen-cli rustPlatform nodejs;
    in {
      defaultPackage = rustPlatform.buildRustPackage {
        name = "connection-string-wasm";
        src = builtins.path { path = ./connection-string-wasm; name = "connection-string-wasm"; };

        cargoLock = {
          lockFile = ./connection-string-wasm/Cargo.lock;
        };

        nativeBuildInputs = [ rust wasm-bindgen-cli ];

        buildPhase = ''
          export RUSTFLAGS="-Dwarnings"
          export RUST_BACKTRACE=1

          cargo build --release --target=wasm32-unknown-unknown
          echo 'Creating out dir...'
          mkdir -p $out/src;
          echo 'Copying package.json...'
          cp ./package.json $out/;
          echo 'Copying README.md...'
          cp README.md $out/;
          echo 'Generating node module...'
          wasm-bindgen \
            --target nodejs \
            --out-dir $out/src \
            target/wasm32-unknown-unknown/release/connection_string_wasm.wasm;
        '';
        checkPhase = "echo 'Check phase: skipped'";
        installPhase = "echo 'Install phase: skipped'";
      };

      packages = {
        cargo = {
          type = "app";
          program = "${rust}/bin/cargo";
        };

        # Takes the new package version as first and only argument, and updates package.json
        updatePackageVersion = pkgs.writeShellScriptBin "updateNpmPackageVersion" ''
          ${pkgs.jq}/bin/jq ".version = \"$1\"" package.json > /tmp/package.json
          rm package.json
          cp /tmp/package.json package.json
          sed -i "s/^version\ =.*$/version = \"$1\"/" Cargo.toml
        '';
        test = pkgs.writeShellScriptBin "test" ''
          export RUSTFLAGS="-Dwarnings"
          export RUST_BACKTRACE=1

          ${rust}/bin/cargo test
        '';
        publishRust = pkgs.writeShellScriptBin "publishRust" ''
          ${rust}/bin/cargo publish
        '';
        publishJavascript = pkgs.writeShellScriptBin "publishRust" ''
          ${nix}/bin/nix build
          ${nodejs}/bin/npm publish ./result --access public --tag latest
        '';
        publish = pkgs.writeShellScriptBin "publish" ''
          ${nix}/bin/nix publishRust
          ${nix}/bin/nix publishJavascript
        '';
        npm = {
          type = "app";
          program = "${nodejs}/bin/npm";
        };
        wasm-bindgen = {
          type = "app";
          program = "${wasm-bindgen-cli}/bin/wasm-bindgen";
        };
        syncWasmBindgenVersions = pkgs.writeShellScriptBin "updateWasmBindgenVersion" ''
          echo 'Syncing wasm-bindgen version in crate with that of the installed CLI...'
          sed -i "s/^wasm-bindgen\ =.*$/wasm-bindgen = \"=${wasm-bindgen-cli.version}\"/" Cargo.toml
        '';
      };
      devShell = pkgs.mkShell {
        nativeBuildInputs = [ pkgs.bashInteractive ];
        buildInputs = [
          rust
          pkgs.nodejs
          pkgs.wasm-bindgen-cli
        ];
      };
    });
}
