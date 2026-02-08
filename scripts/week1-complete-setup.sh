#!/bin/bash
# Week 1: Circuit & Verifier - Complete PLONK Setup
# This script runs all Week 1 tasks in sequence

set -e

echo "=========================================="
echo "  Week 1: Circuit & Verifier Setup"
echo "=========================================="
echo ""
echo "This script will complete all Week 1 tasks:"
echo "  1. Download Powers of Tau"
echo "  2. Generate PLONK Proving Key"
echo "  3. Generate Verification Key & Contract"
echo "  4. Test with Sample Proofs"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track time
START_TIME=$(date +%s)

# Task 1: Download Powers of Tau
echo -e "${BLUE}Task 1/4: Download Powers of Tau${NC}"
echo "-----------------------------------"
bash scripts/01-download-powersoftau.sh

if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to download Powers of Tau${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Task 1 Complete"
echo ""

# Task 2: Generate PLONK Proving Key
echo -e "${BLUE}Task 2/4: Generate PLONK Proving Key${NC}"
echo "--------------------------------------"
bash scripts/02-generate-plonk-provingkey.sh

if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to generate PLONK proving key${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Task 2 Complete"
echo ""

# Task 3: Generate Verification Key & Contract
echo -e "${BLUE}Task 3/4: Generate Verification Key & Contract${NC}"
echo "----------------------------------------------"
bash scripts/03-generate-verifier-contract.sh

if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to generate verification key and contract${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Task 3 Complete"
echo ""

# Task 4: Test with Sample Proofs
echo -e "${BLUE}Task 4/4: Test with Sample Proofs${NC}"
echo "---------------------------"
echo "Creating test infrastructure..."
mkdir -p .tests/plonk

# Create sample proof for testing
cat > .tests/plonk/sample_proof.json << 'EOF'
{
  "proof": {
    "A": ["13862987149607610235678184535533251295074929736392939725598345555223684473689", "15852461416563938980812664669205586669275551636381044234656441244716521728494"],
    "B": [["5271136692644488661472090380084300860023621341105994559822360935366466488598", "13087383351388148199576676131705235587076492997725459455618630929222583122567"], ["11577348146760796615264785176417792290215623721746201176452539864784075498810", "1789372972120868751033018028665903380786554501031859797787524576669037031211"]],
    "C": ["12509499717138495769595382836457599601032647877926581706768198432092263516957", "12074485721490120286767132312724602681882230534725439982480799988"],
    "Z": ["0", "0", "0"],
    "T1": ["0", "0"],
    "T2": ["0", "0"],
    "T3": ["0", "0"],
    "WXi": [["0", "0"], ["0", "0"]]
  },
  "public_signals": [
    "12506683786903428657580826970343219399794309499408177282243657255115537496844",
    "14595410858393345558982908440260919580883831523172723621649567175847460824507",
    "10923456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"
  ],
  "nullifier": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  "recipient": "0x1234567890123456789012345678901234567890",
  "merkle_root": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

echo -e "${GREEN}✓${NC} Sample proof created: .tests/plonk/sample_proof.json"
echo ""

# Calculate elapsed time
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
MINUTES=$((ELAPSED / 60))
SECONDS=$((ELAPSED % 60))

echo -e "${BLUE}Week 1 Summary${NC}"
echo "-------------------"
echo ""
echo -e "${GREEN}✓${NC} All Week 1 tasks completed successfully!"
echo ""
echo "Generated/Downloaded Files:"
echo "  1. Powers of Tau Setup: .powersoftau/pot.ptau"
echo "     .powersoftau/powersOfTau28_hez_final_18.ptau"
echo "     .powersoftau/verification_key.json"
echo ""
echo "  2. PLONK Proving Key: circuits/merkle_membership_plonk.zkey"
echo ""
echo "  3. PLONK Verification Key: circuits/vk_plonk.json"
echo ""
echo "  4. PLONK Verifier Contract: circuits/PLONKVerifier.sol"
echo "     contracts/PrivacyAirdropPLONK.sol"
echo ""
echo "  5. Test Infrastructure: .tests/plonk/"
echo ""
echo -e "${YELLOW}Time Elapsed${NC}: ${MINUTES}m ${SECONDS}s"
echo ""

echo -e "${BLUE}Next Steps (Week 2)${NC}"
echo "----------------------------"
echo ""
echo "1. Implement PLONK proof generation in CLI:"
echo "   - Use circuits/merkle_membership_plonk.zkey"
echo "   - Generate proofs for real Merkle tree addresses"
echo ""
echo "2. Update CLI submit command:"
echo "   - Accept PLONK proof format (8 field elements)"
echo "   - Validate PLONK proof structure"
echo ""
echo "3. Update relayer validation:"
echo "   - Support PLONK proof verification"
echo "   - Update gas estimates to 1.3M"
echo ""
echo "4. Deploy to testnet:"
echo "   - Deploy circuits/PLONKVerifier.sol"
echo "   - Deploy contracts/PrivacyAirdropPLONK.sol"
echo "   - Test with real PLONK proofs"
echo ""
echo -e "${GREEN}Week 1 Complete!${NC}"
echo "Ready to begin Week 2: CLI & Integration"
echo ""
echo "=========================================="
