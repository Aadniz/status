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

     nixosModules.status = { config, pkgs, lib, ... }: {
       options.services.status = {
         enable = lib.mkEnableOption "status service";

         package = lib.mkOption {
           type = lib.types.package;
           default = self.defaultPackage.${system};
           description = "The status package to use.";
         };

         settingsPath = lib.mkOption {
           type = lib.types.path;
           description = "The path to the settings.json file.";
         };

         restartSec = lib.mkOption {
           type = lib.types.int;
           default = 120;
           description = "The number of seconds to wait before restarting the service.";
         };
       };
       config = lib.mkIf config.services.status.enable {
         systemd.services.status = {
           description = "Checking all executables, test application written in rust";
           after = [ "network-online.target" ];
           wantedBy = [ "multi-user.target" ];
           serviceConfig = {
             Type = "simple";
             ExecStart = "${config.services.status.package}/bin/status ${config.services.status.settingsPath}";
             Restart = "always";
             RestartSec = toString config.services.status.restartSec;
           };
         };
       };
     };
  });
}
