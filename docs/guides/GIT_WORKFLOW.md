# Git Workflow and GitHub Best Practices

**MCP Boilerplate Rust**  
**Last Updated:** 2026-01-08  
**Status:** Recommended Practices

---

## Overview

This document outlines the recommended Git workflow and GitHub project management practices for contributing to MCP Boilerplate Rust. Following these practices ensures code quality, collaboration efficiency, and maintainable project history.

## Core Principles

1. **Never commit directly to main**
2. **Always work in feature branches**
3. **Use Pull Requests for code review**
4. **Keep commits atomic and meaningful**
5. **Write clear commit messages**
6. **Test before pushing**

---

## Branch Strategy

### Branch Types

**main**
- Production-ready code only
- Protected branch
- Requires PR approval
- CI/CD must pass

**development**
- Integration branch
- Pre-production testing
- Merge target for features
- Optional but recommended

**feature/***
- New features
- Branch from: main or development
- Merge to: development or main
- Example: `feature/add-calculator-tool`

**fix/***
- Bug fixes
- Branch from: main
- Merge to: main
- Example: `fix/echo-validation-error`

**docs/***
- Documentation updates
- Branch from: main
- Merge to: main
- Example: `docs/update-security-guide`

**refactor/***
- Code refactoring
- Branch from: main
- Merge to: development or main
- Example: `refactor/simplify-error-types`

**test/***
- Test improvements
- Branch from: main
- Merge to: main
- Example: `test/add-integration-tests`

### Branch Naming Convention

```
<type>/<short-description>

Examples:
- feature/http-streaming
- fix/ansi-logging-issue
- docs/claude-integration
- refactor/tool-handler
- test/validation-suite
```

**Rules:**
- Lowercase only
- Hyphen-separated words
- Descriptive but concise
- No issue numbers in name (use PR description)

---

## Workflow Process

### 1. Starting New Work

```bash
# Update local main
git checkout main
git pull origin main

# Create feature branch
git checkout -b feature/new-tool

# Verify branch
git branch
```

### 2. Making Changes

```bash
# Make changes to files
vim src/tools/new_tool.rs

# Check status
git status

# Stage changes
git add src/tools/new_tool.rs

# Commit with clear message
git commit -m "feat: Add new calculation tool with validation"

# Continue working...
git add tests/test_calculator.sh
git commit -m "test: Add calculator tool test suite"
```

### 3. Pushing to Remote

```bash
# Push feature branch
git push origin feature/new-tool

# If branch doesn't exist on remote yet
git push -u origin feature/new-tool
```

### 4. Creating Pull Request

**On GitHub:**
1. Navigate to repository
2. Click "Pull Requests" tab
3. Click "New Pull Request"
4. Select base: `main`, compare: `feature/new-tool`
5. Fill in PR template
6. Request reviewers
7. Link related issues
8. Submit PR

### 5. Code Review

**As Author:**
- Respond to review comments
- Make requested changes
- Push updates to same branch
- Mark conversations as resolved

**As Reviewer:**
- Review code thoroughly
- Test locally if needed
- Provide constructive feedback
- Approve or request changes

### 6. Merging

```bash
# After PR approval and CI passing
# On GitHub: Click "Merge Pull Request"
# Select merge strategy: "Squash and merge" (recommended)

# Update local main
git checkout main
git pull origin main

# Delete feature branch
git branch -d feature/new-tool
git push origin --delete feature/new-tool
```

---

## Commit Message Convention

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, no logic change)
- **refactor**: Code refactoring
- **test**: Adding or updating tests
- **chore**: Build process, dependencies, tooling
- **perf**: Performance improvements
- **ci**: CI/CD configuration changes

### Examples

**Simple feature:**
```
feat: Add calculator tool for arithmetic operations
```

**Bug fix with details:**
```
fix: Prevent echo tool from accepting empty messages

The echo tool was not validating for empty strings, causing
undefined behavior. Added validation to reject empty messages
and return appropriate error.

Closes #42
```

**Documentation update:**
```
docs: Update Claude Desktop integration guide

- Add troubleshooting section
- Include macOS and Windows paths
- Add performance benchmarks
```

**Breaking change:**
```
feat!: Change echo tool response format

BREAKING CHANGE: Echo tool now returns object with 'message'
and 'timestamp' fields instead of plain string. Update clients
to use response.message instead of response directly.
```

### Rules

1. **Subject line:**
   - 50 characters or less
   - Lowercase type and scope
   - No period at end
   - Imperative mood ("Add" not "Added")

2. **Body:**
   - Wrap at 72 characters
   - Explain what and why, not how
   - Separate from subject with blank line

3. **Footer:**
   - Reference issues: `Closes #123`
   - Breaking changes: `BREAKING CHANGE:`
   - Co-authors: `Co-authored-by: Name <email>`

---

## Pull Request Best Practices

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests passing
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings
- [ ] Tests pass locally

## Related Issues
Closes #123
Relates to #456
```

### PR Title Format

```
<type>: <description>

Examples:
- feat: Add HTTP streaming support
- fix: Resolve ANSI escape code issue
- docs: Improve security documentation
```

### PR Description Guidelines

1. **Clear summary** - What does this PR do?
2. **Motivation** - Why is this change needed?
3. **Implementation** - How was it implemented?
4. **Testing** - How was it tested?
5. **Screenshots** - If UI changes (N/A for this project)
6. **Breaking changes** - Clearly marked
7. **Related issues** - Link all related issues

### PR Size

**Ideal PR:**
- Changes: 100-400 lines
- Files: 1-10 files
- Scope: Single feature or fix
- Review time: 15-30 minutes

**Too large:**
- Changes: >1000 lines
- Files: >20 files
- Multiple unrelated changes
- Split into smaller PRs

---

## Code Review Process

### Reviewer Checklist

**Code Quality:**
- [ ] Follows Rust style guidelines
- [ ] No clippy warnings
- [ ] Proper error handling
- [ ] Input validation present
- [ ] No code duplication

**Testing:**
- [ ] Tests added for new code
- [ ] Edge cases covered
- [ ] All tests passing
- [ ] No test coverage reduction

**Documentation:**
- [ ] Code comments where needed
- [ ] README updated if needed
- [ ] API docs updated
- [ ] CHANGELOG updated

**Security:**
- [ ] No security vulnerabilities
- [ ] Input sanitization
- [ ] No hardcoded secrets
- [ ] Dependencies reviewed

**Performance:**
- [ ] No obvious performance issues
- [ ] Resource usage acceptable
- [ ] No memory leaks
- [ ] Benchmarks updated

### Review Comments

**Good comments:**
```
"Consider using pattern matching here for better readability"
"This could panic if the vector is empty - add bounds checking"
"Great error handling! Clear and informative messages"
```

**Avoid:**
```
"This is wrong" (not constructive)
"I would do it differently" (not specific)
"Please fix" (no explanation)
```

### Approval Process

1. **At least 1 approval required** (configure in GitHub)
2. **All conversations resolved**
3. **CI/CD passing**
4. **No merge conflicts**
5. **Up to date with base branch**

---

## GitHub Project Management

### Issues

**Issue Types:**
- Bug Report
- Feature Request
- Documentation
- Question
- Enhancement

**Issue Template Example:**

```markdown
### Bug Report

**Description:**
Clear description of the bug

**Steps to Reproduce:**
1. Step one
2. Step two
3. See error

**Expected Behavior:**
What should happen

**Actual Behavior:**
What actually happens

**Environment:**
- OS: macOS 14.1
- Rust: 1.88.0
- Version: 0.3.1

**Additional Context:**
Any other relevant information
```

**Issue Labels:**
- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Documentation improvements
- `good first issue` - Good for newcomers
- `help wanted` - Extra attention needed
- `priority:high` - High priority
- `priority:low` - Low priority
- `status:in-progress` - Being worked on
- `status:blocked` - Blocked by something

### Projects

**Project Board Columns:**

1. **Backlog**
   - New issues
   - Not prioritized
   - Future work

2. **To Do**
   - Prioritized
   - Ready to work
   - Assigned or unassigned

3. **In Progress**
   - Actively being worked
   - Has assignee
   - Has branch/PR

4. **Review**
   - PR submitted
   - Awaiting review
   - CI running

5. **Done**
   - Merged
   - Deployed
   - Closed

**Using Projects:**

```bash
# Link PR to project
# In PR description:
Projects: #1

# Link issue to project
# In issue description:
Projects: #1
```

### Milestones

**Example Milestones:**
- v0.4.0 - Next Minor Release
- v1.0.0 - Stable Release
- Documentation Improvements
- Performance Optimization

**Milestone Planning:**
1. Create milestone with due date
2. Assign issues to milestone
3. Track progress
4. Close when complete

### Releases

**Release Process:**

1. **Version Bump:**
   ```bash
   # Update Cargo.toml
   version = "0.4.0"
   
   # Commit
   git commit -am "chore: Bump version to 0.4.0"
   ```

2. **Create Tag:**
   ```bash
   git tag -a v0.4.0 -m "Release v0.4.0"
   git push origin v0.4.0
   ```

3. **GitHub Release:**
   - Go to Releases
   - Click "Create a new release"
   - Select tag v0.4.0
   - Add release notes
   - Publish release

**Release Notes Format:**

```markdown
# v0.4.0 - 2026-01-15

## Features
- Add calculator tool (#45)
- Add HTTP streaming support (#52)

## Bug Fixes
- Fix echo validation error (#48)
- Resolve memory leak in HTTP mode (#51)

## Documentation
- Update Claude integration guide (#49)
- Add troubleshooting section (#50)

## Breaking Changes
- None

## Contributors
Thanks to @user1, @user2 for contributions!
```

---

## CI/CD Configuration

### GitHub Actions Workflow

**Recommended workflows:**

**.github/workflows/ci.yml:**
```yaml
name: CI

on:
  pull_request:
    branches: [main, development]
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check
```

**.github/workflows/security.yml:**
```yaml
name: Security Audit

on:
  schedule:
    - cron: '0 0 * * 0' # Weekly
  pull_request:

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Security audit
        run: cargo audit
```

### Branch Protection

**Settings > Branches > Branch protection rules:**

**For main branch:**
- [x] Require pull request reviews (1)
- [x] Require status checks to pass
  - [x] CI tests
  - [x] Clippy
  - [x] Security audit
- [x] Require branches to be up to date
- [x] Require conversation resolution
- [x] Include administrators
- [ ] Allow force pushes (disabled)
- [ ] Allow deletions (disabled)

---

## Local Development Workflow

### Daily Workflow

```bash
# Morning: Update local main
git checkout main
git pull origin main

# Start new feature
git checkout -b feature/my-feature

# Work and commit
git add .
git commit -m "feat: Add initial implementation"

# Continue working
# ... make changes ...
git commit -am "feat: Add tests for new feature"

# Push to remote
git push origin feature/my-feature

# Create PR on GitHub
# ... wait for review ...

# After merge: Clean up
git checkout main
git pull origin main
git branch -d feature/my-feature
```

### Keeping Branch Updated

```bash
# While working on feature branch
git checkout feature/my-feature

# Update from main
git fetch origin
git rebase origin/main

# Or merge if rebase is problematic
git merge origin/main

# Push updated branch
git push origin feature/my-feature --force-with-lease
```

### Handling Conflicts

```bash
# When conflicts occur during rebase
git rebase origin/main

# Fix conflicts in files
vim conflicted-file.rs

# Mark as resolved
git add conflicted-file.rs

# Continue rebase
git rebase --continue

# Or abort if needed
git rebase --abort
```

---

## Quality Gates

### Pre-Commit Checks

```bash
# Run before committing
cargo fmt
cargo clippy -- -D warnings
cargo test
./scripts/test_mcp.sh
./scripts/test_validation.sh
```

### Pre-Push Checks

```bash
# Run before pushing
cargo build --release
cargo build --release --features http
cargo audit
./scripts/verify_claude_ready.sh
```

### Pre-Merge Checks

```bash
# Run before merging PR
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt -- --check
cargo audit
```

---

## Common Scenarios

### Scenario 1: Adding New Tool

```bash
# 1. Create feature branch
git checkout -b feature/add-math-tool

# 2. Implement tool
vim src/tools/math.rs

# 3. Add tests
vim scripts/test_math.sh
chmod +x scripts/test_math.sh

# 4. Update documentation
vim README.md
vim docs/API.md

# 5. Commit changes
git add src/tools/math.rs scripts/test_math.sh README.md docs/API.md
git commit -m "feat: Add math tool for calculations"

# 6. Test
cargo test
./scripts/test_math.sh

# 7. Push and create PR
git push origin feature/add-math-tool
```

### Scenario 2: Fixing Bug

```bash
# 1. Create fix branch from issue
git checkout -b fix/echo-empty-message

# 2. Fix the bug
vim src/tools/echo.rs

# 3. Add test for regression
vim scripts/test_validation.sh

# 4. Commit
git commit -am "fix: Prevent echo tool accepting empty messages

Closes #42"

# 5. Test
cargo test
./scripts/test_validation.sh

# 6. Push and create PR
git push origin fix/echo-empty-message
```

### Scenario 3: Updating Documentation

```bash
# 1. Create docs branch
git checkout -b docs/improve-claude-guide

# 2. Update documentation
vim claude.md

# 3. Commit
git commit -am "docs: Add troubleshooting section to Claude guide"

# 4. Push and create PR
git push origin docs/improve-claude-guide
```

---

## Best Practices Summary

### Do's

- Work in feature branches
- Write clear commit messages
- Test before pushing
- Request code review
- Keep PRs focused and small
- Update documentation
- Link issues in PRs
- Resolve all conversations
- Squash commits when merging
- Delete merged branches

### Don'ts

- Commit directly to main
- Force push to shared branches
- Commit untested code
- Include unrelated changes
- Create giant PRs
- Ignore review feedback
- Merge without approval
- Leave conflicts unresolved
- Commit sensitive data
- Skip CI checks

---

## Tools and Resources

### Recommended Tools

**Git Clients:**
- Command line (git)
- GitHub CLI (gh)
- GitKraken (GUI)
- Sourcetree (GUI)

**Code Review:**
- GitHub web interface
- GitHub CLI
- gh-dash (terminal UI)

**Project Management:**
- GitHub Projects
- GitHub Issues
- Milestones
- Labels

### Useful Commands

```bash
# View commit history
git log --oneline --graph --all

# Interactive rebase
git rebase -i HEAD~3

# Stash changes
git stash
git stash pop

# Cherry-pick commit
git cherry-pick <commit-hash>

# Amend last commit
git commit --amend

# View diff
git diff
git diff --staged

# Branch management
git branch -a
git branch -d feature-branch
git branch -D force-delete-branch
```

---

## Conclusion

Following these Git workflow and GitHub best practices ensures:

- Clean project history
- Efficient collaboration
- High code quality
- Easier maintenance
- Better documentation
- Professional development process

**Remember:** Never commit directly to main. Always use feature branches and Pull Requests.

---

**Last Updated:** 2026-01-08  
**Status:** Recommended for all contributors  
**Enforcement:** Branch protection on main enabled