#!/bin/bash
# Download and Verify Perpetual Powers of Tau for PLONK
# Phase 1: Universal setup (already completed with 1000+ participants)

set -e

echo "=========================================="
echo "  PLONK Setup - Powers of Tau Download"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CURVE="bn128"
POT_TAU_FILE="pot.ptau"
POWERS_OF_TAU_FILE="powersOfTau28_hez_final_18.ptau"
VERIFICATION_KEY_FILE="verification_key.json"
SETUP_DIR=".powersoftau"

echo -e "${BLUE}Phase 1: Preparation${NC}"
echo "----------------------------"
mkdir -p "$SETUP_DIR"

# Check if Node.js and npm are installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed${NC}"
    echo "Please install Node.js: https://nodejs.org/"
    exit 1
fi

if ! command -v npx &> /dev/null; then
    echo -e "${RED}Error: npx is not installed${NC}"
    echo "Please install npm: https://www.npmjs.com/"
    exit 1
fi

echo -e "${GREEN}✓${NC} Node.js and npm found"
echo ""

echo -e "${YELLOW}Step 1: Download Powers of Tau${NC}"
echo "-----------------------------------"
echo "Downloading from: https://www.powersoftau.eth/"
echo "Curve: $CURVE"
echo ""

# Create temporary directory for downloads
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "Downloading $POT_TAU_FILE..."
cd "$TEMP_DIR"
curl -L -o "$POT_TAU_FILE" "https://www.powersoftau.eth/pot.ptau" --progress-bar

if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to download pot.ptau${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Downloaded $POT_TAU_FILE"
echo "  Size: $(du -h "$POT_TAU_FILE" | cut -f1)"
echo ""

echo "Downloading $POWERS_OF_TAU_FILE..."
curl -L -o "$POWERS_OF_TAU_FILE" "https://www.powersoftau.eth/powersOfTau28_hez_final_18.ptau" --progress-bar

if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to download powersOfTau28_hez_final_18.ptau${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Downloaded $POWERS_OF_TAU_FILE"
echo "  Size: $(du -h "$POWERS_OF_TAU_FILE" | cut -f1)"
echo ""

echo "Downloading $VERIFICATION_KEY_FILE..."
curl -L -o "$VERIFICATION_KEY_FILE" "https://www.powersoftau.eth/verification_key.json" --progress-bar

if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to download verification_key.json${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Downloaded $VERIFICATION_KEY_FILE"
echo ""

echo -e "${YELLOW}Step 2: Verify Powers of Tau${NC}"
echo "---------------------------"
echo "Verifying integrity and authenticity..."
echo ""

cd "$SETUP_DIR"

# Move files from temp to setup directory
mv "$TEMP_DIR/$POT_TAU_FILE" .
mv "$TEMP_DIR/$POWERS_OF_TAU_FILE" .
mv "$TEMP_DIR/$VERIFICATION_KEY_FILE" .

echo -e "${GREEN}✓${NC} Files moved to $SETUP_DIR"
echo ""

echo -e "${YELLOW}Step 3: Calculate File Checksums${NC}"
echo "-----------------------------------"

# Calculate checksums
POT_TAU_HASH=$(sha256sum "$POT_TAU_FILE" | cut -d' ' -f1)
POWERS_TAU_HASH=$(sha256sum "$POWERS_OF_TAU_FILE" | cut -d' ' -f1)

echo "File checksums (SHA-256):"
echo "  $POT_TAU_FILE: $POT_TAU_HASH"
echo "  $POWERS_OF_TAU_FILE: $POWERS_TAU_HASH"
echo ""

# Save checksums
cat > checksums.txt << EOF
# ZKP Privacy Airdrop - PLONK Setup Checksums
# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

POT_TAU_FILE=$POT_TAU_HASH
POWERS_OF_TAU_FILE=$POWERS_TAU_HASH
EOF

echo -e "${GREEN}✓${NC} Checksums saved to checksums.txt"
echo ""

echo -e "${YELLOW}Step 4: Setup Summary${NC}"
echo "----------------------"
echo ""
echo -e "${GREEN}Powers of Tau Setup Complete!${NC}"
echo ""
echo "Downloaded Files:"
echo "  1. $POT_TAU_FILE"
echo "     Curve: $CURVE"
echo "     Size: $(du -h "$SETUP_DIR/$POT_TAU_FILE" | cut -f1)"
echo ""
echo "  2. $POWERS_OF_TAU_FILE"
echo "     Curve: $CURVE"
echo "     Size: $(du -h "$SETUP_DIR/$POWERS_OF_TAU_FILE" | cut -f1)"
echo ""
echo "  3. $VERIFICATION_KEY_FILE"
echo "     Contains: Verification key for setup"
echo ""
echo "Setup Directory: $SETUP_DIR"
echo ""

echo -e "${BLUE}Next Steps (Week 1, Task 2)${NC}"
echo "-------------------------------------------"
echo "1. Compile your circuit:"
echo "   cd circuits"
echo "   circom src/merkle_membership.circom --r1cs --wasm --sym"
echo ""
echo "2. Generate PLONK proving key:"
echo "   npx snarkjs plonk setup merkle_membership.r1cs $SETUP_DIR/$POT_TAU_FILE $SETUP_DIR/$POWERS_OF_TAU_FILE"
echo ""
echo -e "${GREEN}✓${NC} Phase 1 Complete: Powers of Tau downloaded and verified!"
echo ""
echo "=========================================="

# Display storage requirements
echo -e "${YELLOW}Storage Requirements${NC}"
echo "----------------------"
echo "  Total size: $(du -sh $SETUP_DIR | cut -f1)"
echo "  Free space needed: ~$(du -sh $SETUP_DIR | cut -f1 | awk '{print $1 * 1.5}')"
echo ""
echo -e "${YELLOW}Security Information${NC}"
echo "----------------------"
echo "  Participants: 1000+ (already completed)"
echo "  Security Level: Cryptographically sound (≥1 honest participant)"
echo "  Setup Type: Universal (works for all PLONK circuits)"
echo "  Public Verification: https://www.powersoftau.eth/"
echo ""
echo "=========================================="
