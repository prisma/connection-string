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
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        packages.test = pkgs.writeShellScriptBin "test" ''
          export RUSTFLAGS="-Dwarnings"
          export RUST_BACKTRACE=1

          ${rust}/bin/cargo test
        '';
        devShells.default = pkgs.mkShell {
          packages = [ rust ];
        };
      });
}
