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
        extensions = ["rust-src" "rustfmt"];
      };
    };

    packages = forEachSupportedSystem ({pkgs}: {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = "trexanh";
        version = "0.9.1";
        src = ./.;

        cargoHash = "sha256-fhipxHvGllBuMSAgq46HIYoV0lZZpD0FhVO/p4qUJvs=";

        nativeBuildInputs = with pkgs; [perl rustToolchain];
      };
    });

    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        packages = with pkgs; [
          cargo-audit
          cargo-deny
          cargo-edit
          cargo-watch
          rust-analyzer
          rustToolchain

          git-cliff
        ];

        env = {
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
        };
      };
    });
  };
}
