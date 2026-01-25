#!/usr/bin/env bash
# Test gem installation across multiple Ruby versions using Docker
#
# Usage:
#   ./scripts/test-gem-install.sh           # Test all versions (3.4, 4.0)
#   ./scripts/test-gem-install.sh 4.0       # Test specific version
#   ./scripts/test-gem-install.sh 3.4 4.0   # Test multiple specific versions

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Default Ruby versions to test
DEFAULT_VERSIONS=("3.4" "4.0")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Parse arguments
if [ $# -eq 0 ]; then
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

    # Build the test image
    if docker build \
        --build-arg RUBY_VERSION="${version}" \
        -f docker/Dockerfile.test \
        -t "$IMAGE_NAME" \
        . 2>&1; then

        log_success "Ruby ${version}: gem install succeeded!"
        PASSED_VERSIONS+=("$version")

        # Run additional verification
        log_info "Running verification tests..."
        if docker run --rm "$IMAGE_NAME" ruby -e "
            require 'rfmt'
            puts \"  Version: #{Rfmt::VERSION}\"
            puts \"  Rust version: #{Rfmt.rust_version}\"
            puts \"  Ruby: #{RUBY_VERSION}\"
        "; then
            log_success "Ruby ${version}: verification passed!"
        else
            log_warn "Ruby ${version}: verification had issues"
        fi
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

if [ ${#PASSED_VERSIONS[@]} -gt 0 ]; then
    log_success "Passed: ${PASSED_VERSIONS[*]}"
fi

if [ ${#FAILED_VERSIONS[@]} -gt 0 ]; then
    log_error "Failed: ${FAILED_VERSIONS[*]}"
    echo ""
    exit 1
fi

log_success "All tests passed!"
echo ""
