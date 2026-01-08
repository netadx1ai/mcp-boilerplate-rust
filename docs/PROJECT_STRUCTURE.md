# Project Structure

**MCP Boilerplate Rust v0.4.0-rc**  
**Last Updated:** 2026-01-08 (HCMC Timezone)

---

## 📁 Complete Directory Tree

```
mcp-boilerplate-rust/
│
├── 📄 Core Files
│   ├── README.md                      # Project overview and quick start
│   ├── START_HERE.md                  # Main entry point for new users
│   ├── LICENSE                        # MIT License
│   ├── Cargo.toml                     # Rust dependencies
│   ├── Cargo.lock                     # Dependency lock file
│   ├── Makefile                       # Build automation
│   ├── Dockerfile                     # Container configuration
│   ├── docker-compose.yml             # Docker compose setup
│   ├── deny.toml                      # Cargo deny configuration
│   ├── .gitignore                     # Git ignore rules
│   └── COMMIT_MESSAGE.txt             # Pre-written commit message
│
├── 📦 Source Code (src/)
│   ├── main.rs                        # Application entry point
│   │
│   ├── mcp/                           # MCP server implementation
│   │   ├── mod.rs                     # Module exports
│   │   └── stdio_server.rs            # Stdio server with RequestContext
│   │
│   ├── tools/                         # Tool implementations
│   │   ├── mod.rs                     # Tool registry
│   │   ├── shared.rs                  # Shared types (ToolInput, ToolOutput)
│   │   ├── echo.rs                    # Echo tool with validation
│   │   ├── calculator.rs              # Calculate & evaluate tools
│   │   └── advanced.rs                # 6 advanced tools (NEW)
│   │       ├── process_with_progress  # Progress notifications
│   │       ├── batch_process          # Batch operations
│   │       ├── transform_data         # Data transformation
│   │       ├── simulate_upload        # Upload simulation
│   │       ├── health_check           # System health
│   │       └── long_task              # Long-running operation
│   │
│   ├── prompts/                       # Prompt implementations
│   │   ├── mod.rs                     # Prompt registry
│   │   └── code_prompts.rs            # Code-related prompts
│   │
│   ├── resources/                     # Resource implementations
│   │   ├── mod.rs                     # Resource registry
│   │   └── server_resources.rs        # Server resources
│   │
│   ├── transport/                     # Transport layer
│   │   ├── mod.rs                     # Transport module
│   │   └── stdio.rs                   # Stdio transport
│   │
│   ├── middleware/                    # Middleware components
│   │   └── auth.rs                    # JWT authentication
│   │
│   └── utils/                         # Utility modules
│       ├── types.rs                   # Error types (McpError)
│       └── logger.rs                  # Logging setup
│
├── 📚 Documentation (docs/)
│   ├── INDEX.md                       # Documentation index
│   │
│   ├── guides/                        # How-to guides
│   │   ├── QUICK_START.md            # 5-minute setup
│   │   ├── TESTING_GUIDE.md          # Testing all features
│   │   ├── ACTION_PLAN.md            # What to do next
│   │   ├── INSTALLATION.md           # Installation details
│   │   └── GIT_WORKFLOW.md           # Git best practices
│   │
│   ├── reference/                     # Reference documentation
│   │   ├── QUICK_REFERENCE.md        # Fast lookup guide
│   │   ├── API.md                    # API reference
│   │   ├── OUTPUT_SCHEMAS.md         # Tool output schemas
│   │   ├── claude.md                 # AI assistant guide
│   │   ├── CONTRIBUTING.md           # Contribution guidelines
│   │   ├── SECURITY.md               # Security guidelines
│   │   ├── AI_TOOL_PATTERN.md        # AI tool patterns
│   │   ├── PROMPTS_AND_RESOURCES.md  # Prompts & resources
│   │   ├── CODE_ORGANIZATION.md      # Code structure
│   │   └── FILE_SIZE_ENFORCEMENT.md  # File size rules
│   │
│   ├── advanced-features/             # Advanced implementation docs
│   │   ├── SESSION_COMPLETE.md       # Implementation summary
│   │   ├── DEEP_RESEARCH_IMPROVEMENTS.md  # rust-sdk analysis
│   │   ├── VISUAL_SUMMARY.md         # Visual overview
│   │   ├── IMPLEMENTATION_SUMMARY.md # Technical details
│   │   ├── PROTOCOL_UPGRADE_GUIDE.md # Protocol upgrades
│   │   ├── STDIO_WRAPPER_INTEGRATION.md  # Wrapper integration
│   │   ├── MCP_SPEC_REVIEW_SUMMARY.md    # MCP spec review
│   │   └── NATIVE_STDIO_GUIDE.md     # Native stdio guide
│   │
│   ├── integration/                   # Integration guides
│   │   ├── claude_desktop.md         # Claude Desktop setup
│   │   └── mcp_inspector.md          # MCP Inspector setup
│   │
│   ├── troubleshooting/               # Troubleshooting guides
│   │   └── common_issues.md          # Common problems & fixes
│   │
│   └── sessions/                      # Development session notes
│       └── (historical session logs)
│
├── 📝 Examples (examples/)
│   ├── advanced_features_demo.md      # Advanced tools usage examples
│   ├── claude_desktop_config_binary.json      # Binary mode config
│   ├── claude_desktop_config_stdio.json       # Stdio mode config
│   └── claude_desktop_config_http_wrapper.json # HTTP wrapper config
│
├── 🧪 Scripts (scripts/)
│   ├── test_mcp.sh                    # Main MCP protocol tests
│   ├── test_prompts_resources.sh      # Prompts & resources tests
│   ├── test_validation.sh             # Input validation tests
│   ├── test_output_schemas.sh         # Output schema tests
│   ├── test_calculator.sh             # Calculator tool tests
│   ├── test_http.sh                   # HTTP mode tests
│   └── verify_claude_ready.sh         # Pre-flight check (10 checks)
│
├── 🔧 GitHub Workflows (.github/)
│   └── workflows/
│       └── ci.yml                     # CI/CD pipeline
│
└── 🎯 Build Output (target/)
    └── release/
        └── mcp-boilerplate-rust       # Compiled binary (2.4MB)
```

---

## 🎯 Key Directories Explained

### `/src` - Source Code
All Rust source code organized by responsibility:
- **mcp/** - MCP server implementation with RequestContext
- **tools/** - All 11 tools (5 basic + 6 advanced)
- **prompts/** - Prompt templates with icons
- **resources/** - Server resources with annotations
- **transport/** - Communication layer (stdio/HTTP)
- **middleware/** - Cross-cutting concerns (auth, etc.)
- **utils/** - Shared utilities (errors, logging)

### `/docs` - Documentation
Comprehensive documentation organized by purpose:
- **guides/** - Step-by-step how-to guides
- **reference/** - Reference documentation and APIs
- **advanced-features/** - Deep dives and research
- **integration/** - Integration with external tools
- **troubleshooting/** - Problem-solving guides
- **sessions/** - Historical development notes

### `/examples` - Configuration Examples
Ready-to-use configuration files for:
- Claude Desktop integration (3 modes)
- Advanced features demonstration
- Usage examples for all tools

### `/scripts` - Test & Automation Scripts
Automated testing and verification:
- Protocol compliance tests
- Feature verification tests
- Pre-flight checks for Claude Desktop

---

## 📊 File Statistics

| Category | Files | Total Lines |
|----------|-------|-------------|
| Source Code | 15 | ~1,200 |
| Documentation | 29 | ~9,300 |
| Tests/Scripts | 7 | ~800 |
| Examples | 4 | ~600 |
| Config Files | 8 | ~300 |
| **Total** | **63** | **~12,200** |

---

## 🔑 Important Files

### Must Read
1. **START_HERE.md** - Entry point for new users
2. **README.md** - Project overview
3. **docs/INDEX.md** - Documentation navigation

### Configuration
1. **Cargo.toml** - Rust dependencies and metadata
2. **examples/claude_desktop_config_*.json** - Claude Desktop configs
3. **deny.toml** - Security and license checks

### Development
1. **src/main.rs** - Application entry point
2. **src/mcp/stdio_server.rs** - Main server implementation
3. **src/tools/advanced.rs** - Advanced tool examples
4. **docs/reference/claude.md** - AI assistant guide

### Testing
1. **scripts/test_mcp.sh** - Main test suite
2. **scripts/verify_claude_ready.sh** - Pre-flight checks
3. **docs/guides/TESTING_GUIDE.md** - Testing guide

---

## 🎨 Code Organization Principles

### 1. Separation of Concerns
- Each directory has a single, clear responsibility
- Tools, prompts, and resources are separated
- Transport layer isolated from business logic

### 2. Documentation Co-location
- Guides for users in `/docs/guides/`
- Reference for developers in `/docs/reference/`
- Advanced topics in `/docs/advanced-features/`

### 3. Examples & Scripts
- Runnable examples in `/examples/`
- Test scripts in `/scripts/`
- Configuration templates provided

### 4. Clean Root
- Only essential files in root directory
- Most documentation moved to `/docs/`
- Clear entry points (START_HERE.md, README.md)

---

## 🚀 Quick Access Paths

### New Users
```
START_HERE.md
    ↓
docs/guides/QUICK_START.md
    ↓
docs/reference/QUICK_REFERENCE.md
```

### Developers
```
README.md
    ↓
docs/reference/claude.md
    ↓
src/tools/advanced.rs
```

### Testing
```
scripts/test_mcp.sh
    ↓
docs/guides/TESTING_GUIDE.md
    ↓
scripts/verify_claude_ready.sh
```

---

## 📦 Build Artifacts

### Release Build
```
target/release/
└── mcp-boilerplate-rust    # 2.4MB optimized binary
```

### Debug Build
```
target/debug/
└── mcp-boilerplate-rust    # ~50MB with debug symbols
```

---

## 🔍 Finding Files

### By Purpose
- **Getting Started:** START_HERE.md, docs/guides/QUICK_START.md
- **Testing:** docs/guides/TESTING_GUIDE.md, scripts/
- **API Reference:** docs/reference/API.md
- **Security:** docs/reference/SECURITY.md
- **Contributing:** docs/reference/CONTRIBUTING.md

### By Type
- **Markdown:** 32 files (~9,900 lines)
- **Rust:** 15 files (~1,200 lines)
- **JSON:** 4 files (~600 lines)
- **Shell:** 7 files (~800 lines)
- **TOML:** 3 files (~300 lines)

---

## 🧹 Clean Structure Benefits

### Before Cleanup
- 23+ MD files in root directory
- Duplicated documentation
- Unclear navigation
- Hard to find specific docs

### After Cleanup
- 3 MD files in root (README, START_HERE, PROJECT_STRUCTURE)
- Organized by purpose in `/docs/`
- Clear documentation index
- Easy navigation paths

---

## 📝 Maintenance

### Adding New Documentation
1. Determine category (guide/reference/advanced)
2. Place in appropriate `/docs/` subdirectory
3. Update `/docs/INDEX.md`
4. Link from START_HERE.md if important

### Adding New Tools
1. Create tool in `/src/tools/`
2. Register in `/src/tools/mod.rs`
3. Add to server in `/src/mcp/stdio_server.rs`
4. Update docs/reference/QUICK_REFERENCE.md
5. Add examples to examples/advanced_features_demo.md

### Adding Tests
1. Create test script in `/scripts/`
2. Make executable: `chmod +x scripts/new_test.sh`
3. Document in docs/guides/TESTING_GUIDE.md
4. Add to verify_claude_ready.sh if critical

---

## 🏆 Best Practices

### Documentation
- Keep root directory clean (only 3-5 MD files)
- Organize docs by purpose, not by date
- Maintain clear navigation in INDEX.md
- Link related documents

### Code Organization
- One tool per file in `/src/tools/`
- Shared types in `shared.rs`
- Keep files under 500 lines
- Use modules to organize related code

### Testing
- One test script per feature area
- All scripts in `/scripts/` directory
- Make scripts executable
- Document expected output

---

**Maintained by:** NetAdx AI  
**License:** MIT  
**Last Updated:** 2026-01-08