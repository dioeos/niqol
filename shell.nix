{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer

    nixd
    nixfmt
  ];

  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  shellHook = ''
    echo "Entered Niqol shell..."
  '';
}

