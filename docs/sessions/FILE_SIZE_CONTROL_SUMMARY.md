# File Size Control - Complete Summary

## Critical Rule

**EVERY Rust source file MUST be under 500 lines**

No exceptions. No excuses. Split before committing.

## Quick Check

```bash
# Check all files
make check-size

# Or directly
./scripts/check-file-sizes.sh
```

## Thresholds

| Lines | Status | Action |
|-------|--------|--------|
| 0-400 | ✅ OK | No action needed |
| 401-500 | ⚠️ Warning | Consider splitting soon |
| 501+ | ❌ VIOLATION | MUST split immediately |

## When to Split

Split a file when:
- Approaching 400 lines (warning threshold)
- Multiple distinct responsibilities exist
- Code becomes hard to navigate
- Adding new features would exceed 500 lines

## How to Split - Quick Guide

### Strategy 1: Module Directory (Recommended)

When tool file gets large:

```bash
# Create module directory
mkdir -p src/tools/my_tool

# Split into files
touch src/tools/my_tool/mod.rs           # Main struct (100-200 lines)
touch src/tools/my_tool/handlers.rs      # Action handlers (200-300 lines)
touch src/tools/my_tool/types.rs         # Type definitions (100-200 lines)
```

Example structure:
```
src/tools/user_management/
├── mod.rs           # Tool struct & execute() - 150 lines
├── create.rs        # Create operations - 150 lines
├── read.rs          # Read/List operations - 150 lines
├── update.rs        # Update operations - 120 lines
├── delete.rs        # Delete operations - 100 lines
└── types.rs         # Shared types - 100 lines
```

### Strategy 2: Service Layer

Move complex business logic to services:

```rust
// Tool file: 150 lines
// src/tools/payment.rs
use crate::services::payment_service::PaymentService;

pub struct PaymentTool {
    service: PaymentService,
}

// Service file: 350 lines
// src/services/payment_service.rs
pub struct PaymentService;
impl PaymentService {
    // Complex business logic here
}
```

### Strategy 3: Handler Pattern

Split by feature/handler:

```
src/tools/notification/
├── mod.rs              # Main coordinator - 80 lines
├── handlers/
│   ├── email.rs       # Email notifications - 250 lines
│   ├── sms.rs         # SMS notifications - 200 lines
│   └── push.rs        # Push notifications - 250 lines
└── types.rs           # Shared types - 100 lines
```

## Automated Enforcement

### 1. Pre-Commit Check

```bash
# Setup git hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
./scripts/check-file-sizes.sh
EOF

chmod +x .git/hooks/pre-commit
```

### 2. Make Command

```bash
# Check before commit
make check-size

# Run full check suite
make all  # Includes size check + fmt + lint + test + build
```

### 3. CI/CD Integration

Add to GitHub Actions:

```yaml
- name: Check file sizes
  run: |
    chmod +x scripts/check-file-sizes.sh
    ./scripts/check-file-sizes.sh
```

## Manual Checking

```bash
# Find largest files
find src -name "*.rs" -exec wc -l {} \; | sort -rn | head -10

# Check specific file
wc -l src/tools/my_tool.rs

# Find files over 400 lines
find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 400'

# Find violations (>500 lines)
find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 500'
```

## Refactoring Workflow

1. **Identify**: Check file size
   ```bash
   wc -l src/tools/my_tool.rs
   ```

2. **Plan**: Determine split strategy
   - Action-based? (create, read, update, delete)
   - Service layer? (move logic to services)
   - Handler-based? (split by feature)

3. **Create**: Set up structure
   ```bash
   mkdir -p src/tools/my_tool
   ```

4. **Move**: Split incrementally
   - Move types first
   - Move one handler at a time
   - Update imports after each move

5. **Test**: Verify after each step
   ```bash
   cargo check
   cargo test
   make check-size
   ```

6. **Commit**: Save changes
   ```bash
   git add .
   git commit -m "Refactor: Split my_tool into modules"
   ```

## File Size Targets

| File Type | Max Lines | Recommended | Warning |
|-----------|-----------|-------------|---------|
| Tool main | 500 | 200-300 | 400 |
| Handler | 500 | 200-350 | 400 |
| Service | 500 | 300-450 | 400 |
| Types | 500 | 100-200 | 300 |
| Utils | 500 | 100-200 | 300 |

## Common Patterns

### CRUD Tool Split

```
src/tools/resource/
├── mod.rs        # execute() and routing - 100 lines
├── create.rs     # Create operations - 150 lines
├── read.rs       # Read/Get/List - 150 lines
├── update.rs     # Update operations - 120 lines
├── delete.rs     # Delete operations - 100 lines
└── types.rs      # Shared types - 80 lines
```

### Complex Business Logic Split

```
src/
├── tools/
│   └── payment.rs           # Thin layer - 200 lines
├── services/
│   └── payment_service/
│       ├── mod.rs          # Interface - 100 lines
│       ├── processor.rs    # Processing - 300 lines
│       ├── validator.rs    # Validation - 200 lines
│       └── gateway.rs      # Gateway integration - 400 lines
└── models/
    └── payment.rs          # Data models - 150 lines
```

## Best Practices

1. **Check Before Commit**
   - Always run `make check-size`
   - Fix violations before pushing

2. **Split Early**
   - Don't wait until 500 lines
   - Split at 400 lines (warning threshold)
   - Easier to refactor incrementally

3. **Follow Patterns**
   - Use established splitting strategies
   - Consistent organization across tools
   - See CODE_ORGANIZATION.md for patterns

4. **Keep mod.rs Simple**
   - Main struct definition
   - execute() method with routing
   - Re-exports from sub-modules
   - 100-200 lines maximum

5. **Test After Splitting**
   - Run `cargo check` frequently
   - Ensure tests still pass
   - Update imports correctly

## Documentation References

- **CODE_ORGANIZATION.md** - Complete organization guide (497 lines)
- **FILE_SIZE_ENFORCEMENT.md** - Detailed enforcement guide (535 lines)
- **AI_TOOL_PATTERN.md** - Includes file size control section (850+ lines)
- **TOOL_QUICK_REFERENCE.md** - Quick reference with size rules (370+ lines)

## Quick Commands Reference

```bash
# Check file sizes
make check-size

# Find large files
find src -name "*.rs" -exec wc -l {} \; | sort -rn | head -10

# Check specific file
wc -l src/tools/my_tool.rs

# Watch for changes
find src -name "*.rs" | entr -c make check-size

# Full quality check
make all
```

## Example: Splitting a Large File

### Before (600 lines - VIOLATION)

```rust
// src/tools/user_management.rs - 600 lines
pub struct UserManagementTool;

impl UserManagementTool {
    // 30+ methods in one file
    async fn create_user() { }
    async fn update_user() { }
    async fn delete_user() { }
    async fn get_user() { }
    async fn list_users() { }
    async fn search_users() { }
    // ... many more methods
}
```

### After (Well-Organized)

```rust
// src/tools/user_management/mod.rs - 120 lines
mod create;
mod read;
mod update;
mod delete;
mod types;

pub use types::*;
use create::CreateHandler;
use read::ReadHandler;
use update::UpdateHandler;
use delete::DeleteHandler;

pub struct UserManagementTool {
    create: CreateHandler,
    read: ReadHandler,
    update: UpdateHandler,
    delete: DeleteHandler,
}

impl UserManagementTool {
    pub async fn execute(&self, request: ToolRequest) -> McpResult<ToolResult> {
        match request.action.as_str() {
            "create" => self.create.handle(request).await,
            "get" => self.read.get(request).await,
            "list" => self.read.list(request).await,
            "search" => self.read.search(request).await,
            "update" => self.update.handle(request).await,
            "delete" => self.delete.handle(request).await,
            _ => Err(McpError::InvalidAction(request.action)),
        }
    }
}

// src/tools/user_management/create.rs - 150 lines
pub struct CreateHandler;
impl CreateHandler {
    pub async fn handle(&self, request: ToolRequest) -> McpResult<ToolResult> {
        // Create logic
    }
}

// src/tools/user_management/read.rs - 180 lines
pub struct ReadHandler;
impl ReadHandler {
    pub async fn get(&self, request: ToolRequest) -> McpResult<ToolResult> { }
    pub async fn list(&self, request: ToolRequest) -> McpResult<ToolResult> { }
    pub async fn search(&self, request: ToolRequest) -> McpResult<ToolResult> { }
}

// ... other handlers
```

## Team Workflow

### For Developers
1. Run `make check-size` before every commit
2. Split files at 400 lines (don't wait for 500)
3. Follow established patterns
4. Test after splitting

### For Reviewers
1. Check file sizes in PR
2. Request splits for files >400 lines
3. Verify proper module organization
4. Ensure tests are updated

### For CI/CD
1. Automated size check on every push
2. Block merges with violations
3. Generate size reports
4. Track metrics over time

## Remember

- **500 lines is MAXIMUM** - not a target
- **400 lines is warning** - start planning split
- **No exceptions** - every file must comply
- **Split immediately** - don't accumulate tech debt
- **Follow patterns** - use established strategies
- **Test thoroughly** - ensure nothing breaks

## Summary Checklist

- [ ] File size checked before commit
- [ ] Files under 500 lines (under 400 is better)
- [ ] Large tools split into modules
- [ ] Complex logic moved to services
- [ ] Types extracted to separate files
- [ ] Tests pass after splitting
- [ ] Imports updated correctly
- [ ] Code organized consistently
- [ ] Documentation updated
- [ ] No violations committed

---

**CRITICAL**: Always run `make check-size` before committing!

Version: 1.0.0
Last Updated: 2025-01-08
Author: NetADX MCP Team