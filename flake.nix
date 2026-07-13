{
  description = "A very basic flake";

  inputs = {
    # nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    solana-nix = {
      url = "github:ellttben/solana-nix";
      # inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      # nixpkgs,
      solana-nix,
    }:
    let
      pkgs = solana-nix.inputs.nixpkgs.legacyPackages."x86_64-linux";
    in
    {
      devShells."x86_64-linux".default = pkgs.mkShell {
        buildInputs = [
          pkgs.cargo
          pkgs.rustc
          pkgs.rustfmt
          pkgs.clippy
          pkgs.rust-analyzer
          pkgs.openssl
          pkgs.nodejs
          pkgs.yarn
          solana-nix.packages.x86_64-linux.solana-rust
          solana-nix.packages.x86_64-linux.solana-platform-tools
          solana-nix.packages.x86_64-linux.solana-cli
          solana-nix.packages.x86_64-linux.anchor-cli
        ];

        nativeBuildInputs = [ pkgs.pkg-config ];

        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
    };
}
