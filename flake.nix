{
  description = "A reproducible development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };

          # Libraries Bevy needs at build time and loads at runtime (Linux only)
          bevyLibs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
            alsa-lib
            udev
            vulkan-loader
            libxkbcommon
            wayland
            libx11
            libxcursor
            libxi
            libxrandr
          ]);
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = bevyLibs;

            packages = with pkgs; [
              git
              ripgrep
              just
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
            ];

            shellHook = ''
              export PROJECT_ENV="development"
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath bevyLibs}:$LD_LIBRARY_PATH"

              echo -e "\033[1;33m Welcome to your Nix development shell!\033[0m"
              echo -e "\033[1;33m to run the game use the command: just run\033[0m"
            '';
          };
        });
    };
}
