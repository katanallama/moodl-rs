{
  lib,
  rustPlatform,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  name = "moodl-rs";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  buildInputs = [openssl];

  nativeBuildInputs = [pkg-config];
}
