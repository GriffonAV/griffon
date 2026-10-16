{
  description = "Griffon";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = (import nixpkgs) {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        darwinBuildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
        ];

        linuxBuildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
          pkgs.pkg-config
          pkgs.glib
          pkgs.gtk3
          pkgs.webkitgtk_4_1
          pkgs.libsoup_3
          pkgs.libayatana-appindicator
          pkgs.librsvg
          pkgs.openssl
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            # Rust binaries come from rust-toolchain.toml.
            rustToolchain
            pkgs.rustup

            pkgs.just
            pkgs.pandoc
            pkgs.convco
            pkgs.zip
            pkgs.reuse

            # For releases
            pkgs.b3sum
            pkgs.cargo-bump

            # For generating demo
            pkgs.vhs

            pkgs.cargo-hack
            pkgs.cargo-udeps
            pkgs.cargo-outdated
          ]
          ++ darwinBuildInputs
          ++ linuxBuildInputs;
        };
      }
    );
}
