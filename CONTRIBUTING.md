# Contributing to ZKP Privacy Airdrop

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Python 3.9+
- PostgreSQL 14+
- Redis 7+

### Setting up the Development Environment

```bash
# Clone the repository
git clone https://github.com/your-org/zkp-privacy-airdrop.git
cd zkp-privacy-airdrop

# Install dependencies
cd contracts && npm install
cd ../cli && cargo build
cd ../relayer && cargo build
cd ../tree-builder && cargo build
```

## Code Style

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Run `cargo clippy` to check for issues
- Ensure all code passes `cargo test`

```bash
cd cli
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test

cd ../relayer
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

### Solidity

- Follow [Solidity Style Guide](https://docs.soliditylang.org/en/latest/style-guide.html)
- Run `npm run lint` to check for style issues
- Run `npm run lint:fix` to auto-fix where possible
- Ensure all contracts compile without warnings

```bash
cd contracts
npm run lint
npm run compile
```

### Python

- Follow [PEP 8](https://peps.python.org/pep-0008/)
- Use [Black](https://github.com/psf/black) for formatting
- Use [isort](https://github.com/PyCQA/isort) for import sorting

```bash
pip install black isort pylint
black tests/
isort tests/
pylint tests/
```

## Testing

### Running Tests

```bash
# Solidity tests
cd contracts
npm test

# Rust tests
cd cli && cargo test
cd ../relayer && cargo test
cd ../tree-builder && cargo test

# Python tests
pytest tests/
```

### Writing Tests

- Ensure new code has appropriate test coverage
- Tests should be deterministic and independent
- Use descriptive test names
- Mock external dependencies

## Commit Guidelines

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions/changes
- `chore`: Build process or auxiliary tool changes
- `ci`: CI/CD changes

Examples:
```
feat(relayer): add support for PLONK proofs

Implement PLONK proof verification to support universal trusted setup,
reducing setup cost and time.

Closes #123
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/amazing-feature`)
3. Make your changes
4. Run tests and linting
5. Commit with conventional commit messages
6. Push to your fork (`git push origin feat/amazing-feature`)
7. Open a Pull Request

### PR Checklist

- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] New tests added for new features
- [ ] Documentation updated
- [ ] Commit messages follow convention
- [ ] No merge commits in the branch

## Security Considerations

When contributing, keep security in mind:
- Never commit secrets or private keys
- Validate all external inputs
- Use safe cryptographic libraries
- Report security vulnerabilities privately

## Getting Help

- Open an issue for bugs or feature requests
- Join our Discord/Slack for discussions
- Check existing documentation first

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
