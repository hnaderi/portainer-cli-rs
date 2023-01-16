{
  inputs = { nixpkgs.url = "github:nixos/nixpkgs"; };
  outputs = { self, nixpkgs }:
    let pkgs = import nixpkgs { system = "x86_64-linux"; };
    in {
      devShell.x86_64-linux = with pkgs;
        mkShell {
          name = "portainer-cli";
          nativeBuildInputs = [ rustc cargo gcc openssl.dev pkg-config ];
          buildInputs = [ rustfmt rust-analyzer clippy rustup ];

          RUST_SRC_PATH =
            "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
    };
}
