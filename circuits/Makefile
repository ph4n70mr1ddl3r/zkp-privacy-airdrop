.PHONY: build test clean

build:
	@echo "Building circuits..."
	npm install
	mkdir -p build
	circom src/merkle_membership.circom --r1cs --wasm --sym -o build
	@echo "Circuit built successfully!"

test:
	@echo "Testing circuits..."
	npm test

clean:
	@echo "Cleaning circuits..."
	rm -rf build
	@echo "Clean!"
