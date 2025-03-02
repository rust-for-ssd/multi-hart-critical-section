{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:

      {
        devShells.default = let
          pkgsCross =
            nixpkgs.legacyPackages.${system}.pkgsCross.riscv64-embedded;
          rust-bin = rust-overlay.lib.mkRustBin { } pkgsCross.buildPackages;
        in pkgsCross.callPackage
        ({ mkShell, pkg-config, qemu, rust-analyzer, stdenv }:
          mkShell {
            nativeBuildInputs = [ rust-bin.nightly.latest.default pkg-config ];

            depsBuildBuild = [ qemu rust-analyzer pkgsCross.buildPackages.gdb ];

            env = {
              # Makes rust-gdb use the riscv64-none-elf-gdb binary
              RUST_GDB =
                "${pkgsCross.buildPackages.gdb}/bin/riscv64-none-elf-gdb";
            };

          }) { };
      });
}

