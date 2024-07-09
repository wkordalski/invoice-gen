{
  description = "Invoice generator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        nativeBuildInputs = with pkgs; [
          
        ];

        buildInputs = with pkgs; [];

        rpath = pkgs.lib.makeLibraryPath buildInputs;
      in
      rec {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          shellHook = ''
            LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${rpath}
            export RUST_BACKTRACE=1
          '';
        };
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "invoice-gen";
          version = "0.1";
          cargoLock.lockFile = ./Cargo.lock;
          src = pkgs.lib.cleanSource ./.;
        };
      }
    )
}