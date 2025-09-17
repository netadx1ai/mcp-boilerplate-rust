# Shell Scripts Directory

This directory contains organized shell scripts for setup, testing, and verification of the MCP Boilerplate Rust project.

## Directory Structure

```
scripts/shell/
├── README.md                         # This file
├── setup/                           # Environment and configuration setup
│   └── setup_gemini_env.sh          # Google Gemini API environment setup
├── testing/                         # Test execution scripts
│   ├── run_e2e_tests.sh             # End-to-end testing suite
│   └── test_image_generation_server.sh  # Image generation server tests
└── verification/                    # Verification and validation scripts
    └── verify_gemini_fix.sh          # Gemini integration verification
```

## Quick Start

### From Project Root (Recommended)

```bash
# Interactive setup menu
./setup.sh

# Interactive testing menu
./test.sh

# Direct commands
./setup.sh gemini     # Setup Gemini API
./test.sh quick       # Run quick tests
```

### Direct Script Execution

```bash
# Setup Gemini environment
scripts/shell/setup/setup_gemini_env.sh

# Run verification tests
scripts/shell/verification/verify_gemini_fix.sh

# Run comprehensive E2E tests
scripts/shell/testing/run_e2e_tests.sh

# Test image generation server
scripts/shell/testing/test_image_generation_server.sh
```

## Script Categories

### 🔧 Setup Scripts (`setup/`)

#### `setup_gemini_env.sh`
**Purpose**: Configure Google Gemini API environment for AI-powered image generation.

**Features**:
- Interactive API key setup with validation
- Environment variable configuration (.env file)
- Shell profile integration (persistent setup)
- API key testing and verification
- Project build and basic functionality testing

**Usage**:
```bash
scripts/shell/setup/setup_gemini_env.sh
```

**What it does**:
1. Validates project directory structure
2. Checks for existing API key configuration
3. Prompts for new API key input
4. Tests API key with Google Gemini service
5. Saves configuration to .env file
6. Updates shell profile for persistence
7. Builds the project
8. Runs basic functionality tests

**Requirements**:
- Google Gemini API key from [Google AI Studio](https://makersuite.google.com/app/apikey)
- Internet connection for API validation
- Write permissions for .env file and shell profile

### 🧪 Testing Scripts (`testing/`)

#### `run_e2e_tests.sh`
**Purpose**: Comprehensive end-to-end testing suite for all MCP servers.

**Features**:
- Tests multiple server types (filesystem, image generation, blog, creative content)
- Both stdio and HTTP transport testing
- Deadlock detection and prevention
- Performance benchmarking
- Timeout protection
- Detailed reporting

**Usage**:
```bash
scripts/shell/testing/run_e2e_tests.sh
```

**Test Coverage**:
- Server startup and shutdown
- Tool registration and discovery
- Request/response handling
- Error conditions and edge cases
- Performance metrics
- Memory leak detection

#### `test_image_generation_server.sh`
**Purpose**: Specialized testing for the image generation server.

**Features**:
- Mock mode testing (no API key required)
- AI mode testing (with Gemini API)
- Both stdio and HTTP transport protocols
- Response validation and parsing
- Performance measurement
- Error handling verification

**Usage**:
```bash
scripts/shell/testing/test_image_generation_server.sh
```

**Test Scenarios**:
- Basic tool discovery
- Mock image generation
- AI-enhanced image generation
- Invalid prompt handling
- Timeout scenarios
- Transport protocol switching

### ✅ Verification Scripts (`verification/`)

#### `verify_gemini_fix.sh`
**Purpose**: Verify Google Gemini integration fixes and validate production readiness.

**Features**:
- Code quality verification (fmt, clippy, check)
- Build system validation
- Unit test execution
- Documentation generation
- Functional testing (mock and AI modes)
- Production readiness assessment

**Usage**:
```bash
scripts/shell/verification/verify_gemini_fix.sh
```

**Verification Steps**:
1. **Environment Check**: API key availability
2. **Code Quality**: Formatting, linting, compilation
3. **Build Verification**: Successful compilation
4. **Test Suite**: Unit tests execution
5. **Documentation**: Doc generation and validation
6. **Functional Testing**: Server startup and basic operations
7. **Production Assessment**: Overall readiness evaluation

## Common Usage Patterns

### Initial Project Setup

```bash
# Complete setup from scratch
./setup.sh all

# Or step by step
./setup.sh gemini    # Setup API
./setup.sh build     # Build project
./test.sh quick      # Verify everything works
```

### Development Workflow

```bash
# After making changes
./test.sh quality    # Check code quality
./test.sh rust       # Run Rust tests
./test.sh functional # Test functionality

# Before committing
./test.sh all        # Run complete test suite
```

### Troubleshooting

```bash
# Verify Gemini integration
scripts/shell/verification/verify_gemini_fix.sh

# Test specific server
scripts/shell/testing/test_image_generation_server.sh

# Debug E2E issues
scripts/shell/testing/run_e2e_tests.sh
```

## Environment Requirements

### System Dependencies
- **Bash**: Version 4.0+ (macOS, Linux)
- **Rust**: Latest stable version with Cargo
- **Python**: 3.7+ (for client scripts)
- **curl**: For API testing
- **timeout**: For process management

### Project Dependencies
- Must be run from project root directory
- Requires `Cargo.toml` and `examples/` directory
- Built binaries in `target/debug/` (after build)

### API Dependencies
- **Google Gemini API Key**: For AI mode testing
- **Internet Connection**: For API validation
- **API Quotas**: Sufficient quota for testing

## Script Configuration

### Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `GEMINI_API_KEY` | Google Gemini API key | For AI mode | None |
| `GOOGLE_API_KEY` | Alternative API key name | For AI mode | None |
| `TIMEOUT_SECONDS` | Test timeout duration | No | 30 |
| `MAX_STARTUP_TIME` | Server startup timeout | No | 5 |

### Configuration Files
- `.env` - Project environment variables
- Shell profiles (`.zshrc`, `.bashrc`, `.bash_profile`) - Persistent environment

## Error Handling

### Common Exit Codes
- `0` - Success
- `1` - General error (script failure)
- `2` - Missing dependencies
- `3` - API key issues
- `4` - Build failures
- `5` - Test failures

### Common Issues and Solutions

#### "Script must be run from project root"
```bash
cd /path/to/mcp-boilerplate-rust
./setup.sh
```

#### "API key test failed"
- Verify API key is correct
- Check internet connection
- Ensure sufficient API quota
- Try regenerating API key

#### "Build failed"
```bash
cargo clean
cargo build --workspace
```

#### "Tests timeout"
- Check for deadlocks in code
- Increase timeout values
- Review recent changes for infinite loops

## Best Practices

### For Development
1. **Always run quality checks** before committing
2. **Use quick tests** during active development
3. **Run full test suite** before pull requests
4. **Keep API keys secure** and out of version control

### For CI/CD
1. **Use command-line interface** for automation
2. **Set appropriate timeouts** for your environment
3. **Configure API keys** via secure environment variables
4. **Parse exit codes** for build status

### For Production
1. **Verify all tests pass** before deployment
2. **Run verification script** to ensure production readiness
3. **Monitor test performance** for regressions
4. **Keep scripts updated** with codebase changes

## Integration with Main Scripts

These shell scripts are integrated with the main project convenience scripts:

- `setup.sh` (project root) → calls scripts in `scripts/shell/setup/`
- `test.sh` (project root) → calls scripts in `scripts/shell/testing/` and `scripts/shell/verification/`

This allows for both convenient high-level access and direct low-level script execution.

## Maintenance

### Adding New Scripts
1. Place in appropriate subdirectory (`setup/`, `testing/`, `verification/`)
2. Follow naming convention: `action_component.sh`
3. Include proper error handling and timeouts
4. Add to main wrapper scripts (`setup.sh`, `test.sh`)
5. Update this README

### Updating Existing Scripts
1. Test changes in isolated environment
2. Ensure backward compatibility
3. Update documentation if interface changes
4. Verify integration with wrapper scripts

## Security Considerations

- **API Keys**: Never commit API keys to version control
- **Permissions**: Scripts create files with appropriate permissions
- **Cleanup**: Temporary files and processes are properly cleaned up
- **Input Validation**: All user inputs are validated
- **Network Security**: API calls use HTTPS and proper authentication

## Performance

### Script Execution Times
- Setup scripts: 30-60 seconds (including API validation)
- Verification scripts: 60-120 seconds (full test suite)
- Testing scripts: 30-300 seconds (depending on scope)

### Optimization Tips
- Use `quick` test modes for rapid iteration
- Run specific test categories instead of full suite
- Cache build artifacts when possible
- Use local mock mode for development

This directory provides a comprehensive, production-ready shell scripting framework for MCP project management, testing, and deployment preparation.