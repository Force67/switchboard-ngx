{
  description = "Switchboard NGX workspace (web + backend)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells = {
          default = pkgs.mkShell {
            packages = [
              pkgs.bun
              pkgs.nodejs_20
              pkgs.rustc
              pkgs.cargo
              pkgs.rustfmt
              pkgs.clippy
              pkgs.pkg-config
              pkgs.openssl
            ];
            shellHook = ''
              echo "Loaded Switchboard workspace shell (web + backend)"
            '';
          };

          web = pkgs.mkShell {
            packages = [
              pkgs.bun
              pkgs.nodejs_20
              pkgs.typescript
              pkgs.watchexec
            ];
            shellHook = ''
              echo "Loaded Switchboard web shell"
            '';
          };

          backend = pkgs.mkShell {
            packages = [
              pkgs.rustc
              pkgs.cargo
              pkgs.rustfmt
              pkgs.clippy
              pkgs.pkg-config
              pkgs.openssl
            ];
            shellHook = ''
              export RUST_LOG="info"
              echo "Loaded Switchboard backend shell"
            '';
          };
        };
      }
    );
}
