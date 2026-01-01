#!/bin/bash
# test-ci.sh - Local CI test script for myrtle.moe
# This script simulates the GitHub Actions CI workflow locally

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Configuration
SKIP_FRONTEND=false
SKIP_BACKEND=false
SKIP_ASSETS=false
VERBOSE=false
FIX_LINT=false
PARALLEL=false

# Tracking results
declare -a PASSED_CHECKS=()
declare -a FAILED_CHECKS=()
declare -a SKIPPED_CHECKS=()

# Functions
print_header() {
    echo ""
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  $1${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_step() {
    echo -e "${CYAN}➤ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${NC}  $1${NC}"
}

run_check() {
    local name="$1"
    local cmd="$2"
    local continue_on_error="${3:-false}"

    print_step "$name"

    if $VERBOSE; then
        echo "  Running: $cmd"
    fi

    if eval "$cmd"; then
        print_success "$name passed"
        PASSED_CHECKS+=("$name")
        return 0
    else
        if [ "$continue_on_error" = "true" ]; then
            print_warning "$name failed (continuing)"
            FAILED_CHECKS+=("$name (non-blocking)")
            return 0
        else
            print_error "$name failed"
            FAILED_CHECKS+=("$name")
            return 1
        fi
    fi
}

skip_check() {
    local name="$1"
    print_warning "Skipping: $name"
    SKIPPED_CHECKS+=("$name")
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        print_error "$1 is not installed"
        return 1
    fi
    return 0
}

print_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Local CI test script that simulates GitHub Actions workflows.

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -f, --fix           Run lint fixes instead of just checking
    -p, --parallel      Run independent checks in parallel
    --skip-frontend     Skip frontend checks
    --skip-backend      Skip backend checks
    --skip-assets       Skip assets checks
    --frontend-only     Only run frontend checks
    --backend-only      Only run backend checks
    --assets-only       Only run assets checks

EXAMPLES:
    $0                      # Run all checks
    $0 --frontend-only      # Only test frontend
    $0 --fix                # Run with auto-fix for linting
    $0 -v --skip-assets     # Verbose, skip assets

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            print_usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -f|--fix)
            FIX_LINT=true
            shift
            ;;
        -p|--parallel)
            PARALLEL=true
            shift
            ;;
        --skip-frontend)
            SKIP_FRONTEND=true
            shift
            ;;
        --skip-backend)
            SKIP_BACKEND=true
            shift
            ;;
        --skip-assets)
            SKIP_ASSETS=true
            shift
            ;;
        --frontend-only)
            SKIP_BACKEND=true
            SKIP_ASSETS=true
            shift
            ;;
        --backend-only)
            SKIP_FRONTEND=true
            SKIP_ASSETS=true
            shift
            ;;
        --assets-only)
            SKIP_FRONTEND=true
            SKIP_BACKEND=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Start
print_header "myrtle.moe CI Test"
echo "Testing at: $SCRIPT_DIR"
echo "Date: $(date)"
echo ""

# Check prerequisites
print_header "Checking Prerequisites"

PREREQ_FAILED=false

if ! check_command "rustc"; then
    print_info "Install Rust from https://rustup.rs"
    PREREQ_FAILED=true
fi

if ! check_command "cargo"; then
    print_info "Cargo should come with Rust installation"
    PREREQ_FAILED=true
fi

if ! check_command "bun"; then
    print_warning "Bun not found, trying npm instead"
    if ! check_command "npm"; then
        print_info "Install Bun from https://bun.sh or npm from https://nodejs.org"
        PREREQ_FAILED=true
    fi
fi

if [ "$PREREQ_FAILED" = true ]; then
    print_error "Prerequisites check failed. Please install missing tools."
    exit 1
fi

print_success "All prerequisites satisfied"

# Frontend checks
if [ "$SKIP_FRONTEND" = false ]; then
    print_header "Frontend Checks (Next.js + Bun)"

    cd "$SCRIPT_DIR/frontend"

    # Determine package manager
    PKG_MGR="bun"
    if ! command -v bun &> /dev/null; then
        PKG_MGR="npm"
        print_warning "Using npm instead of bun"
    fi

    # Install dependencies
    print_step "Installing dependencies..."
    if [ "$PKG_MGR" = "bun" ]; then
        bun install --frozen-lockfile 2>/dev/null || bun install
    else
        npm ci 2>/dev/null || npm install
    fi
    print_success "Dependencies installed"

    # Biome lint
    if [ "$FIX_LINT" = true ]; then
        run_check "Biome lint (with fix)" "$PKG_MGR run lint:fix" "true"
    else
        run_check "Biome lint" "$PKG_MGR run lint" "true"
    fi

    # Biome check
    run_check "Biome check" "$PKG_MGR run check" "true"

    # TypeScript
    run_check "TypeScript type check" "$PKG_MGR run typecheck" "false"

    # Build
    run_check "Next.js build" "SKIP_ENV_VALIDATION=true $PKG_MGR run build" "false"

    cd "$SCRIPT_DIR"
else
    skip_check "Frontend checks"
fi

# Backend checks
if [ "$SKIP_BACKEND" = false ]; then
    print_header "Backend Checks (Rust + Axum)"

    cd "$SCRIPT_DIR/backend"

    # Format check
    if [ "$FIX_LINT" = true ]; then
        run_check "Cargo format (with fix)" "cargo fmt --all" "false"
    else
        run_check "Cargo format check" "cargo fmt --all -- --check" "true"
    fi

    # Clippy
    run_check "Clippy lint" "cargo clippy --all-targets --all-features -- -D warnings" "true"

    # Build
    run_check "Cargo build" "cargo build --release" "false"

    # Tests
    run_check "Cargo test" "cargo test --all-features" "true"

    cd "$SCRIPT_DIR"
else
    skip_check "Backend checks"
fi

# Assets checks
if [ "$SKIP_ASSETS" = false ]; then
    print_header "Assets Checks (Rust)"

    # Unity-RS
    print_step "Checking unity-rs..."
    cd "$SCRIPT_DIR/assets/unity-rs"

    if [ "$FIX_LINT" = true ]; then
        run_check "unity-rs: Cargo format (with fix)" "cargo fmt --all" "false"
    else
        run_check "unity-rs: Cargo format check" "cargo fmt --all -- --check" "true"
    fi

    # Note: Using --lib only because tests/bins require FMOD linking
    run_check "unity-rs: Clippy lint" "cargo clippy --lib -- -D warnings" "true"
    run_check "unity-rs: Check compilation" "cargo check --release --lib" "false"

    # Downloader
    print_step "Checking downloader..."
    cd "$SCRIPT_DIR/assets/downloader"

    if [ "$FIX_LINT" = true ]; then
        run_check "downloader: Cargo format (with fix)" "cargo fmt --all" "false"
    else
        run_check "downloader: Cargo format check" "cargo fmt --all -- --check" "true"
    fi

    # Note: Using --lib only because tests/bins require FMOD linking
    run_check "downloader: Clippy lint" "cargo clippy --lib -- -D warnings" "true"
    run_check "downloader: Check compilation" "cargo check --release --lib" "false"

    # Unpacker
    print_step "Checking unpacker..."
    cd "$SCRIPT_DIR/assets/unpacker"

    if [ "$FIX_LINT" = true ]; then
        run_check "unpacker: Cargo format (with fix)" "cargo fmt --all" "false"
    else
        run_check "unpacker: Cargo format check" "cargo fmt --all -- --check" "true"
    fi

    # Note: Using --lib only because tests/bins require FMOD linking
    run_check "unpacker: Clippy lint" "cargo clippy --lib -- -D warnings" "true"
    run_check "unpacker: Check compilation" "cargo check --release --lib" "false"

    cd "$SCRIPT_DIR"
else
    skip_check "Assets checks"
fi

# Summary
print_header "Test Summary"

echo -e "${GREEN}Passed (${#PASSED_CHECKS[@]}):${NC}"
for check in "${PASSED_CHECKS[@]}"; do
    echo -e "  ${GREEN}✓${NC} $check"
done

if [ ${#FAILED_CHECKS[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed (${#FAILED_CHECKS[@]}):${NC}"
    for check in "${FAILED_CHECKS[@]}"; do
        echo -e "  ${RED}✗${NC} $check"
    done
fi

if [ ${#SKIPPED_CHECKS[@]} -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}Skipped (${#SKIPPED_CHECKS[@]}):${NC}"
    for check in "${SKIPPED_CHECKS[@]}"; do
        echo -e "  ${YELLOW}⚠${NC} $check"
    done
fi

echo ""

# Determine exit code
BLOCKING_FAILURES=0
for check in "${FAILED_CHECKS[@]}"; do
    if [[ "$check" != *"(non-blocking)"* ]]; then
        ((BLOCKING_FAILURES++))
    fi
done

if [ $BLOCKING_FAILURES -gt 0 ]; then
    print_error "CI test failed with $BLOCKING_FAILURES blocking failure(s)"
    exit 1
else
    print_success "CI test completed successfully!"
    if [ ${#FAILED_CHECKS[@]} -gt 0 ]; then
        print_warning "Note: ${#FAILED_CHECKS[@]} non-blocking check(s) failed"
    fi
    exit 0
fi
