{
  description = "Niqol flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    niqol = pkgs.rustPlatform.buildRustPackage {
      pname = "niqol";
      version = "0.1.0";

      src = ./.;

      cargoLock.lockFile = ./Cargo.lock;
    };
  in
  {
    devShells.${system}.default =
      import ./shell.nix { inherit pkgs; };

    packages.${system} = {
      default = niqol;
    };

    homeManagerModules.default =
      import ./niqol-module.nix;
  };
}

