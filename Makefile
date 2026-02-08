.PHONY: all build test clean test-integration install

all: build

build:
	@echo "Building all components..."
	$(MAKE) -C contracts build
	$(MAKE) -C circuits build
	$(MAKE) -C cli build
	$(MAKE) -C relayer build
	$(MAKE) -C tree-builder build
	@echo "Build complete!"

test:
	@echo "Running tests..."
	$(MAKE) -C contracts test
	$(MAKE) -C circuits test
	$(MAKE) -C cli test
	$(MAKE) -C relayer test
	$(MAKE) -C tree-builder test
	@echo "Tests complete!"

test-integration:
	@echo "Running integration tests..."
	cd tests && pytest

clean:
	@echo "Cleaning..."
	$(MAKE) -C contracts clean
	$(MAKE) -C circuits clean
	$(MAKE) -C cli clean
	$(MAKE) -C relayer clean
	$(MAKE) -C tree-builder clean
	@echo "Clean complete!"

install:
	@echo "Installing CLI..."
	cd cli && cargo install --path .
	@echo "CLI installed!"
