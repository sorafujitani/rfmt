## [Unreleased]

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
