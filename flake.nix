{
  description = "Status program for running tests, check the status of different test executables you write";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }: flake-utils.lib.eachDefaultSystem (system: let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ rust-overlay.overlays.default ];
    };
    rustPlatform = pkgs.rustPlatform;
    cargo = pkgs.cargo;
  in {
    packages.status = rustPlatform.buildRustPackage rec {
      name = "status";
      src = ./.;
      cargoSha256 = "sha256-owFG9Il289NqXNaV45CbZrKcqQl2py6dLKzX+d+j1wo=";
      buildInputs = [ cargo ];
    };
    defaultPackage = self.packages.${system}.status;
  });
}
