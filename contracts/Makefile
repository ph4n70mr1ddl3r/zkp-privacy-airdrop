.PHONY: build test clean deploy_testnet deploy_mainnet

build:
	@echo "Compiling contracts..."
	npm install
	npx hardhat compile
	@echo "Contracts compiled successfully!"

test:
	@echo "Running contract tests..."
	npx hardhat test

deploy_testnet:
	@echo "Deploying to Optimism Sepolia testnet..."
	npm run deploy:testnet

deploy_mainnet:
	@echo "Deploying to Optimism mainnet..."
	npm run deploy:mainnet

clean:
	@echo "Cleaning contracts..."
	npx hardhat clean
	rm -rf artifacts cache
	@echo "Clean!"
