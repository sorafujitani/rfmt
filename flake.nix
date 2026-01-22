{
  description = "rfmt - A Ruby code formatter written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-utils.inputs.systems.follows = "systems";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      systems,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        rubyVersion = pkgs.ruby_3_4;

        buildInputs =
          with pkgs;
          [
            rubyVersion
            rubyVersion.devEnv
            rustc
            cargo
            clippy
            rustfmt
            pkg-config
            openssl
            zlib
            libffi
            libyaml
            gnumake
            git
            gh
            bundix
            bundler
            sqlite
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.libiconv
          ];

        shellEnv = {
          RUBY_VERSION = rubyVersion.version;
          GEM_HOME = "$PWD/.nix-gem-home";
          GEM_PATH = "$PWD/.nix-gem-home:${rubyVersion}/lib/ruby/gems/${rubyVersion.version}";
          CARGO_HOME = "$PWD/.nix-cargo-home";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.libffi.dev}/lib/pkgconfig:${pkgs.libyaml}/lib/pkgconfig";
          LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          RUBY_CC_VERSION = rubyVersion.version;
        };

        mkRfmt =
          {
            ruby ? rubyVersion,
          }:
          pkgs.stdenv.mkDerivation {
            pname = "rfmt";
            version = "1.4.1";
            src = ./.;

            buildInputs = buildInputs;
            nativeBuildInputs = with pkgs; [
              ruby
              rustc
              cargo
              bundler
              bundler-audit
            ];

            inherit (shellEnv) PKG_CONFIG_PATH LIBRARY_PATH;

            configurePhase = ''
              export GEM_HOME=$out/lib/ruby/gems/${ruby.version}
              export GEM_PATH=$GEM_HOME:${ruby}/lib/ruby/gems/${ruby.version}
              gem install bundler --no-document
              bundle config --local path vendor/bundle
              bundle install
            '';

            buildPhase = ''
              bundle exec rake compile
            '';

            installPhase = ''
              mkdir -p $out/bin $out/lib/ruby/gems/${ruby.version}
              gem build *.gemspec
              gem install --local --no-document *.gem --install-dir $out/lib/ruby/gems/${ruby.version}
              cat > $out/bin/rfmt << 'EOF'
              #!${pkgs.bash}/bin/bash
              export GEM_HOME=$out/lib/ruby/gems/${ruby.version}
              export GEM_PATH=$GEM_HOME:${ruby}/lib/ruby/gems/${ruby.version}
              exec ${ruby}/bin/ruby $out/lib/ruby/gems/${ruby.version}/bin/rfmt "$@"
              EOF
              chmod +x $out/bin/rfmt
            '';

            meta = with pkgs.lib; {
              description = "A Ruby code formatter written in Rust";
              homepage = "https://github.com/fs0414/rfmt";
              license = licenses.mit;
              maintainers = [ ];
              platforms = platforms.unix;
            };
          };

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          NIX_CFLAGS_COMPILE = pkgs.lib.optionalString pkgs.stdenv.isDarwin "-I${pkgs.darwin.libiconv}/include";
          NIX_LDFLAGS = pkgs.lib.optionalString pkgs.stdenv.isDarwin "-L${pkgs.darwin.libiconv}/lib -liconv";

          shellHook = ''
            mkdir -p .nix-gem-home .nix-cargo-home 2>/dev/null
            ${pkgs.lib.concatStringsSep "\n" (
              pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${value}\"") shellEnv
            )}
            export PATH="$PWD/.nix-gem-home/bin:$PWD/.nix-cargo-home/bin:$PATH"
            if [ -n "$BASH_VERSION" ]; then
              export PS1="(nix:rfmt) \[\033[1;34m\]\W\[\033[0m\] $ "
            elif [ -n "$ZSH_VERSION" ]; then
              export PROMPT="(nix:rfmt) %F{blue}%1~%f $ "
            else
              export PS1="(nix:rfmt) \W $ "
            fi
            if [[ $- == *i* ]]; then
              echo "🚀 rfmt dev env ready | Ruby ${rubyVersion.version} | $(rustc --version 2>/dev/null | cut -d' ' -f1-2 || echo "Rust loading...")"
            fi
          '';
        };

        packages = {
          default = mkRfmt { };
          rfmt = mkRfmt { };
          rfmt-ruby-3_3 = mkRfmt { ruby = pkgs.ruby_3_3; };
          rfmt-ruby-3_4 = mkRfmt { ruby = pkgs.ruby_3_4; };
        };

        formatter = pkgs.nixfmt;

        apps = {
          setup = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScript "rfmt-setup" ''
              set -e
              echo "🔧 Setting up rfmt development environment..."
              if ! command -v nix &> /dev/null; then
                echo "❌ Nix is not installed. Please install Nix first."
                exit 1
              fi
              if ! command -v direnv &> /dev/null; then
                echo "⚠️  direnv not found. Installing..."
                nix profile install nixpkgs#direnv
              fi
              echo "✅ Creating .envrc file..."
              echo "use flake" > .envrc
              direnv allow
              echo "✅ Setup complete! Run 'direnv reload' or enter the directory again."
            '';
          };

          test = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScript "rfmt-test" ''
              set -e
              echo "🧪 Running rfmt tests..."
              echo "📦 Installing dependencies..."
              bundle install
              echo "🔨 Compiling extension..."
              bundle exec rake compile
              echo "🧪 Running Ruby tests..."
              bundle exec rspec
              echo "🦀 Running Rust tests..."
              cargo test --manifest-path ext/rfmt/Cargo.toml
              echo "✅ All tests passed!"
            '';
          };
        };

        checks = {
          nixfmt =
            pkgs.runCommand "rfmt-nixfmt-check"
              {
                nativeBuildInputs = [ pkgs.nixfmt ];
              }
              ''
                cd ${./.}
                nixfmt --check *.nix
                touch $out
              '';
          ruby-syntax =
            pkgs.runCommand "rfmt-ruby-syntax-check"
              {
                nativeBuildInputs = [ rubyVersion ];
              }
              ''
                cd ${./.}
                find lib exe -name "*.rb" -exec ruby -c {} \;
                touch $out
              '';
        }
        // pkgs.lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          rustfmt =
            pkgs.runCommand "rfmt-rustfmt-check"
              {
                nativeBuildInputs = [
                  pkgs.rustfmt
                  pkgs.cargo
                ];
              }
              ''
                cd ${./.}
                cargo fmt --manifest-path ext/rfmt/Cargo.toml -- --check
                touch $out
              '';
        };
      }
    );
}
