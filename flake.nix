{
    description = "";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    };

    outputs = { self, nixpkgs, ... }:
    let
        system = "x86_64-linux";
        pkgs = nixpkgs.legacyPackages.${system};
        name = "pesel-rs";
    in
    {
        devShells.${system}.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
                rustup
            ];

            shellHook = ''
                printf '\x1b[36m\x1b[1m\x1b[4mTime to develop ${name}!\x1b[0m\n\n'
            '';
        };

        packages.${system} = {
            default = pkgs.stdenv.mkDerivation {
                name = name;
                src = ./.;

                nativeBuildInputs = with pkgs; [
                    rustup
                ];

                buildPhase = ''
                    cargo build --release
                '';

                installPhase = ''
                    mv target/release/${name} $out
                '';
            };
        };
    };
}
