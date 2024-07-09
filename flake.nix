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


        tex = pkgs.texlive.combine {
          inherit (pkgs.texlive) scheme-minimal latex-bin tools amsmath polski geometry makecell siunitx;
        };

        nativeBuildInputs = with pkgs; [ ];
        buildInputs = with pkgs; [ tex ];

        rpath = pkgs.lib.makeLibraryPath buildInputs;

        package = pkgs.rustPlatform.buildRustPackage rec {
          pname = "invoice-gen";
          version = "0.1";
          cargoLock.lockFile = ./Cargo.lock;
          src = pkgs.lib.cleanSource ./.;
          PDFLATEX_PATH = "${tex}/bin/pdflatex";
        };
      in
      rec {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          shellHook = ''
            LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${rpath}
            export RUST_BACKTRACE=1
          '';
        };
        packages.default = package;
        overlays.default = (final: prev: {
          invoice-gen = package;
        });
      }
    );
}
