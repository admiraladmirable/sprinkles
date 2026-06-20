{
  description = "bevy flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    # bevy_cli.url = "github:TheBevyFlock/bevy_cli";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      # bevy_cli,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
          (final: prev: {
            rustToolchain =
              let
                rust = prev.rust-bin;
              in
              if builtins.pathExists ./rust-toolchain.toml then
                rust.fromRustupToolchainFile ./rust-toolchain.toml
              else if builtins.pathExists ./rust-toolchain then
                rust.fromRustupToolchainFile ./rust-toolchain
              else
                rust.stable.latest.default.override {
                  extensions = [
                    "rust-src"
                    "rustfmt"
                    "rust-analyzer"
                    "clippy"
                  ];
                };
          })
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            packages = [
              rustToolchain
              foundry
              pkg-config
              cargo-deny
              cargo-edit
              cargo-watch
              cargo-audit
              cargo-machete
              cargo-sort
              openssl
              # bevy_cli.packages.${system}.default
            ];
            buildInputs = [
              # Rust dependencies
              rustToolchain
              # (rust-bin.stable.latest.default.override { extensions = [ "rust-src" ]; })
              pkg-config
              openssl
            ]
            ++ lib.optionals (lib.strings.hasInfix "linux" system) [
              # for Linux
              # Audio (Linux only)
              alsa-lib
              # Cross Platform 3D Graphics API
              vulkan-loader
              # For debugging around vulkan
              vulkan-tools
              # Other dependencies
              libudev-zero
              libX11
              libXcursor
              libXi
              libXrandr
              libxkbcommon
              wayland
              wayland-utils
            ];
            # RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            LD_LIBRARY_PATH = lib.makeLibraryPath [
              vulkan-loader
              libX11
              libXi
              libXcursor
              libxkbcommon
            ];
            CLAUDE_CONFIG_DIR = "/home/rmrf/.claude-personal";
          };
      }
    );
}
