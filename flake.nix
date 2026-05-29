{
  description = "Yasl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-bin-custom = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.pkg-config
            rust-bin-custom
          ];
        };

        packages.clox = pkgs.stdenv.mkDerivation {
          pname = "clox";
          version = "0.1";
          src = ./clox;
          buildInputs = [ pkgs.gcc ];

          buildPhase = ''
            cSources=$(find . -maxdepth 1 -name '*.c' -print);
            echo "C source files found:"
            echo $cSources
            $CC $cSources -o clox
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp clox $out/bin/
          '';

          meta = with pkgs.lib; {
            description = "C project clox";
            license = licenses.mit;
            maintainers = [ "Lucas" ];
          };
        };

        packages.clox-tests = pkgs.stdenv.mkDerivation {
          pname = "clox-tests";
          version = "0.1";
          src = ./clox;

          buildInputs = [ pkgs.gcc ];

          buildPhase = ''
            testSources=$(find tests -name '*.c')
            srcSources=$(find . -maxdepth 1 -name '*.c' ! -name 'main.c')
            echo "Compiling tests..."
            $CC -I. $srcSources $testSources -o clox-tests
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp clox-tests $out/bin/
          '';

          doCheck = true;
          checkPhase = ''
            ./clox-tests
          '';

          meta = with pkgs.lib; {
            description = "Clox tests";
            license = licenses.mit;
          };
        };
      }
    );
}
