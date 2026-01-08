# MCP Boilerplate Rust - Documentation Index

Complete documentation for MCP v5 Rust boilerplate project.

## Quick Links

- [5-Minute Quick Start](../QUICKSTART.md)
- [Complete API Reference](API.md)
- [AI Tool Development Pattern](AI_TOOL_PATTERN.md)
- [Tool Quick Reference](TOOL_QUICK_REFERENCE.md)
- [Code Organization Guide](CODE_ORGANIZATION.md)
- [File Size Enforcement](FILE_SIZE_ENFORCEMENT.md)

## Getting Started

### For New Users
1. **[QUICKSTART.md](../QUICKSTART.md)** - Get started in 5 minutes
   - Prerequisites
   - Setup steps
   - First test
   - Expected output

2. **[INSTALLATION.md](../INSTALLATION.md)** - Complete installation guide
   - Detailed setup
   - Configuration
   - Verification
   - Troubleshooting

### For Developers
3. **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Development guide
   - Code style
   - Adding new tools
   - Testing
   - Pull requests

4. **[PROJECT_OVERVIEW.md](../PROJECT_OVERVIEW.md)** - Architecture overview
   - Project structure
   - Core components
   - Technology stack
   - Design philosophy

5. **[CODE_ORGANIZATION.md](CODE_ORGANIZATION.md)** - Code organization
   - File size limits (500 lines max)
   - Splitting strategies
   - Module patterns
   - Best practices

6. **[FILE_SIZE_ENFORCEMENT.md](FILE_SIZE_ENFORCEMENT.md)** - Size enforcement
   - Automated checking
   - Manual verification
   - Splitting workflows
   - Team processes

## Core Documentation

### API Documentation
**[API.md](API.md)** - Complete API reference (460 lines)
- Base URL and endpoints
- Request/response formats
- Health check endpoints
- Tool endpoints (Echo tool)
- Error responses
- HTTP status codes
- Authentication
- Examples in multiple languages

### Tool Development

**[AI_TOOL_PATTERN.md](AI_TOOL_PATTERN.md)** - AI instruction pattern (850+ lines)

Complete guide for AI assistants to create new tools:
- Project structure
- Tool development pattern (4 steps)
- Parameter extraction patterns
- Error handling patterns
- Response patterns
- Authentication patterns
- Database integration patterns
- Service integration pattern
- Testing pattern
- **File size control (CRITICAL)** - Keep files under 500 lines
- Complete example (User Management Tool)
- Checklist for new tools

**[TOOL_QUICK_REFERENCE.md](TOOL_QUICK_REFERENCE.md)** - Quick reference (370+ lines)

Fast lookup for common patterns:
- **File size limit rule** - Max 500 lines per file
- Minimal tool template
- Register tool
- Add route and handler
- Parameter extraction shortcuts
- Error types
- Response format
- Multiple actions
- File size control strategies
- Testing commands
- Common CRUD patterns
- Checklist

**[CODE_ORGANIZATION.md](CODE_ORGANIZATION.md)** - Code organization guide (497 lines)

Best practices for maintaining code under 500 lines:
- File organization strategies
- Module organization patterns
- When to split files
- Splitting checklist
- Action-based modules
- Service-based organization
- Handler pattern
- Example refactoring workflows
- Naming conventions
- Tools registration patterns

**[FILE_SIZE_ENFORCEMENT.md](FILE_SIZE_ENFORCEMENT.md)** - Enforcement guide (535 lines)

Automated and manual strategies to enforce file size limits:
- Automated enforcement (pre-commit hooks, CI/CD)
- Manual checking commands
- Thresholds and actions
- How to split files
- Splitting checklist
- Common patterns
- Refactoring workflow
- File size targets
- Exception handling (NONE - no exceptions!)
- Editor integration
- Monitoring and reporting
- Team workflow
- Quick reference commands

## Project Files

### Configuration
- **Cargo.toml** - Rust dependencies and features
- **.env.example** - Environment variables template
- **.gitignore** - Git ignore rules
- **LICENSE** - MIT License

### Docker
- **Dockerfile** - Multi-stage production build
- **docker-compose.yml** - Full setup with MongoDB

### Development Tools
- **Makefile** - 20+ convenient commands
- **run.sh** - Server run script (dev/prod/watch)
- **test.sh** - Endpoint testing script
- **verify-setup.sh** - Setup verification script

## Source Code Structure

```
src/
├── main.rs              # Server entry point, routes, handlers
├── types.rs             # Core types, errors, request/response
├── utils/
│   ├── mod.rs          # Utils module exports
│   ├── config.rs       # Configuration helper
│   └── logger.rs       # Logging utility
├── middleware/
│   ├── mod.rs          # Middleware exports
│   └── auth.rs         # JWT authentication
├── services/
│   └── mod.rs          # Business logic services (placeholder)
├── models/
│   └── mod.rs          # Data models (placeholder)
└── tools/
    ├── mod.rs          # Tools module exports
    └── echo.rs         # Sample echo tool (3 actions)
```

## Documentation by Topic

### Setup & Installation
- [QUICKSTART.md](../QUICKSTART.md) - 5-minute guide
- [INSTALLATION.md](../INSTALLATION.md) - Complete installation
- [README.md](../README.md) - Full project documentation

### Development
- [CONTRIBUTING.md](../CONTRIBUTING.md) - How to contribute
- [AI_TOOL_PATTERN.md](AI_TOOL_PATTERN.md) - Tool development pattern
- [TOOL_QUICK_REFERENCE.md](TOOL_QUICK_REFERENCE.md) - Quick reference

### Architecture
- [PROJECT_OVERVIEW.md](../PROJECT_OVERVIEW.md) - Project architecture
- [API.md](API.md) - API documentation

### Reference
- [README.md](../README.md) - Main documentation
- This file (INDEX.md) - Documentation index

## Key Features

### Core Features
- Axum HTTP server with Tokio runtime
- CORS middleware
- Structured logging with tracing
- Type-safe request/response handling
- Comprehensive error handling
- Environment configuration

### Optional Features
- JWT authentication (optional)
- MongoDB support (optional)
- Docker support
- Auto-reload for development

### Development Features
- Makefile with 20+ commands
- Test scripts
- Setup verification
- Hot reload with cargo-watch
- Docker Compose setup

## Quick Commands

```bash
# Setup
make setup

# Run
make run

# Test
make test-curl

# Development
make dev

# Help
make help
```

## Examples

### Echo Tool Actions
1. **Echo** - Echo back a message
2. **Ping** - Simple ping-pong test
3. **Info** - Get tool information

### Creating New Tool
See [AI_TOOL_PATTERN.md](AI_TOOL_PATTERN.md) for complete guide.

Quick steps:
1. Create `src/tools/my_tool.rs`
2. Register in `src/tools/mod.rs`
3. Add route in `src/main.rs`
4. Add handler in `src/main.rs`
5. Test with curl

## Testing

### Manual Testing
```bash
# Health check
curl http://localhost:8025/health

# Echo test
curl -X POST http://localhost:8025/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"action":"ping"}'
```

### Test Script
```bash
./test.sh
```

### Verification
```bash
./verify-setup.sh
```

## Error Types

- `ToolExecution` - Tool execution errors
- `InvalidAction` - Unknown action
- `MissingParameter` - Required parameter missing
- `Database` - Database errors
- `Authentication` - Auth errors
- `Internal` - Internal server errors

## Response Format

All responses follow MCP v5 standard:

```json
{
  "success": true,
  "data": { ... },
  "metadata": {
    "executionTime": 10,
    "timestamp": "2025-01-08T10:30:00Z"
  }
}
```

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `HOST` | 0.0.0.0 | Server bind address |
| `PORT` | 8025 | Server port |
| `RUST_LOG` | info | Log level |
| `MONGODB_URI` | - | MongoDB connection (optional) |
| `JWT_SECRET` | - | JWT secret (optional) |

## Support & Resources

### Documentation
- All docs in `docs/` directory
- Examples in `src/tools/echo.rs`
- Test scripts in root directory

### File Size Control
```bash
make check-size
./scripts/check-file-sizes.sh
```

### Verification
```bash
./verify-setup.sh
```

### Troubleshooting
- Check logs: `RUST_LOG=debug cargo run`
- Health check: `curl http://localhost:8025/health`
- Test suite: `./test.sh`
- Check file sizes: `make check-size`

### External Resources
- Rust: https://doc.rust-lang.org/
- Axum: https://docs.rs/axum/
- Tokio: https://tokio.rs/

## Document Statistics

Total documentation: ~3,600+ lines
- README.md: 416 lines
- QUICKSTART.md: 262 lines
- CONTRIBUTING.md: 254 lines
- PROJECT_OVERVIEW.md: 292 lines
- INSTALLATION.md: 388 lines
- API.md: 460 lines
- AI_TOOL_PATTERN.md: 850+ lines
- TOOL_QUICK_REFERENCE.md: 370+ lines
- CODE_ORGANIZATION.md: 497 lines
- FILE_SIZE_ENFORCEMENT.md: 535 lines
- INDEX.md: 313+ lines

## Version

- Project Version: 0.1.0
- MCP Protocol: v5
- Documentation Version: 1.0.0
- Last Updated: 2025-01-08

## License

MIT License - See [LICENSE](../LICENSE)

## Author

NetADX MCP Team

---

**Note**: This is a living document. As the project evolves, documentation will be updated.

For the most up-to-date information, always check the latest version of each document.