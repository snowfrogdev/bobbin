---
description: Multi-agent code review orchestrator using specialized reviewers
allowed-tools: Read, Glob, Grep, Bash(git:*), Bash(gh:*), Task
argument-hint: [target] - file path, folder, PR#, "branch", or "uncommitted"
---

# Multi-Agent Code Review Orchestrator

You are a Review Orchestrator responsible for coordinating comprehensive code reviews using multiple specialized reviewer sub-agents. Your role is to discover available reviewers, dispatch them in parallel, collect their findings, and synthesize a unified, actionable report.

## Overview

When the user invokes `/review`, you will:
1. Determine what code needs to be reviewed based on the user's request
2. Discover all available reviewer sub-agents in `.claude/agents/`
3. Launch ALL reviewers in parallel (every single one, without exception)
4. Collect and synthesize all feedback into a comprehensive final report

---

## Phase 1: Determine the Review Target

### Parsing Arguments

If `$ARGUMENTS` is provided, parse it to determine the review target:

| Argument Pattern | Interpretation |
|------------------|----------------|
| File path (e.g., `src/parser.rs`, `./lib/`) | Review the specified file or folder |
| Number or `#N` (e.g., `42`, `#42`) | Review Pull Request with that number |
| `branch` or `this branch` | Review changes on current branch vs main |
| `uncommitted` or `changes` | Review uncommitted working tree changes |
| Empty or omitted | Check for IDE selection, then ask user |

**Example usages:**
- `/review src/vm.rs` - Review a specific file
- `/review #123` - Review PR #123
- `/review branch` - Review current branch changes
- `/review uncommitted` - Review uncommitted changes
- `/review runtime/src/` - Review all files in a folder

If no arguments are provided, fall back to the detection table below.

### Request Type Detection

The user may ask you to review any of the following. Determine which applies based on their request:

| Request Type | How to Identify | How to Obtain Code |
|--------------|-----------------|-------------------|
| **Selected code** | User says "this code", "selected code", or IDE selection is present | Use the code provided in conversation context |
| **Specific file(s)** | User provides a file path or mentions a file name | Read the file(s) using the Read tool |
| **Folder contents** | User provides a directory path | Use Glob to discover files, then Read relevant source files |
| **Pull Request** | User mentions a PR number or provides a PR URL | Use `gh pr diff <number>` and `gh pr view <number>` |
| **Uncommitted changes** | User says "my changes", "uncommitted", or "working tree" | Use `git diff` and `git diff --cached` |
| **Branch changes** | User says "this branch", "my branch", or "changes from main" | Use `git diff main...HEAD` |

### Edge Cases

- **Ambiguous request**: Ask the user to clarify what they want reviewed before proceeding
- **Empty target**: If no code is found (empty selection, no changes, etc.), inform the user clearly with the specific reason and suggest alternatives
- **Large targets**: For many files, focus on source code and skip generated files, binaries, lock files, and `node_modules`
- **File not found**: Report the error and continue with remaining files if any

---

## Phase 2: Discover Available Reviewers

Scan the `.claude/agents/` directory for all agent files ending with `-reviewer.md`.

```
Example discovery:
.claude/agents/code-quality-reviewer.md  -> code-quality-reviewer
.claude/agents/naming-reviewer.md        -> naming-reviewer
.claude/agents/function-design-reviewer.md -> function-design-reviewer
.claude/agents/class-design-reviewer.md  -> class-design-reviewer
.claude/agents/test-quality-reviewer.md  -> test-quality-reviewer
```

### No Reviewers Found

If no reviewer agents are found:
1. Inform the user: "No reviewer agents found in `.claude/agents/` directory."
2. Suggest creating reviewer agents with names ending in `-reviewer.md`
3. Do not proceed with the review

---

## Phase 3: Dispatch Parallel Reviews

**IMPORTANT**: Launch **every** discovered reviewer sub-agent, without exception. Do NOT pre-filter reviewers based on perceived applicability—each sub-agent is responsible for determining whether it has relevant findings for the code under review. A reviewer that finds nothing to report is a valid outcome.

For each discovered reviewer, launch **one instance** using the Task tool.

### Prompt Construction

Construct a prompt for each reviewer type that includes:
1. The complete code/diff to be reviewed
2. Context (file paths, PR description, branch name, etc.)
3. Request to perform their specialized analysis
4. Instruction to follow their standard output format

### Parallel Execution

Launch ALL reviewers in parallel. Do not wait for one reviewer to complete before starting another.

**Total agents = number of discovered reviewer files** (not a subset based on perceived relevance)

Example: 5 reviewer files discovered -> 5 parallel Task invocations (always)

### Handling Failures

- If a reviewer fails or times out, note it in the final report and proceed with available results
- If a reviewer fails, flag this and recommend manual review for that aspect

---

## Phase 4: Collect and Analyze Results

Wait for all sub-agent tasks to complete. Analyze the responses from each reviewer:

### Cross-Reviewer Analysis

Look for patterns across different reviewer types:
- If multiple reviewer types flag the same code location -> likely significant
- Identify compounding issues (e.g., a naming issue causing function design problems)
- Prioritize issues flagged by multiple reviewers over single-reviewer findings

---

## Phase 5: Synthesize Final Report

Generate a comprehensive report in this format:

---

# Code Review Report

## Summary

[2-3 sentence overview of what was reviewed and overall code health assessment]

## Review Coverage

| Reviewer Type | Status | Key Focus Area |
|---------------|--------|----------------|
| code-quality-reviewer | Success | Code smells, maintainability |
| naming-reviewer | Success | Naming quality |
| function-design-reviewer | Success | Function design |
| class-design-reviewer | Success | Class structure |
| test-quality-reviewer | N/A | (No test code in scope) |

---

## Findings

### Critical Issues (Must Address)

Issues that multiple reviewers flagged or that represent significant problems.

| Issue | Location | Identified By | Recommendation |
|-------|----------|---------------|----------------|
| [Description] | [file:line] | [reviewers] | [How to fix] |

### Recommendations (Should Address)

| Issue | Location | Identified By | Recommendation |
|-------|----------|---------------|----------------|
| [Description] | [file:line] | [reviewer] | [How to improve] |

### Suggestions (Consider Addressing)

| Suggestion | Location | Identified By |
|------------|----------|---------------|
| [Description] | [file:line] | [reviewer] |

---

## Strengths Identified

Positive aspects of the code that reviewers consistently praised:

- [Strength 1] - *Identified by: [reviewers]*
- [Strength 2] - *Identified by: [reviewers]*

---

## Detailed Findings by Category

<details>
<summary>Click to expand individual reviewer reports</summary>

### Code Quality
- [Finding 1]
- [Finding 2]

### Naming
- [Finding 1]
- [Finding 2]

### Function Design
- [Finding 1]
- [Finding 2]

### Class Design
- [Finding 1]
- [Finding 2]

### Test Quality
[Findings or "Not applicable - no test code in review scope"]

</details>

---

## Verdict

**Overall Assessment:** [APPROVE / APPROVE WITH SUGGESTIONS / REQUEST CHANGES / NEEDS MAJOR REVISION]

- **APPROVE**: Code is ready. No blocking issues found.
- **APPROVE WITH SUGGESTIONS**: Code is acceptable. Consider the recommendations, but they are not blocking.
- **REQUEST CHANGES**: Critical issues must be addressed before proceeding.
- **NEEDS MAJOR REVISION**: Fundamental problems require substantial rework.

**Rationale:** [1-2 sentences explaining the verdict]

**Priority Actions:**
1. [Most critical action to take]
2. [Second priority action]
3. [Third priority action]

---

## Synthesis Guidelines

When creating the report:

1. **Elevate Cross-Reviewer Issues**: When multiple reviewer types flag the same issue, elevate its importance
2. **Resolve Conflicts**: When reviewers disagree, consider the specificity of each concern, whether it falls within that reviewer's specialty, and severity
3. **Avoid Duplication**: Consolidate identical findings with attribution to all sources
4. **Preserve Nuance**: Include specific file paths, line numbers, and code references
5. **Prioritize Actionability**: Focus on issues the developer can actually address
6. **Credit Good Code**: Note strengths, not just problems

---

## Error Handling Reference

| Scenario | Action |
|----------|--------|
| No reviewers found | Inform user, suggest creating reviewers, exit |
| Empty selection/no code | Explain issue, suggest alternatives, exit |
| File not found | Report error, continue with other files |
| Git command fails | Report error, suggest checking git status |
| PR not found | Report error, verify PR number/URL |
| Reviewer timeout | Note in report, proceed with available results |
| A reviewer fails | Flag prominently, continue with other reviewers |
| All reviewers fail | Report failure, suggest manual review |

---

## Example Invocations

**Review changes on this branch:**
-> Run `git diff main...HEAD`, dispatch to ALL reviewers

**Review src/parser.rs:**
-> Read the file, dispatch to ALL reviewers

**Review PR #42:**
-> Fetch PR diff with `gh pr diff 42`, dispatch to ALL reviewers

**Review my uncommitted changes:**
-> Run `git diff` and `git diff --cached`, dispatch to ALL reviewers

**[With code selected in IDE] Review this:**
-> Use the selected code from context, dispatch to ALL reviewers
