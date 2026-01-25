#!/usr/bin/env bash
# Test gem installation across multiple Ruby versions using Docker
#
# Usage:
#   ./scripts/test-gem-install.sh           # Test all versions (3.4, 4.0)
#   ./scripts/test-gem-install.sh 4.0       # Test specific version
#   ./scripts/test-gem-install.sh 3.4 4.0   # Test multiple specific versions

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_VERSIONS=("3.4" "4.0")

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

log_info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1"; }

if [[ $# -eq 0 ]]; then
    VERSIONS=("${DEFAULT_VERSIONS[@]}")
else
    VERSIONS=("$@")
fi

echo ""
echo "========================================"
echo "  rfmt gem installation test"
echo "========================================"
echo ""
log_info "Testing Ruby versions: ${VERSIONS[*]}"
echo ""

cd "$PROJECT_DIR"

FAILED_VERSIONS=()
PASSED_VERSIONS=()

for version in "${VERSIONS[@]}"; do
    echo "----------------------------------------"
    log_info "Testing Ruby ${version}..."
    echo "----------------------------------------"

    IMAGE_NAME="rfmt-test:ruby-${version}"

    if docker build \
        --build-arg RUBY_VERSION="${version}" \
        --target test \
        -f docker/Dockerfile.test \
        -t "$IMAGE_NAME" \
        . 2>&1; then

        log_success "Ruby ${version}: gem install succeeded!"
        PASSED_VERSIONS+=("$version")

        log_info "Running verification tests..."
        docker run --rm "$IMAGE_NAME" ruby -e "
            require 'rfmt'
            puts \"  Version: #{Rfmt::VERSION}\"
            puts \"  Rust version: #{Rfmt.rust_version}\"
            puts \"  Ruby: #{RUBY_VERSION}\"
        " && log_success "Ruby ${version}: verification passed!"
    else
        log_error "Ruby ${version}: gem install FAILED!"
        FAILED_VERSIONS+=("$version")
    fi

    echo ""
done

echo "========================================"
echo "  Summary"
echo "========================================"
echo ""

[[ ${#PASSED_VERSIONS[@]} -gt 0 ]] && log_success "Passed: ${PASSED_VERSIONS[*]}"

if [[ ${#FAILED_VERSIONS[@]} -gt 0 ]]; then
    log_error "Failed: ${FAILED_VERSIONS[*]}"
    echo ""
    exit 1
fi

log_success "All tests passed!"
echo ""
