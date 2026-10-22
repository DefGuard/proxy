{
  description = "Rust development flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane.url = "github:ipetkov/crane";

    # include the proto and defguard-ui submodules in the flake source
    self.submodules = true;
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-analyzer" "rust-src" "rustfmt" "clippy"];
      };

      # nightly rustfmt, needed only for the unstable import-grouping options that
      # `fmt-imports` passes via --config.
      rustfmtNightly = pkgs.rust-bin.nightly.latest.rustfmt;

      # Usage: fmt-imports [cargo fmt flags]   e.g. fmt-imports --check
      fmtImports = pkgs.writeShellScriptBin "fmt-imports" ''
        set -euo pipefail
        root="$(${pkgs.git}/bin/git rev-parse --show-toplevel)"
        cd "$root"
        export RUSTFMT="${rustfmtNightly}/bin/rustfmt"
        exec ${rustToolchain}/bin/cargo fmt "$@" -- \
          --config imports_granularity=Crate,group_imports=StdExternalCrate
      '';

      craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);
      defguard-proxy = pkgs.callPackage ./nix/package.nix {
        inherit pkgs craneLib;
        gitRev = if self ? rev then self.rev else "unknown";
      };

      # define shared build inputs
      nativeBuildInputs = with pkgs; [rustToolchain pkg-config];
      buildInputs = with pkgs;
        [openssl protobuf nodejs_26 pnpm_11]
        ++ lib.optionals stdenv.hostPlatform.isLinux [systemd];
    in {
      packages = {
        default = defguard-proxy;
        inherit defguard-proxy;
      };

      checks.default = defguard-proxy;

      devShells.default = pkgs.mkShell {
        inherit nativeBuildInputs buildInputs;

        packages = with pkgs; [
          fmtImports
          # TS/JS LSP
          vtsls
          # protobuf formatter
          buf
          # image signarute verification
          cosign
          trivy
        ];

        # Specify the rust-src path (many editors rely on this)
        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
      };
    })
    // {
      nixosModules.default = import ./nix/nixos-module.nix {
        mkCraneLib = crane.mkLib;
      };
    };
}
