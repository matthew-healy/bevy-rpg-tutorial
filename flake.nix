{
  description = "Bevy RPG tutorial by Logic Projects on YouTube";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, pre-commit-hooks, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        naersk' = pkgs.callPackage naersk { };

        mkGame = args:
          let
            defaultArgs = {
              src = ./.;

              cargoTestCommands = prev: prev ++ [
                ''cargo $cargo_options clippy -- -Dwarnings''
              ];

              override = prev:
                prev // {
                  nativeBuildInputs = prev.nativeBuildInputs ++ [ pkgs.clippy ];
                };
            };
          in
          naersk'.buildPackage (defaultArgs // args);

        preCommitHook = pre-commit-hooks.lib.${system}.run {
          src = self;
          hooks = {
            nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
          };
        };
      in
      rec {
        packages = rec {
          game = mkGame { };

          default = game;
        };

        apps.default = {
          type = "app";
          program = "${packages.bevyTutorial}/bin/bevy-rpg-tutorial";
        };

        devShells.default = pkgs.mkShell rec {
          shellHook = ''
            ${preCommitHook.shellHook}
          '';

          nativeBuildInputs = with pkgs; [
            rustc
            rustfmt
            cargo
            clippy
            rust-analyzer
            nixpkgs-fmt
            pkg-config
          ];

          buildInputs = with pkgs; [
            alsa-lib
            systemd
            libxkbcommon
            udev
            vulkan-loader
            wayland
          ];

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
        };

        formatter = pkgs.nixpkgs-fmt;

        checks = {
          game = mkGame { doCheck = true; };

          pre-commit = preCommitHook;
        };
      }
    );
}
