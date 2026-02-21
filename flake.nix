{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustPackages.clippy
              pnpm
              pkg-config
              openssl
              nodejs-slim
            ];
            nativeBuildInputs = [
              rustfmt
              just
              nushell
              sea-orm-cli
              typeshare
              biome
              vtsls
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
