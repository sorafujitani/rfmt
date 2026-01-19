{
  description = "rfmt - A Ruby code formatter written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain matching CI requirements
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        # Ruby version matching mise.toml
        rubyVersion = pkgs.ruby_3_4;

        # Common build inputs for both shell and package
        buildInputs = with pkgs; [
          # Core dependencies
          rubyVersion
          rubyVersion.devEnv
          rustToolchain

          # System dependencies for building gems with native extensions
          pkg-config
          openssl
          zlib
          libffi
          
          # Build tools
          gcc
          gnumake
          
          # Development tools
          git
          bundix # For generating nix expressions from Gemfile

          # Optional: useful development tools
          sqlite
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          # macOS specific dependencies
          pkgs.libiconv
        ];

        # Development shell environment variables
        shellEnv = {
          # Ruby environment
          RUBY_VERSION = rubyVersion.version;
          GEM_HOME = "$PWD/.nix-gem-home";
          GEM_PATH = "$PWD/.nix-gem-home:${rubyVersion}/lib/ruby/gems/${rubyVersion.version}";
          
          # Rust environment  
          CARGO_HOME = "$PWD/.nix-cargo-home";
          RUSTUP_HOME = "$PWD/.nix-rustup-home";
          
          # Build environment
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.libffi.dev}/lib/pkgconfig";
          
          # Ensure native extensions can find system libraries
          LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          
          # Magnus/rb-sys specific
          RUBY_CC_VERSION = rubyVersion.version;
        } // pkgs.lib.optionalAttrs pkgs.stdenv.isDarwin {
          # macOS specific environment
          DYLD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };

      in
      {
        # Development shell with all dependencies
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          
          shellHook = ''
            echo "🚀 Entering rfmt development environment"
            echo "Ruby: ${rubyVersion.version}"
            echo "Rust: $(${rustToolchain}/bin/rustc --version)"
            echo ""
            
            # Create local directories for gems and cargo
            mkdir -p .nix-gem-home .nix-cargo-home .nix-rustup-home
            
            # Set environment variables
            ${pkgs.lib.concatStringsSep "\n" 
              (pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${value}\"") shellEnv)}
            
            # Add local bin directories to PATH
            export PATH="$PWD/.nix-gem-home/bin:$PWD/.nix-cargo-home/bin:$PATH"
            
            # Verify installation
            echo "📦 Development tools ready:"
            echo "  - Ruby: $(ruby --version)"
            echo "  - Bundler: $(bundle --version)"
            echo "  - Cargo: $(cargo --version)"
            echo "  - Rust: $(rustc --version)"
            echo ""
            echo "🔧 Quick commands:"
            echo "  bundle install     # Install Ruby dependencies"
            echo "  bundle exec rake compile  # Build Rust extension"
            echo "  bundle exec rspec  # Run tests"
            echo "  cargo test --manifest-path ext/rfmt/Cargo.toml  # Run Rust tests"
            echo ""
          '';
        };

        # Package definition for building rfmt
        packages.default = pkgs.stdenv.mkDerivation rec {
          pname = "rfmt";
          version = "1.4.1";

          src = ./.;

          inherit buildInputs;

          nativeBuildInputs = with pkgs; [
            rubyVersion
            rustToolchain
            bundler-audit
          ];

          # Set the same environment variables as the dev shell
          inherit (shellEnv) PKG_CONFIG_PATH LIBRARY_PATH;

          configurePhase = ''
            export GEM_HOME=$out/lib/ruby/gems/${rubyVersion.version}
            export GEM_PATH=$GEM_HOME:${rubyVersion}/lib/ruby/gems/${rubyVersion.version}
            
            # Install bundler if needed
            gem install bundler --no-document
            
            # Install dependencies
            bundle config --local path vendor/bundle
            bundle install
          '';

          buildPhase = ''
            # Build the native extension
            bundle exec rake compile
          '';

          installPhase = ''
            mkdir -p $out/bin $out/lib/ruby/gems/${rubyVersion.version}
            
            # Install the gem
            gem build *.gemspec
            gem install --local --no-document *.gem --install-dir $out/lib/ruby/gems/${rubyVersion.version}
            
            # Create wrapper script
            cat > $out/bin/rfmt << 'EOF'
            #!${pkgs.bash}/bin/bash
            export GEM_HOME=$out/lib/ruby/gems/${rubyVersion.version}
            export GEM_PATH=$GEM_HOME:${rubyVersion}/lib/ruby/gems/${rubyVersion.version}
            exec ${rubyVersion}/bin/ruby $out/lib/ruby/gems/${rubyVersion.version}/bin/rfmt "$@"
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

        # Formatter for nix files
        formatter = pkgs.nixfmt;

        # Development scripts
        apps = {
          # Quick setup script
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
          
          # Test runner
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

        # Check phase for CI/development
        checks = pkgs.lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          # Rust formatting (disabled on macOS for now due to SDK issues)
          rustfmt = pkgs.runCommand "rfmt-rustfmt-check" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${./.}
            cargo fmt --manifest-path ext/rfmt/Cargo.toml -- --check
            touch $out
          '';
        };
      });
}