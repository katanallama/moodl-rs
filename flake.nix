{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = {
    nixpkgs,
    rust-overlay,
    ...
  }: let
    system = "x86_64-linux";

    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };

    crossPkgs = import nixpkgs {
      inherit system;
      crossSystem = {
        config = "x86_64-w64-mingw32";
      };
    };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        toolchain
        pkgs.beekeeper-studio
        pkgs.sqlite
        pkgs.openssl
        pkgs.pkg-config
        pkgs.rust-analyzer-unwrapped
      ];

      RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
    };

    packages.${system}.moodl-rs = pkgs.callPackage ./moodl-rs.nix {};
    packages."x86_64-w64-mingw32".moodl-rs = crossPkgs.callPackage ./moodl-rs.nix {};
  };
}
