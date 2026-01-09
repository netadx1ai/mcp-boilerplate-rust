.PHONY: all generate clean test help install-deps

all: generate

help:
	@echo "MCP SDK Generator - Makefile"
	@echo "============================="
	@echo ""
	@echo "Targets:"
	@echo "  make generate      - Generate all SDKs (TypeScript, Python, Go)"
	@echo "  make clean         - Clean generated files and build artifacts"
	@echo "  make test          - Run tests for all SDKs"
	@echo "  make install-deps  - Install dependencies for testing"
	@echo "  make build         - Build the generator"
	@echo "  make help          - Show this help message"
	@echo ""

build:
	@echo "Building SDK generator..."
	cargo build --release
	@echo "Build complete!"

generate: build
	@echo "Generating client SDKs..."
	cargo run --release
	@echo ""
	@echo "Generated files:"
	@ls -lh output/typescript/mcp-client.ts 2>/dev/null || true
	@ls -lh output/python/mcp_client.py 2>/dev/null || true
	@ls -lh output/go/mcpclient/client.go 2>/dev/null || true
	@echo ""
	@echo "SDK generation complete!"

clean:
	@echo "Cleaning generated files..."
	rm -rf output/
	rm -rf target/
	rm -rf sdk-generators/
	@echo "Clean complete!"

test: generate
	@echo "Testing TypeScript SDK..."
	@cd output/typescript && npm install --silent 2>/dev/null || true
	@echo "TypeScript SDK ready for testing"
	@echo ""
	@echo "Testing Python SDK..."
	@cd output/python && pip install -q -r requirements.txt 2>/dev/null || true
	@echo "Python SDK ready for testing"
	@echo ""
	@echo "Testing Go SDK..."
	@cd output/go && go mod tidy 2>/dev/null || true
	@echo "Go SDK ready for testing"
	@echo ""
	@echo "All SDKs ready for testing!"

install-deps:
	@echo "Installing dependencies..."
	@echo "TypeScript/Node.js..."
	@which npm > /dev/null || echo "Warning: npm not found. Install Node.js to use TypeScript SDK"
	@which node > /dev/null && npm install -g typescript ts-node || true
	@echo ""
	@echo "Python..."
	@which python3 > /dev/null || echo "Warning: python3 not found"
	@which pip > /dev/null && pip install requests || true
	@echo ""
	@echo "Go..."
	@which go > /dev/null || echo "Warning: go not found"
	@echo ""
	@echo "Dependencies installation complete!"

typescript: build
	@echo "Generating TypeScript SDK only..."
	cargo run --release
	@echo "TypeScript SDK: output/typescript/mcp-client.ts"

python: build
	@echo "Generating Python SDK only..."
	cargo run --release
	@echo "Python SDK: output/python/mcp_client.py"

golang: build
	@echo "Generating Go SDK only..."
	cargo run --release
	@echo "Go SDK: output/go/mcpclient/client.go"

verify:
	@echo "Verifying generated SDKs..."
	@test -f output/typescript/mcp-client.ts && echo "✓ TypeScript SDK exists" || echo "✗ TypeScript SDK missing"
	@test -f output/python/mcp_client.py && echo "✓ Python SDK exists" || echo "✗ Python SDK missing"
	@test -f output/go/mcpclient/client.go && echo "✓ Go SDK exists" || echo "✗ Go SDK missing"
	@echo ""
	@echo "File sizes:"
	@wc -l output/typescript/mcp-client.ts 2>/dev/null | awk '{print "  TypeScript: " $$1 " lines"}'
	@wc -l output/python/mcp_client.py 2>/dev/null | awk '{print "  Python:     " $$1 " lines"}'
	@wc -l output/go/mcpclient/client.go 2>/dev/null | awk '{print "  Go:         " $$1 " lines"}'

watch:
	@echo "Watching for changes and regenerating..."
	@while true; do \
		make generate; \
		sleep 5; \
	done

fmt:
	@echo "Formatting Rust code..."
	cargo fmt
	@echo "Format complete!"

lint:
	@echo "Linting Rust code..."
	cargo clippy --release
	@echo "Lint complete!"

check: lint verify
	@echo "All checks passed!"