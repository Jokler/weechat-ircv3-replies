{
  description = "An IRCv3 reply plugin for WeeChat";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/fd1462031fdee08f65fd0b4c6b64e22239a77870";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
        # Dependencies needed for both compiling and running
        buildInputs = with pkgs; [
          weechat
        ];
      in {
        # Development Shell configuration
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            buildInputs
            ++ [
              rust-analyzer
              rustfmt
              clippy
              bacon
            ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = [
            "-isystem ${pkgs.glibc.dev}/include"
            "-isystem ${pkgs.weechat-unwrapped}/include"
          ];

          # Environment variables required during the cargo compilation phase
          shellHook = ''
            export WEECHAT_HOME="$PWD/test_dir"
            echo "WeeChat Rust Development Shell Activated!"
            echo "Run 'cargo build --release' to compile your plugin."
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "weechat-ircv3-reply-tags";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;

            outputHashes = {
              "weechat-0.4.0" = "sha256-ryWuUwz3qXx5WwcNvrBSGoIjjMufNOzyr5/3omNQzUA=";
            };
          };

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.glibc.dev}/include -isystem ${pkgs.weechat-unwrapped}/include";

          inherit buildInputs;

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustPlatform.bindgenHook
          ];

          postInstall = ''
            # Nix by default might drop it in /bin. WeeChat plugins don't have a main entry point,
            # so we ensure the compiled library is preserved appropriately.
            mkdir -p $out/lib/weechat/plugins
            find target -name "libircv3_reply_tags.so" -exec cp {} $out/lib/weechat/plugins/ircv3_reply_tags.so \;
          '';
        };
      }
    );
}
