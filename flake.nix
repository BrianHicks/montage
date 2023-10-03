{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nmattia/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import inputs.nixpkgs { inherit system; };
        naersk-lib = inputs.naersk.lib."${system}";
      in
      rec {
        formatter = pkgs.nixpkgs-fmt;

        # `nix build`
        packages.montage = naersk-lib.buildPackage {
          root = ./.;
          buildInputs = [ pkgs.libiconv pkgs.rustPackages.clippy ];
        };
        defaultPackage = packages.montage;
        overlay = final: prev: { montage = packages.montage; };

        devShell =
          pkgs.mkShell {
            packages = [
              # for the program itself
              pkgs.cargo
              pkgs.clippy
              pkgs.rustc
              pkgs.rustfmt
              pkgs.libiconv
              pkgs.rust-analyzer
              pkgs.sqlx-cli
              pkgs.cargo-insta
              pkgs.cargo-machete
              pkgs.cargo-outdated
              pkgs.cargo-edit

              # for the OmniFocus scripts
              pkgs.nodePackages.npm
              pkgs.nodePackages.prettier
              pkgs.nodePackages.typescript
              pkgs.nodePackages.typescript-language-server

              # misc stuff
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];
          };
      }
    );
}
