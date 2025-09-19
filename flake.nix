{
  description = "trein – select area -> OCR -> DeepL translate for Wayland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (
      system: let
        pkgs = import nixpkgs {inherit system;};

        muslTarget =
          {
            x86_64-linux = "x86_64-unknown-linux-musl";
            aarch64-linux = "aarch64-unknown-linux-musl";
          }.${
            system
          };
      in rec {
        packages.trein = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
          pname = "trein";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          # Ensure cargo targets musl; some crates require explicit crt-static.
          CARGO_BUILD_TARGET = muslTarget;
          RUSTFLAGS = "-C target-feature=+crt-static";

          cargoBuildFlags = ["--locked"];
          doCheck = false;
        };

        packages.default = packages.trein;

        apps.default = {
          type = "app";
          program = "${packages.trein}/bin/trein";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            pkg-config
            mold
            grim
            slurp
            tesseract
            wl-clipboard
          ];
          shellHook = ''
            export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };

        checks.build = packages.trein;
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
