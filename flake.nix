{
  description = "frosted: freezed light";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
      inherit (nixpkgs) lib;
      systems = lib.systems.flakeExposed;
      forAllSystems = lib.genAttrs systems;
      spkgs = system: nixpkgs.legacyPackages.${system}.pkgs;
    in {
      packages = forAllSystems (s: with spkgs s; rec {
        parser = rustPlatform.buildRustPackage {
          pname = "frosted";
          version = "0.0.1";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "tree-sitter-dart-0.0.1" = "sha256-6TJV4N0YKufvMJh81stpKGaxmFTzeFwoEz5t07yXS24=";
            };
          };
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            openssl
          ];
        };
        default = parser;
      });

      devShells = forAllSystems (s: with spkgs s; {
        default = mkShell {
          RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
          buildInputs = [
            cargo
            rustc
          ];
        };
      });
  };
}
