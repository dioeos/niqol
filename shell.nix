{ pkgs ? import <nixpkgs> {}, niqol-pkg }:

pkgs.mkShell {
  inputsFrom = [
    niqol-pkg
  ];

  packages = with pkgs; [
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

