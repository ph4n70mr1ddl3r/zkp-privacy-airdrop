# Documentation Updates Summary

## Overview
Updated documentation to clearly reflect the claimant perspective, accounts qualification criteria, contract immutability, and relayer optionality.

## Files Updated

### 1. README.md
- **Added**: Detailed claimant workflow with step-by-step instructions
- **Added**: Accounts list download information and qualification criteria
- **Added**: Clear explanation of two claim options (relayer vs direct)
- **Added**: Emphasis on contract immutability and permissionless access
- **Added**: Note about relayers being optional and community-funded

### 2. docs/00-specification.md (v1.2.0 → v1.3.0)
- **Added**: Accounts qualification criteria (≥0.004 ETH gas fees by Dec 31, 2025)
- **Added**: Contract immutability and key properties section
- **Added**: Clear description of two claiming options
- **Updated**: Token distribution section with detailed accounts criteria

### 3. docs/01-overview.md
- **Added**: Accounts qualification criteria section
- **Updated**: Architecture flow to show both claim paths
- **Added**: Key principles about contract immutability and relayer optionality
- **Enhanced**: User workflow description

### 4. docs/02-technical-specification.md
- **Added**: Comprehensive claimant workflow section at beginning
- **Added**: Step-by-step process with eligibility check
- **Added**: Clear explanation of two claim options
- **Added**: Key principles about system design

### 5. docs/06-privacy-analysis.md
- **Updated**: Statistical analysis section with accounts criteria details
- **Added**: Note about fair distribution to active Ethereum users

### 6. docs/07-consistency-checklist.md
- **Updated**: Version history to v1.3.0
- **Added**: Documentation of all new claimant-focused updates

### 7. New File: CLAIMANT_GUIDE.md
- **Created**: Comprehensive step-by-step guide for claimants
- **Includes**: Eligibility checking, proof generation, claim submission options
- **Covers**: FAQ, security considerations, getting help
- **Provides**: Complete claimant perspective from start to finish

## Key Information Added

### Accounts Qualification Criteria
- **File**: `accounts.csv` (download via `gdown 1yvgsuDMhamUoKAfH59iuyDtm7x5mnHRX`)
- **Size**: 65,249,064 Ethereum addresses
- **Criteria**: Addresses that paid ≥0.004 ETH in gas fees from genesis until December 31, 2025
- **Purpose**: Fair and wide distribution to active Ethereum users

### Claimant Workflow
1. **Check eligibility**: Verify address in accounts.csv
2. **Prepare credentials**: Private key + recipient address
3. **Generate proof**: CLI creates proof.json (offline)
4. **Claim tokens**: Choose relayer (free gas) or direct (pay gas)

### System Design Principles
- **Contract Immutability**: Cannot be modified after deployment
- **Permissionless**: Anyone with valid proof can claim
- **Trustless**: No admin keys, no upgrades
- **Relayers Optional**: Always can submit directly
- **Privacy by Design**: Qualified address never revealed

### Claim Options
1. **Relayer Submission**:
   - Free gas (community-funded)
   - Off-chain proof validation
   - Multiple relayers available
   - Optional convenience service

2. **Direct Submission**:
   - Pay your own gas (~700,000 gas)
   - Maximum privacy
   - No third-party involvement
   - Always available as fallback

## Consistency Achieved
- All documents now reference accounts qualification criteria
- Consistent messaging about contract immutability
- Clear explanation of two claim paths
- Unified terminology across documents
- Updated version numbers where appropriate

## Remaining Issues (From Previous Review)
The following issues from the documentation review still need attention:

1. **Nullifier hash calculation inconsistency** - Different descriptions in spec vs API vs code
2. **Gas estimates mismatch** - Hardcoded 700,000 vs documented breakdown
3. **Storage size unit confusion** - GiB vs GB inconsistencies
4. **Proof size composition** - Unclear what makes 1,032 bytes total
5. **Field element encoding ambiguity** - No clear decimal vs hex guidance

These will be addressed in a future documentation update focusing on technical consistency.

## Additional Update: Relayer as API Service (Not Web App)

### Clarified: Relayer is API-only, No Frontend
**Updated Files**:
- README.md: Changed "Web relayer service" to "API relayer service (no frontend)"
- docs/README.md: Same change
- docs/01-overview.md: Changed "Web Relayer Service" to "API Relayer Service"
- docs/02-technical-specification.md: Added "Pure API, no frontend" clarification
- docs/03-implementation-roadmap.md: Changed "Web Service Foundation" to "API Service Foundation"
- docs/04-api-reference.md: Added note "REST API service with no frontend/UI"
- CLAIMANT_GUIDE.md: Added "Optional API services (no frontend/UI)"

**Key Clarifications**:
- Relayer is a **pure REST API service** with no frontend/UI
- CLI tool interacts directly with API endpoints
- Can be deployed as a standalone API server
- No web interface needed - all interaction is programmatic
- Multiple independent API relayers can run simultaneously

**Architecture Updated**:
- Clear separation: CLI ↔ API ↔ Contract
- No browser or user interface required
- Simplified deployment (just run the API server)
- Direct integration with CLI tool