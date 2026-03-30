{
  inputs = {
         nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
           utils.url = "github:numtide/flake-utils";
  };
  outputs =
    { nixpkgs, utils, rust-overlay, ... }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        wasm-server-runner = pkgs.rustPlatform.buildRustPackage {
          pname = "wasm-server-runner";
          version = "1.0.1";
          src = pkgs.fetchCrate {
            pname = "wasm-server-runner";
            version = "1.0.1";
            hash = "sha256-3DrbhmlKRUm2qj8wyQl5wBG2dbd7RUPXm/hPNt6txnk=";
          };
          cargoHash = "sha256-CBIqRIdYNFg1SP6Km4ypO0NhJGkQuxZrD1zOcRhUDdk=";
        };
        wasm-bindgen-cli = pkgs.rustPlatform.buildRustPackage {
          pname = "wasm-bindgen-cli";
          version = "0.2.114";
          src = pkgs.fetchCrate {
            pname = "wasm-bindgen-cli";
            version = "0.2.114";
            hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
          };
          cargoHash = "sha256-Z8+dUXPQq7S+Q7DWNr2Y9d8GMuEdSnq00quUR0wDNPM=";
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            wasm-server-runner
            wasm-bindgen-cli
            pkgs.pkg-config
            pkgs.wayland
            pkgs.alsa-lib
            pkgs.libudev-zero
            pkgs.vulkan-loader
            pkgs.libxkbcommon
            pkgs.binaryen
            (pkgs.rust-bin.selectLatestNightlyWith ( toolchain:
              toolchain.default.override {
                extensions = [ "rust-src" "rust-analyzer" ];
                targets = [ "wasm32-unknown-unknown" ];
              }
            ))
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.vulkan-loader
            pkgs.libxkbcommon
          ];
          shellHook = "exec ${pkgs.fish}/bin/fish";
        };
      }
    );
}
