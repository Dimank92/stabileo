{
  description = "Dedaliano — structural analysis engine + AI backend";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # The one toolchain, shared with rustup and every CI job (scripts/check-pins.sh).
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Rust
            rustToolchain
            pkgs.wasm-pack
            pkgs.cargo-watch
            pkgs.cargo-nextest
            # Compiled crates shared by every worktree (RUSTC_WRAPPER below).
            pkgs.sccache

            # Node / frontend: the major in .nvmrc, which CI reads.
            pkgs.nodejs_22

            # Backend runtime deps
            pkgs.pkg-config
            pkgs.openssl

            # Docker (optional — uses host docker)
            pkgs.docker-compose

            # Dev tools
            pkgs.just
            pkgs.jq
            pkgs.curl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.apple-sdk_15
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # engine/.cargo/config.toml links x86_64 Linux builds with clang + mold.
            pkgs.clang
            pkgs.mold
          ];

          RUSTC_WRAPPER = "sccache";

          shellHook = ''
            echo "🏗  Dedaliano dev shell ready"
            echo "   make dev        — start backend + web"
            echo "   make test       — run all tests"
            echo "   make docker-up  — start via docker compose"
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # Verification tooling: Lean (elan reads verify/lean-toolchain), Kani, SAT solvers.
        # Kani is not in nixpkgs: scripts/kani-setup.sh installs it into ./.tools, and it
        # keeps its own nightly toolchain and CBMC under ./.tools/kani (KANI_HOME). Do not add
        # nixpkgs' cbmc here: it shadows Kani's bundled CBMC and breaks goto-cc on macOS.
        devShells.verify = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.elan
            pkgs.kissat
            pkgs.cadical
            pkgs.git
            pkgs.curl
            pkgs.jq
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.apple-sdk_15
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # engine/.cargo/config.toml links x86_64 Linux builds with clang + mold.
            pkgs.clang
            pkgs.mold
          ];

          shellHook = ''
            root=$(git rev-parse --show-toplevel)
            export ELAN_HOME="$root/.tools/elan"
            export KANI_HOME="$root/.tools/kani"
            export PATH="$root/.tools/bin:$ELAN_HOME/bin:$PATH"
          '';
        };
      }
    );
}
