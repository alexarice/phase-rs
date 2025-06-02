{
  description = "Simple rust enviroment";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
          {
            devShells.default = with pkgs; mkShell {
              buildInputs = [
                rust-bin.stable.latest.default
              ];
            };
          }
    );
}
