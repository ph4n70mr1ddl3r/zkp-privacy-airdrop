# Documentation Inconsistencies Fixed

## Summary of Changes Made

### 1. Created Unified Specification Document
- Created `docs/00-specification.md` as single source of truth
- Consolidated all constants, formats, and interfaces
- Established authoritative reference for implementations

### 2. Fixed Numerical Inconsistencies
- **Token Count**: Standardized to 65,000,000 (was 65M and 65,249,064)
- **Storage Size**: Fixed compressed tree size from 8.3MB to 8.3GB
- **Proof Size**: Clarified 200 bytes (Groth16) vs 832 bytes (Merkle path)

### 3. Clarified Technical Specifications
- **Circuit Design**: Fixed duplicated pseudocode, added Poseidon parameters
- **Nullifier Computation**: Clarified 64-byte input with padding
- **Field Element Encoding**: Added specification for BN128 field encoding
- **Proof Verification**: Separated off-chain and on-chain verification flows

### 4. Standardized Relayer Architecture
- **Funding Model**: Clarified as community-funded donation model
- **Authentication**: Made clear that claim submission is permissionless
- **Multi-Relayer**: Emphasized decentralized, multiple relayer ecosystem
- **Open Source**: Added requirement for public auditability

### 5. Fixed API Documentation
- **Rate Limits**: Standardized to 1 request/60s per nullifier, 100 requests/60s per IP
- **Endpoints**: Clarified relayer URL format (no `/api/v1` in CLI)
- **Error Codes**: Aligned with implementation pseudocode
- **Formats**: Added `hex` and `raw` format options to CLI

### 6. Updated Resource Estimates
- **Development Cost**: Updated from $250k-350k to $800k-1.4M (more realistic)
- **Timeline**: Added missing milestones (tree generation, trusted setup)
- **Testing**: Added load testing, network partition testing

### 7. Added Privacy Analysis
- Created `docs/06-privacy-analysis.md` with transparent limitations
- Documented what the system does/doesn't protect
- Added threat model and mitigation strategies
- Provided user recommendations based on risk profile

### 8. Fixed Miscellaneous Issues
- **CLI Format**: Removed ambiguous `--format binary`, standardized to JSON
- **Node.js Usage**: Clarified where Node.js is actually needed
- **Contract Interface**: Added missing `estimateClaimGas` function
- **Public Inputs**: Documented order as `[merkleRoot, recipient, nullifier]`

## Remaining Considerations

### 1. Poseidon Implementation Details
- Need to specify exact Poseidon parameters (constants, MDS matrix)
- Consider publishing reference implementation

### 2. Trusted Setup Logistics
- Detailed ceremony procedure needed
- Participant selection and verification process

### 3. Merkle Tree Generation
- Compute requirements for 65M leaf tree
- Verification process for community audit

### 4. Legal & Compliance
- Jurisdiction-specific considerations
- Regulatory engagement strategy

### 5. User Experience
- Wallet integration considerations
- Mobile client support
- Browser extension options

## Next Steps

1. **Implementation**: Begin with circuit development based on unified spec
2. **Testing**: Create comprehensive test suite for all components
3. **Audit**: Schedule security audits early in development
4. **Community**: Build multi-relayer ecosystem and community support
5. **Documentation**: Continue refining based on implementation feedback

All inconsistencies have been resolved in the documentation. The unified specification (`docs/00-specification.md`) should be referenced for any future implementation decisions.