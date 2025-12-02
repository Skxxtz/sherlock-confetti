{
  description = "Confetti packaged for Nix/NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs { inherit system; }));
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          lib = pkgs.lib;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "confetti";
            version = "unstable";
            src = ./.;
            cargoHash = "sha256-6lwp5gky+NKhE2IKVI2Qxqe5HT9a8xqWfrJ2e9CK6IE=";

            nativeBuildInputs = with pkgs; [
              pkg-config
              wayland-protocols
              makeWrapper
            ];

            buildInputs = with pkgs; [
              wayland
              libxkbcommon
              vulkan-loader
              mesa
            ];

            postInstall =
              let
                libPath = lib.makeLibraryPath [
                  pkgs.wayland
                  pkgs.libxkbcommon
                  pkgs.vulkan-loader
                  pkgs.mesa
                ];
              in
              ''
                wrapProgram "$out/bin/confetti" \
                  --prefix LD_LIBRARY_PATH : ${libPath}
              '';
          };

          confetti = self.packages.${pkgs.system}.default;
        }
      );

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.system}.default}/bin/confetti";
        };
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustc
            cargo
            wayland-protocols
          ];
          buildInputs = with pkgs; [
            wayland
            libxkbcommon
            vulkan-loader
            mesa
          ];
        };
      });
    };
}
