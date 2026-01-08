# File Size Enforcement Guide

Automated and manual strategies to enforce 500-line limit per file in MCP Boilerplate Rust.

## Rule

**Every Rust source file MUST be under 500 lines**

## Why This Matters

- Improves code readability
- Enforces modular design
- Easier code navigation
- Simpler testing
- Better maintainability
- Forces separation of concerns

## Automated Enforcement

### 1. Pre-Commit Script

Check file sizes before committing:

```bash
# Run manually
./scripts/check-file-sizes.sh

# Or use make
make check-size
```

### 2. Git Pre-Commit Hook

Setup automatic checking:

```bash
# Create hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
./scripts/check-file-sizes.sh
EOF

# Make executable
chmod +x .git/hooks/pre-commit
```

Now commits will fail if files exceed 500 lines.

### 3. CI/CD Integration

Add to GitHub Actions (`.github/workflows/check.yml`):

```yaml
name: Check Code Quality

on: [push, pull_request]

jobs:
  check-file-sizes:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Check file sizes
        run: |
          chmod +x scripts/check-file-sizes.sh
          ./scripts/check-file-sizes.sh
```

### 4. Makefile Integration

Already integrated:

```bash
make check-size     # Check file sizes
make all            # Includes size check + fmt + lint + test + build
```

## Manual Checking

### Check All Files

```bash
# List all Rust files with line counts
find src -name "*.rs" -type f -exec wc -l {} \; | sort -rn

# Find files over 400 lines (warning threshold)
find src -name "*.rs" -type f -exec wc -l {} \; | awk '$1 > 400'

# Find files over 500 lines (violations)
find src -name "*.rs" -type f -exec wc -l {} \; | awk '$1 > 500'
```

### Check Specific File

```bash
wc -l src/tools/my_tool.rs
```

### Watch File Sizes During Development

```bash
# Install entr (file watcher)
# macOS: brew install entr
# Linux: apt install entr

# Watch and check on changes
find src -name "*.rs" | entr -c ./scripts/check-file-sizes.sh
```

## Thresholds

| Status | Lines | Action |
|--------|-------|--------|
| ✅ OK | 0-400 | No action needed |
| ⚠️ Warning | 401-500 | Consider splitting |
| ❌ Violation | 501+ | MUST split immediately |

## How to Split Files

### Strategy 1: Module Directory

When file approaches 400 lines:

```bash
# Before
src/tools/user_management.rs  # 450 lines

# After
mkdir -p src/tools/user_management
mv src/tools/user_management.rs src/tools/user_management/mod.rs
```

Then split `mod.rs` into smaller files:

```
src/tools/user_management/
├── mod.rs           # 100 lines - main struct & execute()
├── create.rs        # 150 lines
├── read.rs          # 150 lines
└── types.rs         # 100 lines
```

### Strategy 2: Service Layer

Move complex logic to services:

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
    // Complex business logic
}
```

### Strategy 3: Handler Pattern

Split actions into handlers:

```
src/tools/notification/
├── mod.rs              # 80 lines
├── handlers/
│   ├── email.rs       # 250 lines
│   ├── sms.rs         # 200 lines
│   └── push.rs        # 250 lines
└── types.rs           # 100 lines
```

## Splitting Checklist

When file exceeds 400 lines:

- [ ] Identify distinct responsibilities
- [ ] Group related functions
- [ ] Create module directory
- [ ] Move types to `types.rs`
- [ ] Split actions into separate files
- [ ] Update imports
- [ ] Run `cargo check`
- [ ] Run tests
- [ ] Verify size: `make check-size`
- [ ] Commit changes

## Common Patterns

### Pattern 1: CRUD Tool

Split by operation:

```
src/tools/resource/
├── mod.rs        # execute() and routing
├── create.rs     # Create operations
├── read.rs       # Read/Get/List operations
├── update.rs     # Update operations
├── delete.rs     # Delete operations
└── types.rs      # Shared types
```

### Pattern 2: Complex Business Logic

Use service layer:

```
src/
├── tools/
│   └── payment.rs           # 200 lines - thin layer
├── services/
│   └── payment_service/
│       ├── mod.rs          # 100 lines
│       ├── processor.rs    # 300 lines
│       ├── validator.rs    # 200 lines
│       └── gateway.rs      # 400 lines
└── models/
    └── payment.rs          # 150 lines
```

### Pattern 3: Multiple Features

Split by feature:

```
src/tools/content/
├── mod.rs           # Main coordinator
├── posts.rs         # Post management
├── categories.rs    # Category management
├── tags.rs          # Tag management
├── media.rs         # Media handling
└── types.rs         # Shared types
```

## Refactoring Workflow

### Step-by-Step Process

1. **Measure Current Size**
   ```bash
   wc -l src/tools/my_tool.rs
   ```

2. **Identify Responsibilities**
   - List all functions/methods
   - Group by responsibility
   - Find natural boundaries

3. **Create Directory Structure**
   ```bash
   mkdir -p src/tools/my_tool
   ```

4. **Move Code Incrementally**
   - Start with types
   - Move one handler at a time
   - Update imports after each move
   - Test after each move

5. **Verify After Each Step**
   ```bash
   cargo check
   cargo test
   make check-size
   ```

6. **Clean Up**
   - Remove old file
   - Update documentation
   - Commit changes

## File Size Targets

| File Type | Max | Recommended | Warning |
|-----------|-----|-------------|---------|
| Tool main | 500 | 200-300 | 400 |
| Handler | 500 | 200-350 | 400 |
| Service | 500 | 300-450 | 400 |
| Types | 500 | 100-200 | 300 |
| Utils | 500 | 100-200 | 300 |
| Models | 500 | 100-200 | 300 |

## Exception Handling

### When to Exceed 500 Lines?

**NEVER**. There are NO exceptions to the 500-line rule.

If you think you need an exception:
1. Reconsider your design
2. Look for hidden responsibilities
3. Extract interfaces/traits
4. Use composition instead of inheritance
5. Split into smaller modules

### Temporary Violations

During rapid development, you may temporarily exceed 500 lines:

1. **Mark with TODO**
   ```rust
   // TODO: This file has 520 lines, needs splitting
   // Split into: create.rs, read.rs, update.rs, delete.rs
   ```

2. **Create Issue**
   - Document the violation
   - Plan the split
   - Assign to next sprint

3. **Fix Immediately**
   - Don't let violations accumulate
   - Split before adding more code

## Editor Integration

### VS Code Settings

Add to `.vscode/settings.json`:

```json
{
  "editor.rulers": [80, 120],
  "files.maxFileSizeWarning": 25000,
  "todo-tree.highlights.customHighlight": {
    "FILE_SIZE": {
      "icon": "alert",
      "type": "text",
      "foreground": "red"
    }
  }
}
```

### Vim Configuration

Add to `.vimrc`:

```vim
" Show line count in status bar
set statusline+=%L\ lines

" Highlight when file exceeds 400 lines
autocmd BufRead,BufNewFile *.rs if line('$') > 400 | echohl WarningMsg | echo "File has " . line('$') . " lines - consider splitting" | echohl None | endif
```

### Rust Analyzer

Configure in VS Code settings:

```json
{
  "rust-analyzer.diagnostics.disabled": [],
  "rust-analyzer.lens.enable": true
}
```

## Monitoring and Reporting

### Weekly Size Report

Generate report of all files:

```bash
#!/bin/bash
# scripts/weekly-size-report.sh

echo "# File Size Report - $(date)"
echo ""
echo "## Violations (>500 lines)"
find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 500 {print}' | sort -rn

echo ""
echo "## Warnings (400-500 lines)"
find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 400 && $1 <= 500 {print}' | sort -rn

echo ""
echo "## Statistics"
echo "Total files: $(find src -name "*.rs" | wc -l)"
echo "Average size: $(find src -name "*.rs" -exec wc -l {} \; | awk '{sum+=$1; count++} END {print int(sum/count)}')"
```

### Dashboard Script

```bash
#!/bin/bash
# scripts/file-size-dashboard.sh

echo "📊 File Size Dashboard"
echo "====================="
echo ""

TOTAL=$(find src -name "*.rs" | wc -l)
VIOLATIONS=$(find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 500' | wc -l)
WARNINGS=$(find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 400 && $1 <= 500' | wc -l)
OK=$((TOTAL - VIOLATIONS - WARNINGS))

echo "Total files:     $TOTAL"
echo "✅ OK (0-400):   $OK"
echo "⚠️  Warning:     $WARNINGS"
echo "❌ Violations:   $VIOLATIONS"
echo ""

if [ "$VIOLATIONS" -gt 0 ]; then
    echo "🚨 Action Required: $VIOLATIONS file(s) exceed 500 lines"
elif [ "$WARNINGS" -gt 0 ]; then
    echo "⚠️  Consider splitting $WARNINGS file(s) approaching limit"
else
    echo "✅ All files within size limits!"
fi
```

## Best Practices

1. **Check Before Commit**
   ```bash
   make check-size
   ```

2. **Split Early**
   - Don't wait until 500 lines
   - Split at 400 lines
   - Easier to refactor

3. **Plan Structure**
   - Think about organization before coding
   - Design module structure upfront
   - Follow established patterns

4. **Regular Reviews**
   - Weekly file size audit
   - Team reviews for large files
   - Continuous refactoring

5. **Document Decisions**
   - Explain module structure
   - Document split decisions
   - Update architecture docs

## Team Workflow

### For Developers

1. Run `make check-size` before commit
2. Split files approaching 400 lines
3. Use service layer for complex logic
4. Follow CODE_ORGANIZATION.md patterns

### For Code Reviewers

1. Check file sizes in PR
2. Request splits for files >400 lines
3. Verify module organization
4. Ensure tests are updated

### For CI/CD

1. Add size check to pipeline
2. Block merges with violations
3. Generate size reports
4. Track metrics over time

## Quick Reference Commands

```bash
# Check all file sizes
make check-size

# Find large files
find src -name "*.rs" -exec wc -l {} \; | sort -rn | head -10

# Check specific file
wc -l src/tools/my_tool.rs

# Watch for changes
find src -name "*.rs" | entr -c make check-size

# Generate report
./scripts/weekly-size-report.sh

# Dashboard
./scripts/file-size-dashboard.sh
```

## Troubleshooting

### "File too large to split easily"

1. Extract all types first
2. Identify handler boundaries
3. Move to service layer
4. Split incrementally

### "Tests fail after splitting"

1. Update import paths
2. Re-export from mod.rs
3. Check for circular dependencies
4. Run `cargo check` frequently

### "Module structure unclear"

1. Review CODE_ORGANIZATION.md
2. Look at echo tool example
3. Follow established patterns
4. Ask for code review

## Resources

- **CODE_ORGANIZATION.md** - Complete organization guide
- **AI_TOOL_PATTERN.md** - Tool development patterns
- **TOOL_QUICK_REFERENCE.md** - Quick reference
- `scripts/check-file-sizes.sh` - Size checking script
- `src/tools/echo.rs` - Example of well-sized file

## Summary

- Every file MUST be under 500 lines
- Split files at 400 lines (warning threshold)
- Use automated checking: `make check-size`
- Follow splitting strategies in CODE_ORGANIZATION.md
- No exceptions to the 500-line rule
- Fix violations immediately

---

Version: 1.0.0
Last Updated: 2025-01-08
Author: NetADX MCP Team