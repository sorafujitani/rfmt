## [Unreleased]

## [1.3.0] - 2026-01-07

### Added
- Precompiled native gem support for multiple platforms:
  - Linux x86_64 (glibc and musl)
  - Linux aarch64 (glibc and musl)
  - macOS x86_64 (Intel) and arm64 (Apple Silicon)
  - Windows x64
- Users no longer need Rust toolchain (cargo) to install rfmt
- GitHub issue and PR templates

### Changed
- Release workflow now uses `oxidize-rb/actions/cross-gem` for cross-compilation
- gemspec updated to exclude compiled artifacts from source gem

## [1.2.7] - 2026-01-04

### Changed
- Remove OpenSSL dependency: Use mtime instead of SHA256 hash for cache invalidation

## [1.2.6] - 2026-01-04

### Changed
- Version bump

## [1.2.5] - 2026-01-04

### Fixed
- Fix trailing comments on `end` keyword (e.g., `end # rubocop:disable`)
- Fix block internal comments being moved outside the block
- Fix blank line preservation between code and comments inside blocks
- Fix leading blank line being added to comment-only files

## [1.2.4] - 2026-01-04

### Fixed
- Fix comment indent space handling

## [1.2.3] - 2026-01-04

### Fixed
- Fix migration file formatting (`emit_rescue` handling for rescue blocks)

## [1.2.2] - 2026-01-04

### Fixed
- Ruby 3.4.1 compatibility: Use `OpenSSL::Digest::SHA256` instead of `Digest::SHA2` to avoid `metadata is not initialized properly` error in Ruby 3.4.1

## [1.2.1] - 2026-01-04

### Fixed
- Ruby 3.4 compatibility: Fix `Digest::SHA256::metadata is not initialized properly` error by using `Digest::SHA2.new(256)` instead of `Digest::SHA256`

## [1.2.0] - 2026-01-04

### Added
- Loop node types support (`for`, `while`, `until`)
- Case/When statement support
- Ensure and Lambda node support
- Begin/End block handling for explicit `begin...end` blocks
- High-priority node types support
- Medium-priority node types support
- Prism supported node viewer task

### Changed
- Consolidated and simplified test suite

### Fixed
- Exclude `.DS_Store` from Git tracking (@topi0247)
- Repository URL changed from `fujitanisora` to `fs0414` (@topi0247)
- End line space handling
- Comment location fix
- End expression indent fix
- Begin formatting fix

## [1.1.0] - 2025-12-12

### Added
- Editor integration (Ruby LSP support)
- Required/Optional keyword parameter node type support

### Fixed
- Migration file superclass corruption (ActiveRecord::Migration[8.1] etc.)

### Changed
- Removed unused scripts and test files (reduced Ruby code by ~38%)

## [1.0.0] - 2025-12-11

### Breaking Changes
- First stable release (v1.0.0)

### Added
- Neovim integration: format-on-save support with autocmd configuration

### Changed
- Set JSON as default output format
- Updated Japanese documentation
- Code formatting improvements

### Fixed
- TOML configuration parsing fix
- Logger initialization fix

## [0.5.0] - 2025-12-07

### Changed
- Synchronized markdown command documentation
- Added project logo

### Fixed
- Removed unnecessary exec command

## [0.4.1] - 2025-11-28

### Fixed
- CLI exec message output optimization for better user experience
- RuboCop compliance issues resolved

### Changed
- Improved output formatting with colored success/failure messages
- Debug logs now only shown with `--verbose` flag or debug environment variables
- Enhanced progress indicators during file processing

## [0.4.0] - 2025-11-26

### Added
- Verbose mode option (`--verbose` flag) for detailed output during formatting
- Git commit hook configuration with Lefthook integration for automatic formatting
- RubyGems badge and installation instructions in README

### Changed
- Improved documentation structure and readability in user guides (English and Japanese)
- Enhanced logging system with verbose output support
- Updated benchmark documentation in README

### Fixed
- Command formatting to execution conversion issues
- Documentation version command display
- Various code quality improvements based on Clippy suggestions

## [0.3.0] - 2025-11-25

### Changed
- **BREAKING**: Default configuration file name changed from `rfmt.yml` to `.rfmt.yml`
  - `rfmt init` now creates `.rfmt.yml` instead of `rfmt.yml`
  - Configuration file search order updated: `.rfmt.yml` > `.rfmt.yaml` > `rfmt.yml` > `rfmt.yaml`
  - This follows Ruby community conventions for hidden configuration files
  - Backward compatibility maintained: `rfmt.yml` is still supported
- Updated README.md to use `.rfmt.yml` in all examples and documentation
- Updated benchmark data with latest accurate measurements (3,241 lines, more realistic performance ratios)
- Removed exaggerated performance claims from README and documentation
- Simplified feature descriptions in README (removed Error Handling and Logging from Features section)

### Documentation
- Updated user guides (English and Japanese) to reflect `.rfmt.yml` as default
- Updated version information in documentation to 0.2.4 → 0.3.0
- Updated benchmark documentation with accurate data from latest measurements
- Added configuration verification examples in examples/ directory
- Improved Ruby API examples with clearer input/output distinction

## [0.2.4] - 2025-11-25

### Fixed
- Fixed if-else expression formatting

## [0.2.3] - 2025-01-25

### Added
- Configuration file (rfmt.yml) is now automatically loaded and applied during formatting
- Automatic config file discovery in current directory, parent directories, and home directory
- Support for custom indent_width and other formatting options via rfmt.yml

### Changed
- Default config file name changed from .rfmt.yml to rfmt.yml (hidden file to regular file)
- Backward compatibility maintained: .rfmt.yml is still supported with lower priority
- Config file search order: rfmt.yml > rfmt.yaml > .rfmt.yml > .rfmt.yaml
- README updated to remove exaggerated expressions and focus on factual, data-driven descriptions

## [0.2.2] - 2025-01-25

### Fixed
- Fixed blank line formatting to output single blank line instead of double blank lines

## [0.2.1] - 2025-01-25

### Fixed
- Fixed GitHub Actions release workflow bundler installation issue
- Resolved clippy warnings in Rust codebase
- Improved enum Default trait implementations using derive macros
- Added `#[cfg(test)]` attributes to test-only code

### Changed
- Updated release workflow to use `bundler-cache: true` for better dependency management

## [0.2.0] - 2025-01-25

### Added
- Security policy implementation with input validation and resource limits
- File size validation (default: 10MB max)
- Source code encoding validation
- Comprehensive error handling with sanitized error messages

### Changed
- Simplified RSpec test suite (reduced from ~200 to 7 essential tests)
- Removed redundant and duplicate tests
- Improved code organization and removed unnecessary comments
- Disabled Windows CI temporarily due to rb-sys compatibility issues

### Fixed
- Rust code formatting issues
- Removed phase-related comments and code noise
- Fixed import order and formatting inconsistencies

## [0.1.0] - 2025-09-08

- Initial release
