{
  description = "trein – select area -> OCR -> DeepL translate for Wayland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    naersk,
    ...
  }:
    flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (
      system: let
        pkgs = import nixpkgs {inherit system;};
        naerskLib = naersk.lib.${system};
        runtimeDeps = with pkgs; [grim slurp tesseract wl-clipboard];
      in rec {
        packages.trein = naerskLib.buildPackage {
          pname = "trein";
          version = "0.1.0";
          src = ./.;
          cargoBuildOptions = opts: opts ++ ["--locked"];
          nativeBuildInputs = [pkgs.makeWrapper];

          # Ensure the binary finds Wayland helpers at runtime.
          postInstall = ''
            wrapProgram "$out/bin/trein" \
              --prefix PATH : ${pkgs.lib.makeBinPath runtimeDeps}
          '';
        };

        packages.default = packages.trein;

        apps.default = {
          meta = {
            description = "Select area -> OCR -> DeepL translate for Wayland";
          };
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
            mold # fast linker (optional)
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
