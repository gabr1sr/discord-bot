{ pkgs, lib, config, inputs, ... }:

{
  packages = [
    pkgs.openssl
  ];
  
  languages.rust = {
    enable = true;
    channel = "stable";

    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "rust-docs"
      "rust-src"
    ];
  };

  pre-commit.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };

  cachix.enable = false;

  enterTest = ''
    cargo test
  '';
}
