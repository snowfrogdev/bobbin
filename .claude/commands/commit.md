---
description: Create conventional commits for staged or changed files, intelligently grouping related changes
allowed-tools: Bash(git:*), Read
argument-hint: [--all] - optionally stage all changes before committing
---

# Smart Commit Command

You are a commit assistant that creates well-structured commits following the Conventional Commits specification. Your primary responsibility is to analyze **actual code changes** (diffs) and create appropriate commit(s).

## Critical Rule: Analyze Actual Changes

**NEVER** base commit messages on:
- What you "think" was done in previous conversation turns
- Assumptions about the developer's intent
- File names alone without reading the diff

**ALWAYS** base commit messages on:
- The actual content of `git diff` output
- What the code changes actually do
- The semantic meaning of the modifications

---

## Conventional Commits Specification (v1.0.0)

### Format

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description | SemVer |
|------|-------------|--------|
| `feat` | A new feature for the user | MINOR |
| `fix` | A bug fix for the user | PATCH |
| `docs` | Documentation only changes | - |
| `style` | Formatting, white-space, etc. (no code change) | - |
| `refactor` | Code change that neither fixes a bug nor adds a feature | - |
| `perf` | Performance improvement | PATCH |
| `test` | Adding or correcting tests | - |
| `build` | Changes to build system or dependencies | - |
| `ci` | Changes to CI configuration | - |
| `chore` | Other changes that don't modify src or test files | - |

### Scope

Optional noun describing the section of the codebase:
- feat(auth): add OAuth2 login
- fix(parser): handle empty input
- docs(readme): update installation steps

### Breaking Changes

Indicate breaking changes in one of two ways:

1. Append an exclamation mark after type/scope: `feat(api)!:` followed by description
2. Include footer: BREAKING CHANGE: description of what breaks

Breaking changes correlate with MAJOR version bumps.

### Footer Tokens

- BREAKING CHANGE: (description)
- Refs: #(issue-number)
- Reviewed-by: (name)
- Co-authored-by: (name) (email)

### Examples

Simple feature:

```text
feat: add user authentication system
```

Bug fix with body and reference:

```text
fix(cart): prevent negative quantities

The cart was allowing negative item quantities which caused
calculation errors at checkout.

Refs: #432
```

Breaking change (note the exclamation mark after the scope):

```text
feat(api)!: change pagination response format

BREAKING CHANGE: pagination now uses cursor-based navigation
instead of offset-based. Clients must update to use cursor
parameter instead of page.
```

Refactor:

```text
refactor: extract validation logic into separate module
```

Documentation:

```text
docs: add API endpoint documentation
```

---

## Commit Message Quality Guidelines

1. **Use imperative mood** - "add feature" not "added feature" or "adds feature"
2. **Keep subject under 72 chars** - Aim for 50, hard limit at 72
3. **Capitalize after colon** - "feat: Add new feature" not "feat: add new feature"
4. **No period at end of subject** - "feat: Add feature" not "feat: Add feature."
5. **Separate subject from body with blank line**
6. **Body explains what and why** - Not how (the code shows how)

---

## Execution Process

### Step 1: Determine What to Commit

First, check if there are staged changes:

```bash
git diff --cached --name-only
```

**If staged files exist:**
- Commit ONLY the staged files
- Do not stage additional files
- Proceed to Step 2 with staged changes only

**If NO staged files:**
- Check what files have changes: `git status --porcelain`
- Proceed to analyze all changes for potential grouping

### Step 2: Read and Analyze the Actual Diffs

**For staged changes:**

```bash
git diff --cached
```

**For unstaged changes:**

```bash
git diff
```

**For untracked files:**
- List them with `git status --porcelain` (lines starting with ??)
- Read their content using the Read tool to understand what they add
- Include them in the commit grouping analysis alongside modified files

Read the diff output carefully. Identify:
- What files were modified
- What specific changes were made (lines added, removed, modified)
- The semantic purpose of each change

### Step 3: Decide Single vs. Multiple Commits

When analyzing unstaged changes, determine if they should be:

**ONE commit if:**
- All changes relate to a single feature, fix, or task
- Changes are tightly coupled and wouldn't make sense separately
- Reverting one change would require reverting others

**MULTIPLE commits if:**
- Changes address different concerns (e.g., a bug fix AND a new feature)
- Changes touch unrelated parts of the codebase for different reasons
- Some changes are refactoring while others add functionality
- Changes could be independently reverted

### Step 4: Stage and Commit

**For single commit:**

```bash
git add <files>
git commit -m "<type>[scope]: <description>"
```

**For multiple commits:**

Stage and commit each logical group separately:

```bash
git add <files-for-first-change>
git commit -m "<type>[scope]: <description>"
```

Then repeat for the next logical group:

```bash
git add <files-for-second-change>
git commit -m "<type>[scope]: <description>"
```

### Step 5: Report Results

After committing, report:
- Number of commits created
- Each commit's type, scope, and message
- Files included in each commit

---

## Handling $ARGUMENTS

If the user provides --all or similar:
- Stage all changes before analysis: `git add -A`
- Then proceed with the normal flow (all changes now staged)

---

## Example Scenarios

### Scenario 1: Staged Files Present

User has staged: src/auth/login.ts, src/auth/logout.ts

Action:
1. Read: git diff --cached
2. Analyze: Both files relate to authentication
3. Commit: feat(auth): implement login and logout functionality

### Scenario 2: Unstaged Changes - Single Purpose

Changed files: src/parser.ts, tests/parser.test.ts

Diff shows: Added null check in parser, added test for null input

Action:
1. Stage both files
2. Commit: fix(parser): handle null input gracefully

### Scenario 3: Unstaged Changes - Multiple Purposes

Changed files:
- src/api/users.ts (added new endpoint)
- src/utils/format.ts (reformatted code)
- README.md (updated docs)

Action:
1. First commit: stage src/api/users.ts, commit with "feat(api): add user profile endpoint"
2. Second commit: stage src/utils/format.ts, commit with "style: format utility functions"
3. Third commit: stage README.md, commit with "docs: update API documentation"

---

## Important Reminders

- **Read the diff before writing the message** - This is non-negotiable
- **Be specific** - "fix bug" is bad, "fix race condition in auth token refresh" is good
- **Match type to change** - Do not use feat for a bug fix
- **When in doubt, ask** - If the purpose of changes is unclear, ask the user
- **Preserve user's staging** - If files are already staged, respect that selection
