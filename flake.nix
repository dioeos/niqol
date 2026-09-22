{
  description = "Niqol flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    niqol-pkg = pkgs.rustPlatform.buildRustPackage {
      pname = "niqol";
      version = "0.1.0";

      src = ./.;

      cargoLock.lockFile = ./Cargo.lock;

      nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
      buildInputs = [ pkgs.fontconfig ];

      postFixup = ''
        wrapProgram "$out/bin/niqol-ui" \
          --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [
            pkgs.wayland
            pkgs.libxkbcommon
          ]}"
      '';
    };
  in
  {
    devShells.${system}.default =
      import ./shell.nix { inherit pkgs niqol-pkg; };

    packages.${system} = {
      default = niqol-pkg;
    };

    homeManagerModules.default =
      import ./niqol-module.nix;
  };
}

