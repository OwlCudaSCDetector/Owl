#!/bin/bash
# OWL AES Analysis - Step-by-Step Guide
# This script will guide you through running the AES example

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Function to print step headers
print_step() {
    echo -e "\n${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${BLUE}STEP $1: $2${NC}"
    echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_command() {
    echo -e "${BOLD}$ $1${NC}"
}

# Welcome message
clear
echo -e "${BOLD}${GREEN}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║     🦉 OWL GPU Side-Channel Detection Framework             ║
║     AES-128-CBC Analysis Tutorial                           ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo "This tutorial will guide you through:"
echo "  1. Setting up the environment"
echo "  2. Running the AES test with GPU tracing"
echo "  3. Analyzing the traces for side-channel leaks"
echo "  4. Visualizing the results"
echo ""
read -p "Press Enter to continue..."

# ============================================================================
print_step 1 "Navigate to AES Example Directory"
# ============================================================================

print_command "cd example/crypt-examples/libgpucrypto"
cd example/crypt-examples/libgpucrypto

print_success "Current directory: $(pwd)"
echo ""
read -p "Press Enter to continue..."

# ============================================================================
print_step 2 "Check Prerequisites"
# ============================================================================

print_info "Checking if AES test binary exists..."
if [ -f "bin/aes_test" ]; then
    print_success "AES test binary found: bin/aes_test"
    file bin/aes_test
else
    print_error "AES test binary not found!"
    print_info "You need to compile it first. Run: make"
    exit 1
fi

echo ""
print_info "Checking if OWL analyzer is built..."
if [ -f "../../../target/release/owl_analyzer" ] || [ -f "../../../target/debug/owl_analyzer" ]; then
    print_success "OWL analyzer found"
else
    print_error "OWL analyzer not built!"
    print_info "Build it with: cd ../../../ && cargo build --release"
    exit 1
fi

echo ""
read -p "Press Enter to continue..."

# ============================================================================
print_step 3 "Clean Previous Results (Optional)"
# ============================================================================

if [ -d "owl_results" ]; then
    print_info "Found previous results directory: owl_results/"
    echo ""
    print_command "rm -rf owl_results"
    echo ""
    read -p "Delete previous results? (y/N): " response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        rm -rf owl_results
        print_success "Previous results deleted"
    else
        print_info "Keeping previous results"
    fi
else
    print_info "No previous results found (this is fine)"
fi

echo ""
read -p "Press Enter to continue..."

# ============================================================================
print_step 4 "Run AES Test with OWL Tracer"
# ============================================================================

print_info "This will:"
echo "  • Run AES encryption 10 times with a FIXED key"
echo "  • Run AES encryption 10 times with RANDOM keys"
echo "  • Capture GPU memory access traces for each run"
echo "  • Save traces to owl_results/0/fix/ and owl_results/0/rnd/"
echo ""

if [ -f "run.sh" ]; then
    print_command "./run.sh"
    echo ""
    read -p "Start tracing? (y/N): " response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        ./run.sh
        print_success "Tracing completed!"
    else
        print_info "Skipping trace collection"
    fi
else
    print_error "run.sh not found!"
    print_info "You can manually run:"
    print_command "owl-wrapper ./bin/aes_test"
    exit 1
fi

echo ""
read -p "Press Enter to continue..."

# ============================================================================
print_step 5 "Verify Trace Files"
# ============================================================================

print_info "Checking generated trace files..."
echo ""

if [ -d "owl_results/0/fix" ]; then
    FIX_COUNT=$(ls -1 owl_results/0/fix/*/kernel.json 2>/dev/null | wc -l)
    print_success "Fixed-key traces: $FIX_COUNT files"
    
    if [ $FIX_COUNT -gt 0 ]; then
        SAMPLE_FILE=$(ls owl_results/0/fix/*/kernel.json | head -1)
        FILE_SIZE=$(du -h "$SAMPLE_FILE" | cut -f1)
        print_info "Sample trace size: $FILE_SIZE"
        echo ""
        print_command "ls -lh $SAMPLE_FILE"
        ls -lh "$SAMPLE_FILE"
    fi
else
    print_error "No fixed-key traces found!"
fi

echo ""

if [ -d "owl_results/0/rnd" ]; then
    RND_COUNT=$(ls -1 owl_results/0/rnd/*/kernel.json 2>/dev/null | wc -l)
    print_success "Random-key traces: $RND_COUNT files"
else
    print_error "No random-key traces found!"
fi

echo ""
read -p "Press Enter to continue..."

# ============================================================================
print_step 6 "Run OWL Analyzer"
# ============================================================================

print_info "Analyzing traces with OWL analyzer..."
print_info "This will:"
echo "  • Load all 20 trace files (10 fixed + 10 random)"
echo "  • Compare memory access patterns statistically"
echo "  • Detect side-channel leaks using Kolmogorov-Smirnov test"
echo "  • Generate report.json with detected leaks"
echo ""

# Find owl_analyzer
if [ -f "../../../target/release/owl_analyzer" ]; then
    OWL_ANALYZER="../../../target/release/owl_analyzer"
elif [ -f "../../../target/debug/owl_analyzer" ]; then
    OWL_ANALYZER="../../../target/debug/owl_analyzer"
else
    print_error "OWL analyzer not found!"
    exit 1
fi

print_command "$OWL_ANALYZER owl_results"
echo ""
read -p "Run analyzer? (y/N): " response
if [[ "$response" =~ ^[Yy]$ ]]; then
    $OWL_ANALYZER owl_results
    print_success "Analysis completed!"
else
    print_info "Skipping analysis"
fi

echo ""
read -p "Press Enter to continue..."

# ============================================================================
print_step 7 "View Results"
# ============================================================================

if [ -f "owl_results/report.json" ]; then
    print_success "Report generated: owl_results/report.json"
    echo ""
    print_info "Report summary:"
    echo ""
    
    # Show kernel leaks
    print_command "jq '.kernel_leak' owl_results/report.json"
    jq '.kernel_leak' owl_results/report.json
    
    echo ""
    print_info "Control flow leaks:"
    print_command "jq '.cf_leak | length' owl_results/report.json"
    CF_LEAKS=$(jq '.cf_leak | length' owl_results/report.json)
    echo "Found $CF_LEAKS control flow leak(s)"
    
    echo ""
    print_info "Data flow leaks:"
    print_command "jq '.df_leak | length' owl_results/report.json"
    DF_LEAKS=$(jq '.df_leak | length' owl_results/report.json)
    echo "Found $DF_LEAKS data flow leak(s)"
    
else
    print_error "No report.json found!"
    print_info "Make sure the analyzer ran successfully"
fi

echo ""
read -p "Press Enter to continue..."

# ============================================================================
print_step 8 "Visualize Results (Optional)"
# ============================================================================

print_info "Creating visualization of memory access patterns..."
echo ""

if [ -f "../../../visualize_results.py" ]; then
    print_command "python3 ../../../visualize_results.py --results-dir owl_results/0"
    echo ""
    read -p "Generate CDF plots? (y/N): " response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        cd ../../..
        python3 visualize_results.py --results-dir example/crypt-examples/libgpucrypto/owl_results/0
        cd example/crypt-examples/libgpucrypto
        print_success "Visualization completed!"
        print_info "Check memory_cdf.png for the plots"
    else
        print_info "Skipping visualization"
    fi
else
    print_error "Visualization script not found!"
    print_info "Expected: ../../../visualize_results.py"
fi

echo ""

# ============================================================================
# Final Summary
# ============================================================================

echo -e "\n${BOLD}${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}${GREEN}✓ TUTORIAL COMPLETED!${NC}"
echo -e "${BOLD}${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

print_info "Summary of what we did:"
echo "  ✓ Collected GPU execution traces (kernel.json files)"
echo "  ✓ Analyzed memory access patterns"
echo "  ✓ Detected side-channel leaks"
echo "  ✓ Generated report and visualizations"
echo ""

print_info "Next steps:"
echo "  • Review report.json for detected leaks"
echo "  • Examine memory_cdf.png to see the difference"
echo "  • Read OWL_EXECUTION_FLOW_DETAILED.md for technical details"
echo "  • Try analyzing other crypto algorithms"
echo ""

print_info "Useful commands:"
echo "  • View full report:"
print_command "    jq '.' owl_results/report.json | less"
echo ""
echo "  • Check specific leak details:"
print_command "    jq '.df_leak' owl_results/report.json"
echo ""
echo "  • Examine a trace file:"
print_command "    jq '.' owl_results/0/fix/0/kernel.json | less"
echo ""

echo -e "${BOLD}Thank you for using OWL! 🦉${NC}\n"
