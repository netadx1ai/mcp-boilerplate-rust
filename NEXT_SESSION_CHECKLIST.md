# Next Session Checklist

**Date**: 2026-01-08  
**Goal**: Upgrade to MCP Protocol 2025-11-25 with Phase 1 Quick Wins  
**Estimated Time**: 2-4 hours

## Pre-Session Setup

- [ ] Read `docs/MCP_SPEC_REVIEW_SUMMARY.md` (5 min)
- [ ] Check rmcp SDK version compatibility
- [ ] Backup current working state (git commit)
- [ ] Create feature branch: `git checkout -b feature/protocol-2025-11-25`

## Task 1: SDK Compatibility Check (15 min)

- [ ] Check current rmcp version: `grep rmcp Cargo.toml`
- [ ] Search for updates: `cargo search rmcp`
- [ ] Review rmcp changelog for 2025-11-25 support
- [ ] Update Cargo.toml if SDK supports new protocol
- [ ] Test build: `cargo build --release`

**If SDK doesn't support 2025-11-25 yet:**
- [ ] Skip protocol version update
- [ ] Focus on features that don't require SDK changes (icons, annotations)
- [ ] Document SDK limitation

## Task 2: Enhanced Error Handling (1 hour)

### 2.1 Update echo.rs
- [ ] Change empty message validation to return Tool Execution Error
- [ ] Change length validation to return Tool Execution Error
- [ ] Add helpful error messages for LLM self-correction
- [ ] Format: `Ok(CallToolResult { content: [TextContent {...}], is_error: true })`

### 2.2 Update calculator.rs
- [ ] Change division-by-zero to Tool Execution Error
- [ ] Change invalid operation to Tool Execution Error
- [ ] Change overflow errors to Tool Execution Error
- [ ] Add descriptive error messages

### 2.3 Testing
- [ ] Add error handling test in `scripts/test_validation.sh`
- [ ] Test LLM-friendly error messages
- [ ] Verify `isError: true` in responses
- [ ] Run: `./scripts/test_mcp.sh`

## Task 3: Icons Support (1-2 hours)

### 3.1 Add Icon Type Support
- [ ] Check rmcp::model::Icon availability
- [ ] Import Icon type in tool/prompt/resource modules
- [ ] Create helper function for default icons (optional)

### 3.2 Add Icons to Tools (src/tools/)
- [ ] echo - Add simple message icon
- [ ] ping - Add network/connectivity icon
- [ ] info - Add information icon
- [ ] calculate - Add calculator icon
- [ ] evaluate - Add math/formula icon

**Example**:
```rust
icons: Some(vec![Icon {
    src: "https://example.com/tool-icon.svg".to_string(),
    mime_type: Some("image/svg+xml".to_string()),
    sizes: Some(vec!["any".to_string()]),
}])
```

### 3.3 Add Icons to Prompts (src/prompts/mod.rs)
- [ ] code_review - Add code review icon
- [ ] explain_code - Add documentation icon
- [ ] debug_help - Add bug/debug icon

### 3.4 Add Icons to Resources (src/resources/mod.rs)
- [ ] config://server - Add settings/config icon
- [ ] info://capabilities - Add capabilities icon
- [ ] doc://quick-start - Add documentation icon
- [ ] stats://usage - Add statistics/chart icon

### 3.5 Testing
- [ ] Build: `cargo build --release`
- [ ] Test icon serialization
- [ ] Verify JSON output includes icons
- [ ] Run: `./scripts/test_prompts_resources.sh`

## Task 4: Resource Annotations (1 hour)

### 4.1 Check Annotation Type
- [ ] Verify rmcp supports annotations in ResourceContents
- [ ] Import/define Annotations type if needed

### 4.2 Add Annotations to Resources
- [ ] config://server
  - `audience: ["user"]`
  - `priority: 0.9` (high - config is important)
  - `lastModified: <timestamp>`

- [ ] info://capabilities
  - `audience: ["user", "assistant"]`
  - `priority: 0.8`
  - `lastModified: <timestamp>`

- [ ] doc://quick-start
  - `audience: ["user"]`
  - `priority: 0.7`
  - `lastModified: <timestamp>`

- [ ] stats://usage
  - `audience: ["user"]`
  - `priority: 0.5` (low - nice to have)
  - `lastModified: <timestamp>`

### 4.3 Testing
- [ ] Build: `cargo build --release`
- [ ] Verify annotation serialization
- [ ] Test resource read with annotations
- [ ] Run: `./scripts/test_prompts_resources.sh`

## Task 5: Update Protocol Version (30 min)

**Only if rmcp SDK supports 2025-11-25**

### 5.1 Update Code
- [ ] Update `src/mcp/stdio_server.rs`:
  - `ProtocolVersion::V_2025_11_25`
- [ ] Update `src/main.rs`:
  - Version string references
- [ ] Update startup logs

### 5.2 Update Documentation
- [ ] README.md - Protocol version
- [ ] CLAUDE.md - Protocol version
- [ ] IMPLEMENTATION_STATUS.md - Protocol version
- [ ] docs/PROMPTS_AND_RESOURCES.md - Add examples

### 5.3 Testing
- [ ] Full test suite: `./scripts/verify_claude_ready.sh`
- [ ] Verify protocol negotiation
- [ ] Check JSON-RPC responses

## Task 6: Documentation Updates (30 min)

### 6.1 Create New Docs
- [ ] Create `docs/PROTOCOL_UPGRADE_GUIDE.md`
  - Migration notes
  - Feature comparison 2024-11-05 vs 2025-11-25
  - Breaking changes (if any)

### 6.2 Update Existing Docs
- [ ] Update `CLAUDE.md`:
  - New protocol version
  - Icon examples
  - Annotation examples
  - Error handling pattern

- [ ] Update `README.md`:
  - Feature highlights
  - Protocol version badge
  - Quick wins section

- [ ] Update `docs/IMPLEMENTATION_STATUS.md`:
  - Mark completed features
  - Update status table
  - Note protocol version

- [ ] Update `docs/PROMPTS_AND_RESOURCES.md`:
  - Add icon usage examples
  - Add annotation examples
  - Update best practices

## Final Validation

### Build & Test
- [ ] Clean build: `cargo clean && cargo build --release`
- [ ] Run clippy: `cargo clippy --release --all-features`
- [ ] Format code: `cargo fmt`
- [ ] Run all tests:
  - [ ] `./scripts/test_mcp.sh`
  - [ ] `./scripts/test_prompts_resources.sh`
  - [ ] `./scripts/test_http.sh` (if HTTP mode)
  - [ ] `./scripts/test_validation.sh`
  - [ ] `./scripts/verify_claude_ready.sh`

### Code Quality
- [ ] Zero clippy warnings
- [ ] All tests passing (34/34)
- [ ] Code formatted
- [ ] No TODO comments added
- [ ] Error messages clear and helpful

### Documentation
- [ ] All docs updated
- [ ] Examples tested
- [ ] Links valid
- [ ] Changelog updated (if applicable)

## Commit & Push

- [ ] Review changes: `git status`
- [ ] Stage files: `git add .`
- [ ] Commit: `git commit -m "feat: upgrade to MCP protocol 2025-11-25 with icons, annotations, and enhanced error handling"`
- [ ] Push: `git push origin feature/protocol-2025-11-25`
- [ ] Create PR (optional)

## Success Criteria

✅ **Must Have**:
- Enhanced error handling implemented (Tool Execution Errors)
- Icons added to all tools, prompts, resources
- Resource annotations implemented
- All tests passing
- Documentation updated

✅ **Nice to Have**:
- Protocol version updated to 2025-11-25 (if SDK supports)
- Examples in docs show new features
- Performance unchanged or improved

## Rollback Plan

If issues occur:
```bash
git checkout main
git branch -D feature/protocol-2025-11-25
```

## Post-Session

- [ ] Test integration with Claude Desktop
- [ ] Verify icons display correctly
- [ ] Test error self-correction with LLM
- [ ] Document any issues found
- [ ] Plan Phase 2 (Output Schemas) for next session

---

**Notes**:
- Keep commits atomic (one feature per commit)
- Test after each major change
- Don't rush - quality over speed
- Document any SDK limitations found

**Session Status**: [ ] Not Started / [ ] In Progress / [ ] Complete