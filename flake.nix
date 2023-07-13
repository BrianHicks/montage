{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import inputs.nixpkgs { inherit system; };
      in { 
        formatter = pkgs.nixpkgs-fmt;

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

              # for the OmniFocus scripts
              pkgs.nodePackages.npm
              pkgs.nodePackages.prettier
              pkgs.nodePackages.typescript
              pkgs.nodePackages.typescript-language-server

              # misc stuff
              pkgs.darwin.apple_sdk.frameworks.Security
            ];
          };
      }
    );
}
