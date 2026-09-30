{
  description = "A reproducible development environment";

  inputs = {
    # Pin to a stable or unstable branch of nixpkgs
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      # Define supported systems (e.g., x86_64-linux, aarch64-darwin for M1/M2/M3 Macs)
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      # Helper function to generate attributes for all systems
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            # Add build/runtime tools here
            packages = with pkgs; [
              git
              ripgrep
              cargo
              rustc
              just
            ];

            # Environment variables to set inside the shell
            shellHook = ''
              echo "🔨 Welcome to your Nix development shell!"
              export PROJECT_ENV="development"
            '';
          };
        });
    };
}
