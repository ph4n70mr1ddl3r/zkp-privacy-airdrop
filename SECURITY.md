# Security Policy

## Reporting Vulnerabilities

If you discover a security vulnerability in this project, please **do not** open a public issue. Instead, send an email to the security team.

### Supported Versions

The following versions are currently being supported with security updates:

| Version | Supported          |
|---------|--------------------|
| 1.0.x   | :white_check_mark: |

## Security Best Practices for Deployments

### Private Key Management

- Never commit private keys to the repository
- Always use environment variables or secret management services
- Rotate keys regularly
- Use hardware wallets for production deployments

### Contract Security

- Always audit contracts before mainnet deployment
- Use testnets extensively
- Implement pause mechanisms
- Set appropriate time locks for sensitive operations

### Relayer Security

- Use rate limiting to prevent abuse
- Monitor for unusual activity
- Keep sufficient ETH balance for gas
- Implement proper error handling

### Rate Limiting

The relayer implements rate limiting at multiple levels:
- Per nullifier: Limits requests from the same claim
- Per IP: Limits requests from the same source
- Global: Overall system-wide limits
- Burst handling: Temporary spike allowance

## Known Security Considerations

### Nullifier Generation

Nullifiers are derived using:
- Keccak256 hashing
- Domain separation to prevent cross-context attacks
- Salt to add randomness

### Smart Contract Vulnerability Classes

The following are addressed in this implementation:

1. **Reentrancy** - Uses OpenZeppelin's ReentrancyGuard
2. **Integer Overflow** - Uses Solidity 0.8+ which has built-in checks
3. **Access Control** - Uses OpenZeppelin's Ownable
4. **Front-running** - Nullifier tracking prevents double-spending

## Audit Information

This project is designed to be audited before production use. If you're interested in conducting a security audit, please contact the team.

## Dependency Updates

Security updates are prioritized. Always keep dependencies updated:

```bash
# Update npm dependencies
cd contracts
npm update

# Update Rust dependencies
cd cli
cargo update

cd relayer
cargo update
```

## License

This project is licensed under the MIT License. See LICENSE file for details.
