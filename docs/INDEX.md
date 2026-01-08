# Documentation Index

**MCP Boilerplate Rust v0.4.0**  
**Last Updated:** 2026-01-09 (HCMC Timezone)

---

## Quick Navigation

### New Users Start Here
- **[../START_HERE.md](../START_HERE.md)** - Main entry point (5 min read)
- **[guides/QUICK_START.md](guides/QUICK_START.md)** - Get running in 5 minutes
- **[reference/QUICK_REFERENCE.md](reference/QUICK_REFERENCE.md)** - Fast lookup guide

### Testing & Usage
- **[guides/TESTING_GUIDE.md](guides/TESTING_GUIDE.md)** - Comprehensive testing guide
- **[guides/ACTION_PLAN.md](guides/ACTION_PLAN.md)** - Step-by-step next actions
- **[../examples/advanced_features_demo.md](../examples/advanced_features_demo.md)** - Tool usage examples

### Advanced Features
- **[advanced-features/SESSION_COMPLETE.md](advanced-features/SESSION_COMPLETE.md)** - Implementation summary
- **[advanced-features/DEEP_RESEARCH_IMPROVEMENTS.md](advanced-features/DEEP_RESEARCH_IMPROVEMENTS.md)** - Complete rust-sdk analysis
- **[advanced-features/VISUAL_SUMMARY.md](advanced-features/VISUAL_SUMMARY.md)** - Visual overview
- **[advanced-features/IMPLEMENTATION_SUMMARY.md](advanced-features/IMPLEMENTATION_SUMMARY.md)** - Technical details

---

## Documentation Structure

```
docs/
├── INDEX.md (this file)
├── PROJECT_STRUCTURE.md             # Complete project structure
│
├── guides/                          # How-to guides
│   ├── QUICK_START.md              # 5-minute setup
│   ├── TESTING_GUIDE.md            # Testing all features
│   ├── ACTION_PLAN.md              # What to do next
│   ├── INSTALLATION.md             # Installation guide
│   └── GIT_WORKFLOW.md             # Git best practices
│
├── reference/                       # Reference documentation
│   ├── QUICK_REFERENCE.md          # Fast lookup
│   ├── API.md                      # API reference
│   ├── OUTPUT_SCHEMAS.md           # Tool output schemas
│   ├── claude.md                   # AI assistant guide
│   ├── CONTRIBUTING.md             # How to contribute
│   ├── SECURITY.md                 # Security guidelines
│   ├── AI_TOOL_PATTERN.md          # AI tool patterns
│   ├── PROMPTS_AND_RESOURCES.md    # Prompt & resource reference
│   ├── CODE_ORGANIZATION.md        # Code structure
│   └── FILE_SIZE_ENFORCEMENT.md    # File size rules
│
├── advanced-features/               # Advanced implementation docs
│   ├── SESSION_COMPLETE.md         # Implementation summary
│   ├── DEEP_RESEARCH_IMPROVEMENTS.md # rust-sdk analysis
│   ├── VISUAL_SUMMARY.md           # Visual overview
│   ├── IMPLEMENTATION_SUMMARY.md   # Technical details
│   ├── PROTOCOL_UPGRADE_GUIDE.md   # Protocol upgrades
│   ├── STDIO_WRAPPER_INTEGRATION.md # Wrapper integration
│   ├── MCP_SPEC_REVIEW_SUMMARY.md  # MCP spec review
│   └── NATIVE_STDIO_GUIDE.md       # Native stdio guide
│
├── integration/                     # Integration guides
│   ├── INTEGRATION_GUIDE.md        # Complete integration guide
│   ├── CLAUDE_DESKTOP_SETUP.md     # Claude Desktop setup
│   ├── HTTP_WRAPPER_INTEGRATION.md # HTTP wrapper setup
│   └── START_TESTING_NOW.md        # Quick testing start
│
├── troubleshooting/                 # Troubleshooting guides
│   ├── COMMON_ISSUES.md            # Consolidated troubleshooting
│   ├── FIX_ANSI_ESCAPE_CODES.md    # ANSI code fix
│   ├── FIX_ESM_REQUIRE.md          # ESM/Node issues
│   ├── FIX_NODE_VERSION.md         # Node version fix
│   └── TROUBLESHOOTING_JSON_ERROR.md # JSON error fixes
│
└── archive/                         # Historical documents
    └── sessions/                    # Development session notes
        └── (17 historical session logs)
```

---

## 🎯 Documentation by Purpose

### Getting Started
1. [../START_HERE.md](../START_HERE.md) - Main entry point
2. [guides/QUICK_START.md](guides/QUICK_START.md) - Quick setup
3. [guides/INSTALLATION.md](guides/INSTALLATION.md) - Installation details
4. [integration/INTEGRATION_GUIDE.md](integration/INTEGRATION_GUIDE.md) - Complete integration guide

### Learning the System
1. [reference/QUICK_REFERENCE.md](reference/QUICK_REFERENCE.md) - Quick lookup
2. [reference/API.md](reference/API.md) - API reference
3. [reference/claude.md](reference/claude.md) - AI assistant guide
4. [advanced-features/VISUAL_SUMMARY.md](advanced-features/VISUAL_SUMMARY.md) - Visual overview

### Testing & Development
1. [guides/TESTING_GUIDE.md](guides/TESTING_GUIDE.md) - Testing guide
2. [guides/ACTION_PLAN.md](guides/ACTION_PLAN.md) - Action plan
3. [../examples/advanced_features_demo.md](../examples/advanced_features_demo.md) - Examples
4. [integration/INTEGRATION_GUIDE.md](integration/INTEGRATION_GUIDE.md) - All integration methods

### Advanced Topics
1. [advanced-features/DEEP_RESEARCH_IMPROVEMENTS.md](advanced-features/DEEP_RESEARCH_IMPROVEMENTS.md) - rust-sdk analysis
2. [advanced-features/SESSION_COMPLETE.md](advanced-features/SESSION_COMPLETE.md) - Implementation summary
3. [advanced-features/IMPLEMENTATION_SUMMARY.md](advanced-features/IMPLEMENTATION_SUMMARY.md) - Technical details
4. [advanced-features/PROTOCOL_UPGRADE_GUIDE.md](advanced-features/PROTOCOL_UPGRADE_GUIDE.md) - Protocol upgrades

### Contributing
1. [reference/CONTRIBUTING.md](reference/CONTRIBUTING.md) - Contribution guidelines
2. [guides/GIT_WORKFLOW.md](guides/GIT_WORKFLOW.md) - Git workflow
3. [reference/CODE_ORGANIZATION.md](reference/CODE_ORGANIZATION.md) - Code structure
4. [reference/SECURITY.md](reference/SECURITY.md) - Security guidelines

### Reference
1. [reference/QUICK_REFERENCE.md](reference/QUICK_REFERENCE.md) - Quick lookup
2. [reference/OUTPUT_SCHEMAS.md](reference/OUTPUT_SCHEMAS.md) - Output schemas
3. [reference/AI_TOOL_PATTERN.md](reference/AI_TOOL_PATTERN.md) - AI patterns
4. [reference/PROMPTS_AND_RESOURCES.md](reference/PROMPTS_AND_RESOURCES.md) - Prompts & resources

---

## 🎓 Learning Paths

### Path 1: Quick User (30 minutes)
```
START_HERE.md (5 min)
    ↓
reference/QUICK_REFERENCE.md (5 min)
    ↓
guides/QUICK_START.md (5 min)
    ↓
Test with MCP Inspector (15 min)
```

### Path 2: Developer (2 hours)
```
START_HERE.md (5 min)
    ↓
guides/TESTING_GUIDE.md (30 min)
    ↓
reference/claude.md (20 min)
    ↓
examples/advanced_features_demo.md (30 min)
    ↓
Build custom tools (45 min)
```

### Path 3: Advanced (4+ hours)
```
START_HERE.md (5 min)
    ↓
advanced-features/SESSION_COMPLETE.md (30 min)
    ↓
advanced-features/DEEP_RESEARCH_IMPROVEMENTS.md (2 hours)
    ↓
Review rust-sdk source (1+ hours)
    ↓
Implement advanced features
```

---

## 📊 Document Statistics

| Category | Files | Total Lines |
|----------|-------|-------------|
| Root Documentation | 2 | ~700 |
| Guides | 5 | ~1,800 |
| Reference | 10 | ~2,200 |
| Advanced Features | 8 | ~3,400 |
| Integration | 4 | ~1,200 |
| Troubleshooting | 5 | ~1,500 |
| Archive/Sessions | 17 | ~3,000 |
| **Total** | **51** | **~13,800** |

---

## 🔍 Find Documentation By Topic

### Progress Notifications
- [advanced-features/SESSION_COMPLETE.md](advanced-features/SESSION_COMPLETE.md)
- [advanced-features/IMPLEMENTATION_SUMMARY.md](advanced-features/IMPLEMENTATION_SUMMARY.md)
- [../examples/advanced_features_demo.md](../examples/advanced_features_demo.md)

### RequestContext
- [reference/claude.md](reference/claude.md)
- [advanced-features/DEEP_RESEARCH_IMPROVEMENTS.md](advanced-features/DEEP_RESEARCH_IMPROVEMENTS.md)
- [reference/API.md](reference/API.md)

### Tools
- [reference/QUICK_REFERENCE.md](reference/QUICK_REFERENCE.md)
- [../examples/advanced_features_demo.md](../examples/advanced_features_demo.md)
- [guides/TESTING_GUIDE.md](guides/TESTING_GUIDE.md)

### Testing
- [guides/TESTING_GUIDE.md](guides/TESTING_GUIDE.md)
- [guides/ACTION_PLAN.md](guides/ACTION_PLAN.md)
- [integration/INTEGRATION_GUIDE.md](integration/INTEGRATION_GUIDE.md)

### Security
- [reference/SECURITY.md](reference/SECURITY.md)
- [reference/CONTRIBUTING.md](reference/CONTRIBUTING.md)

### Architecture
- [reference/CODE_ORGANIZATION.md](reference/CODE_ORGANIZATION.md)
- [advanced-features/VISUAL_SUMMARY.md](advanced-features/VISUAL_SUMMARY.md)
- [reference/claude.md](reference/claude.md)

---

## 📝 Recently Updated

- **2026-01-08:** Second cleanup - streamlined structure
- **2026-01-08:** Consolidated troubleshooting and integration guides
- **2026-01-08:** Archived historical session notes
- **2026-01-08:** Created comprehensive navigation

---

## 🆘 Need Help?

**Can't find what you need?**
1. Check [../START_HERE.md](../START_HERE.md) first
2. Use the search paths above
3. Check [troubleshooting/COMMON_ISSUES.md](troubleshooting/COMMON_ISSUES.md)
4. Review [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)
5. Contact: hello@netadx.ai

---

## 🔗 External Resources

- **MCP Specification:** https://modelcontextprotocol.io
- **Rust SDK:** https://github.com/modelcontextprotocol/rust-sdk
- **Project Repository:** https://github.com/netadx1ai/mcp-boilerplate-rust
- **Website:** https://netadx.ai

---

**Maintained by:** NetAdx AI  
**License:** MIT  
**Last Reviewed:** 2026-01-08