# Scripts Documentation

This document provides comprehensive documentation for all scripts and automation tools in the MCP Boilerplate Rust project.

## Overview

The project includes several interactive scripts and utilities designed to streamline development, testing, and deployment workflows:

- **`setup.sh`**: Interactive environment setup and configuration
- **`test.sh`**: Comprehensive testing suite with multiple options
- **`generate_image.py`**: Direct AI image generation tool
- **`scripts/`**: Organized collection of specialized automation tools

## Interactive Scripts (Project Root)

### setup.sh - Environment Setup

**Purpose**: Guided environment configuration and dependency management

**Usage:**
```bash
# Interactive mode (recommended)
./setup.sh

# Direct commands
./setup.sh build      # Build all components
./setup.sh gemini     # Setup Gemini AI integration
./setup.sh all        # Complete setup with AI
./setup.sh clean      # Clean build artifacts
./setup.sh deps       # Verify dependencies
```

**Features:**
- Automatic dependency verification (Rust, Python, Git)
- Guided AI API key configuration
- Build verification and optimization
- Environment validation
- Interactive menu for selective setup

**Requirements:**
- Rust 1.70+ with Cargo
- Python 3.8+ (for AI integration scripts)
- Git (for development workflow)

**Example Session:**
```bash
$ ./setup.sh
🚀 MCP Boilerplate Rust - Setup Script
================================================
📝 Model Context Protocol implementation with AI image generation

✅ Project directory verified

🔧 Setup Options:
1) Complete setup (recommended)
2) Build components only
3) Setup Gemini AI integration
4) Verify dependencies
5) Clean build artifacts
0) Exit

Choose an option [1-5, 0]: 1
```

### test.sh - Testing Suite

**Purpose**: Comprehensive testing with multiple execution modes

**Usage:**
```bash
# Interactive mode (recommended)
./test.sh

# Direct commands
./test.sh quick       # Fast test subset (< 30s)
./test.sh all         # Complete test suite (< 60s)
./test.sh rust        # Rust tests only
./test.sh e2e         # E2E tests only
./test.sh ai          # AI integration tests (requires API keys)
./test.sh servers     # Test all example servers
```

**Test Categories:**
1. **Quick Tests**: Core protocol and unit tests
2. **Rust Tests**: All Cargo workspace tests
3. **E2E Tests**: Real server lifecycle testing
4. **AI Tests**: Live AI integration validation
5. **Server Tests**: Individual server validation
6. **Complete Suite**: All tests with performance validation

**Example Session:**
```bash
$ ./test.sh
🧪 MCP Boilerplate Rust - Testing Suite
==========================================

✅ Project directory verified

🧪 Testing Options:
1) Quick tests (< 30s)
2) Complete test suite (< 60s)
3) E2E tests only
4) AI integration tests
5) Test specific server
6) Performance validation
0) Exit

Choose an option [1-6, 0]: 2
```

### generate_image.py - AI Image Generation

**Purpose**: Direct command-line AI image generation with Google/Gemini integration

**Usage:**
```bash
# Basic usage
python3 generate_image.py "A serene mountain landscape at sunset"

# With options
python3 generate_image.py "A futuristic cityscape" --style artistic --size 1024x1024

# Interactive mode
python3 generate_image.py
```

**Features:**
- Direct Google/Gemini Imagen integration
- Multiple image styles and sizes
- Automatic output management with timestamps
- Error handling and retry logic
- Progress indicators and status feedback

**Output:**
- Images saved to `generated_content/images/`
- Naming format: `YYYYMMDD_HHMMSS_prompt_image.png`
- Automatic thumbnail generation
- Metadata logging for debugging

**Requirements:**
- `GEMINI_API_KEY` environment variable
- Python 3.8+ with `requests` library
- Active internet connection for AI API calls

**Example:**
```bash
$ python3 generate_image.py "A robot writing code"
🎨 MCP Image Generator
=====================

📝 Prompt: A robot writing code
🎨 Style: photorealistic (default)
📐 Size: 1024x1024 (default)

🚀 Generating image with Gemini...
✅ Image generated successfully!
📁 Saved to: generated_content/images/20250117_143052_robot_writing_code_image.png
🖼️  Thumbnail: generated_content/images/20250117_143052_robot_writing_code_thumbnail.png
```

## Organized Scripts (scripts/ directory)

### Shell Scripts (scripts/shell/)

#### Setup Scripts (scripts/shell/setup/)

**`setup_gemini_env.sh`**
- **Purpose**: Configure Google/Gemini API integration
- **Features**: API key validation, environment setup, testing
- **Usage**: `bash scripts/shell/setup/setup_gemini_env.sh`

#### Testing Scripts (scripts/shell/testing/)

**`run_e2e_tests.sh`**
- **Purpose**: Automated E2E test execution
- **Features**: Parallel server testing, timeout management, result aggregation
- **Usage**: `bash scripts/shell/testing/run_e2e_tests.sh`

**`test_image_generation_server.sh`**
- **Purpose**: Specialized testing for image generation server
- **Features**: Both mock and live AI testing, performance validation
- **Usage**: `bash scripts/shell/testing/test_image_generation_server.sh`

#### Verification Scripts (scripts/shell/verification/)

**`verify_gemini_fix.sh`**
- **Purpose**: Verify Google/Gemini integration fixes
- **Features**: API connectivity testing, response validation
- **Usage**: `bash scripts/shell/verification/verify_gemini_fix.sh`

### Python Scripts (scripts/python/)

#### Client Applications (scripts/python/clients/)

**`image_generator.py`**
- **Purpose**: Primary image generation client
- **Features**: Full AI integration, batch processing, error handling
- **Usage**: `python3 scripts/python/clients/image_generator.py "prompt"`

**`jsonrpc_client.py`**
- **Purpose**: Basic MCP client implementation
- **Features**: JSON-RPC communication, protocol testing
- **Usage**: `python3 scripts/python/clients/jsonrpc_client.py`

#### Debug Tools (scripts/python/debug/)

**`response_debugger.py`**
- **Purpose**: Analysis and debugging of server responses
- **Features**: Response structure analysis, performance timing, error diagnosis
- **Usage**: `python3 scripts/python/debug/response_debugger.py "test prompt"`

#### Legacy Scripts (scripts/python/legacy/)

Historical scripts preserved for reference:
- `create_image_with_save.py`: Early image generation implementation
- `demo_image_generation.py`: Demonstration scripts
- `simple_image_test.py`: Basic testing utilities
- `test_gemini_image_gen.py`: Gemini integration testing
- `test_image_generation.py`: General image testing

## Script Integration Patterns

### Chained Operations
```bash
# Complete development cycle
./setup.sh all && ./test.sh quick && ./generate_image.py "test"
```

### CI/CD Integration
```bash
# Continuous integration pipeline
./setup.sh build
./test.sh rust
./test.sh e2e
# Deploy if all pass
```

### Development Workflow
```bash
# Daily development routine
./setup.sh deps        # Verify environment
./test.sh quick        # Quick validation
# Make changes
./test.sh all          # Full validation
git commit -m "feat: changes"
```

## Environment Variables

### Required for AI Features
```bash
export GEMINI_API_KEY="your-gemini-api-key"
```

### Optional Configuration
```bash
export MCP_LOG_LEVEL="info"                    # Logging level
export MCP_SERVER_TIMEOUT="60"                 # Server timeout (seconds)
export MCP_TEST_PARALLEL="true"                # Parallel test execution
export MCP_CACHE_DIR="./cache"                 # Cache directory
```

### Development Environment
```bash
export RUST_LOG="debug"                        # Rust logging
export RUST_BACKTRACE="1"                      # Error backtraces
export CARGO_INCREMENTAL="1"                   # Incremental compilation
```

## Error Handling & Troubleshooting

### Common Issues

#### Setup Script Failures
```bash
# Issue: setup.sh fails with dependency errors
# Solution: Check system dependencies
./setup.sh deps

# Issue: Gemini setup fails
# Solution: Verify API key and connectivity
bash scripts/shell/setup/setup_gemini_env.sh
```

#### Test Script Failures
```bash
# Issue: E2E tests hang or timeout
# Solution: Check for port conflicts
./test.sh servers    # Test servers individually

# Issue: AI tests fail
# Solution: Verify API key and quotas
./test.sh ai
```

#### Generation Script Failures
```bash
# Issue: generate_image.py fails with API errors
# Solution: Debug with response analyzer
python3 scripts/python/debug/response_debugger.py "test prompt"

# Issue: Permission errors on output directory
# Solution: Check directory permissions
mkdir -p generated_content/images
chmod 755 generated_content/images
```

### Debug Mode

All scripts support debug mode for troubleshooting:

```bash
# Enable debug output
DEBUG=1 ./setup.sh
DEBUG=1 ./test.sh
DEBUG=1 python3 generate_image.py "test"
```

### Log Analysis

```bash
# View setup logs
tail -f mcp_logs/setup.log

# View test execution logs
tail -f mcp_logs/test_execution.log

# View AI integration logs
tail -f mcp_logs/ai_integration.log
```

## Performance Characteristics

### Script Execution Times
- **setup.sh**: 30-60 seconds (first run), 5-10 seconds (subsequent)
- **test.sh quick**: < 30 seconds
- **test.sh all**: < 60 seconds
- **generate_image.py**: 3-10 seconds (depending on AI service)

### Resource Usage
- **Memory**: < 100MB for script execution
- **Disk**: < 50MB for logs and temporary files
- **Network**: Variable (AI API calls only)

### Parallel Execution
Scripts are designed for safe parallel execution:
- Random port allocation prevents conflicts
- Temporary directories for isolation
- Proper process cleanup

## Best Practices

### For Daily Development
1. **Start with setup**: `./setup.sh deps` to verify environment
2. **Quick validation**: `./test.sh quick` before making changes
3. **Full validation**: `./test.sh all` before committing
4. **AI testing**: Regular testing with `./generate_image.py`

### For CI/CD Integration
1. **Deterministic setup**: Use direct commands over interactive mode
2. **Timeout protection**: All scripts have built-in timeouts
3. **Error codes**: Scripts return proper exit codes for automation
4. **Log preservation**: Structured logging for debugging failures

### For Production Deployment
1. **Environment validation**: Verify all required variables set
2. **Security review**: Check API key management and access controls
3. **Performance testing**: Validate under production load
4. **Monitoring setup**: Enable structured logging and metrics

## Advanced Usage

### Custom Configuration

**Custom setup with specific options:**
```bash
# Setup only Rust components (no AI)
./setup.sh build

# Setup with custom Gemini configuration
GEMINI_API_KEY="custom-key" ./setup.sh gemini
```

**Custom testing with filters:**
```bash
# Test only filesystem components
./test.sh rust --package filesystem-server

# Test with custom timeouts
TEST_TIMEOUT=120 ./test.sh e2e
```

### Batch Operations

**Generate multiple images:**
```bash
# Batch generation
for prompt in "sunset" "forest" "ocean"; do
    python3 generate_image.py "$prompt"
done
```

**Comprehensive validation:**
```bash
# Complete project validation
./setup.sh all && \
./test.sh all && \
python3 generate_image.py "validation test" && \
echo "✅ Project fully validated"
```

### Integration with Development Tools

**Git hooks integration:**
```bash
# Add to .git/hooks/pre-commit
#!/bin/bash
./test.sh quick || exit 1
```

**IDE integration:**
```bash
# VS Code tasks.json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Quick Test",
            "type": "shell",
            "command": "./test.sh quick",
            "group": "test"
        }
    ]
}
```

## Maintenance & Updates

### Keeping Scripts Updated
1. **Version compatibility**: Scripts automatically detect Rust/Python versions
2. **Dependency updates**: Regular verification of external dependencies
3. **Feature additions**: New capabilities added to interactive menus
4. **Bug fixes**: Issues tracked and resolved through GitHub issues

### Script Development Guidelines
1. **Error handling**: All scripts use `set -e` and proper error checking
2. **User feedback**: Clear color-coded output and progress indicators
3. **Timeout protection**: Built-in timeouts prevent hanging operations
4. **Documentation**: Inline comments and help text
5. **Testing**: Scripts themselves are tested as part of E2E validation

## Integration Examples

### Development Workflow Integration
```bash
# Daily development routine
alias mcp-setup='cd ~/projects/mcp-boilerplate-rust && ./setup.sh deps'
alias mcp-test='cd ~/projects/mcp-boilerplate-rust && ./test.sh quick'
alias mcp-gen='cd ~/projects/mcp-boilerplate-rust && python3 generate_image.py'
```

### Continuous Integration
```yaml
# GitHub Actions workflow
name: MCP Test Suite
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Setup environment
      run: ./setup.sh build
    - name: Run tests
      run: ./test.sh all
    - name: Test AI integration (if keys available)
      run: ./test.sh ai
      env:
        GEMINI_API_KEY: ${{ secrets.GEMINI_API_KEY }}
```

### Docker Integration
```dockerfile
# Include scripts in Docker image
COPY setup.sh test.sh generate_image.py ./
COPY scripts/ ./scripts/

# Run setup during build
RUN ./setup.sh build

# Health check using scripts
HEALTHCHECK --interval=30s --timeout=10s \
  CMD ./test.sh quick || exit 1
```

## Security Considerations

### API Key Management
- Scripts never log API keys in plain text
- Environment variable validation without exposure
- Secure temporary file handling
- Proper cleanup of sensitive data

### File System Security
- Scripts operate within project boundaries
- Temporary files created in secure locations
- Proper permission handling
- No privilege escalation required

### Network Security
- Scripts validate network connectivity before operations
- Timeout protection prevents hanging network calls
- Error handling for network failures
- Support for proxy configurations

## Troubleshooting Guide

### Common Script Issues

#### Permission Errors
```bash
# Fix script permissions
chmod +x setup.sh test.sh

# Fix output directory permissions
mkdir -p generated_content/images
chmod 755 generated_content/images
```

#### Environment Issues
```bash
# Verify Rust installation
rustc --version
cargo --version

# Verify Python installation
python3 --version
pip3 --version

# Check project structure
./setup.sh deps
```

#### Network Issues
```bash
# Test API connectivity
curl -I https://api.google.com/

# Test local network
./test.sh servers
```

### Debug Mode

Enable debug output for all scripts:
```bash
DEBUG=1 ./setup.sh
DEBUG=1 ./test.sh
DEBUG=1 python3 generate_image.py "test"
```

### Log Analysis

Scripts generate structured logs for troubleshooting:
```bash
# View recent setup logs
tail -n 50 mcp_logs/setup.log

# View test execution logs
tail -n 100 mcp_logs/test_execution.log

# View AI integration logs
grep "ERROR" mcp_logs/ai_integration.log
```

## Performance Optimization

### Script Performance
- **Parallel execution**: Where safe and beneficial
- **Caching**: Build artifacts and dependencies cached
- **Incremental operations**: Only rebuild what's changed
- **Resource monitoring**: Memory and CPU usage tracked

### Optimization Tips
```bash
# Use parallel testing
RUST_TEST_THREADS=4 ./test.sh rust

# Enable incremental builds
export CARGO_INCREMENTAL=1
./setup.sh build

# Cache AI responses (development)
export MCP_CACHE_AI_RESPONSES=1
python3 generate_image.py "test"
```

## Extension Guide

### Adding New Scripts

1. **Follow naming convention**: Use descriptive, hyphenated names
2. **Include documentation**: Add to this file and inline comments
3. **Error handling**: Use `set -e` and proper exit codes
4. **User feedback**: Consistent color coding and progress indicators
5. **Integration**: Add to main interactive menus

### Script Template
```bash
#!/bin/bash
# Script description and purpose

set -e

# Colors for consistent output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Validation
if [[ ! -f "Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: Run from project root${NC}"
    exit 1
fi

# Main functionality
echo -e "${BLUE}🔧 Script starting...${NC}"
# Implementation here
echo -e "${GREEN}✅ Script completed${NC}"
```

## Related Documentation

- **[README.md](README.md)**: Project overview and quick start
- **[API.md](API.md)**: Complete API documentation
- **[PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)**: Project organization
- **[E2E_TESTING_CHEATSHEET.md](E2E_TESTING_CHEATSHEET.md)**: Testing patterns

---

**Version**: 1.0 | **Last Updated**: 2025-01-17 | **Status**: Production Ready ✅