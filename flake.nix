{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {...} @ inputs: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSupportedSystem = f:
      inputs.nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.rust-overlay.overlays.default
              inputs.self.overlays.default
            ];
          };
        });
  in {
    overlays.default = final: prev: {
      rustToolchain = prev.rust-bin.stable.latest.default.override {
        extensions = ["clippy" "rust-src" "rustfmt"];
      };
    };

    packages = forEachSupportedSystem ({pkgs}: {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = "trexanh";
        version = "0.9.7";
        src = ./.;

        cargoHash = "sha256-zkPcjM0fZhN08gQXiKukQpFMI+figq5s8M2nSMzZ3kQ=";

        nativeBuildInputs = with pkgs; [rustToolchain removeReferencesTo];

        postInstall = ''
          remove-references-to -t ${pkgs.rustToolchain} $out/bin/trexanh
          strip $out/bin/*
        '';
      };
    });

    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        packages = with pkgs; [
          cargo-deny
          rust-analyzer
          rustToolchain
        ];

        env = {
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
        };
      };
    });
  };
}
