{
  description = "frosted: freezed light";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    # Upstream grammar (has Dart 3.13 primary constructor support). Taken as a
    # plain source tarball: this skips its `tree-sitter` git submodule, whose
    # scp-style URL cargo can't fetch.
    tree-sitter-dart = {
      url = "github:UserNobody14/tree-sitter-dart";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, tree-sitter-dart }: let
      inherit (nixpkgs) lib;
      systems = lib.systems.flakeExposed;
      forAllSystems = lib.genAttrs systems;
      spkgs = system: nixpkgs.legacyPackages.${system}.pkgs;

      # Upstream's Rust binding still pins tree-sitter 0.22; frosted uses 0.25.
      # Cargo.toml points at ./vendor/tree-sitter-dart, which is filled from here.
      treeSitterDartSrc = system: (spkgs system).runCommand "tree-sitter-dart-src" { } ''
        cp -r ${tree-sitter-dart} $out
        chmod -R u+w $out
        substituteInPlace $out/Cargo.toml \
          --replace-fail 'tree-sitter = "0.22.6"' 'tree-sitter = "0.25"'
      '';
    in {
      packages = forAllSystems (s: with spkgs s; rec {
        parser = rustPlatform.buildRustPackage {
          pname = "frosted";
          version = "0.0.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          postUnpack = ''
            mkdir -p $sourceRoot/vendor
            cp -r ${treeSitterDartSrc s} $sourceRoot/vendor/tree-sitter-dart
            chmod -R u+w $sourceRoot/vendor
          '';
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
          shellHook = ''
            mkdir -p vendor
            ln -sfn ${treeSitterDartSrc s} vendor/tree-sitter-dart
          '';
          buildInputs = [
            cargo
            rustc
          ];
        };
      });
  };
}
