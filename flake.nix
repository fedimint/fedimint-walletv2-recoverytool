{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix?rev=6b5325a017a9a9fe7e6252ccac3680cc7181cd63";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flakebox = {
      url = "github:dpc/flakebox?rev=09d74b0ecac2214a57887f80f2730f2399418067";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      flakebox,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        lib = pkgs.lib;
        stdenv = pkgs.stdenv;

        flakeboxLib = flakebox.lib.mkLib pkgs {
          config = {
            github.ci.enable = false;
            toolchain.channel = "stable";
            toolchain.components = [
              "rustc"
              "cargo"
              "clippy"
              "rust-analyzer"
              "rust-src"
            ];
          };
        };

        stdTargets = flakeboxLib.mkStdTargets { };

        toolchainArgs = lib.optionalAttrs stdenv.isLinux {
          stdenv = p: p.llvmPackages_20.stdenv;
          clang = pkgs.llvmPackages_20.clang;
          libclang = pkgs.llvmPackages_20.libclang.lib;
          clang-unwrapped = pkgs.llvmPackages_20.clang-unwrapped;
        };

        toolchain = flakeboxLib.mkFenixToolchain (
          toolchainArgs
          // {
            targets = (
              pkgs.lib.getAttrs [
                "default"
                "wasm32-unknown"
              ] stdTargets
            );
          }
        );
      in
      {
        devShells.default = flakeboxLib.mkDevShell {
          toolchain = toolchain;
          nativeBuildInputs = with pkgs; [
            trunk
            wasm-bindgen-cli
          ];
        };
      }
    );
}
