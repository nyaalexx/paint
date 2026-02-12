{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    utils.url = "github:numtide/flake-utils";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs:
    inputs.utils.lib.eachDefaultSystem (system:
      let
        pkgs = import inputs.nixpkgs { inherit system; };

        rust-toolchain = with inputs.fenix.packages.${system};
          combine (with complete; [
            rustc
            rust-src
            cargo
            clippy
            rustfmt
            rust-analyzer
            miri
            targets.aarch64-linux-android.latest.rust-std
          ]);
      in {
          devShell = (pkgs.mkShell.override { stdenv = pkgs.clangStdenv; }) rec {
          buildInputs = with pkgs; [
            cargo-nextest
            rust-toolchain
            python3Minimal
            wgsl-analyzer
            clangStdenv
            pkgsCross.aarch64-android-prebuilt.stdenv.cc
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = 1;
          CC_aarch64_linux_android = "${pkgs.pkgsCross.aarch64-android-prebuilt.stdenv.cc}/bin/aarch64-unknown-linux-android-clang";
          CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "${pkgs.pkgsCross.aarch64-android-prebuilt.stdenv.cc}/bin/aarch64-unknown-linux-android-clang";
        };
      }
    );
}
