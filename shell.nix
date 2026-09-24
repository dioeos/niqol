{ pkgs ? import <nixpkgs> {}, niqol-pkg }:

pkgs.mkShell {
  inputsFrom = [
    niqol-pkg
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.wayland
    pkgs.libxkbcommon
  ];

  packages = with pkgs; [
    rustfmt
    clippy
    rust-analyzer

    nixd
    nixfmt

    slint-lsp

    python3
    pyright
  ];

  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  shellHook = ''
    echo "Entered Niqol shell..."
  '';
}

