---
name: session-wrap-up
description: End-of-session automation. Commits unpushed work, extracts learnings to memory, detects patterns, and persists rules to corrections.md. Use at end of long work sessions or when Boss says "wrap up".
---

# Session Wrap-Up

When triggered (Boss says "wrap up", "end session", or similar):

## Steps

1. **Check for unpushed git changes**
   - Run `git status` in the workspace and any active project dirs under `projects/`
   - If there are uncommitted changes, ask Boss if they want to commit

2. **Extract learnings**
   - Review the current conversation for mistakes, lessons, or useful discoveries
   - Log any mistakes to `memory/corrections.md` with format: `YYYY-MM-DD | [Category] | What went wrong | What I learned`

3. **Update daily memory**
   - Append a summary of what was accomplished to `memory/YYYY-MM-DD.md`
   - Include: tasks completed, decisions made, issues encountered

4. **Detect patterns**
   - If the same mistake appears multiple times in `memory/corrections.md`, escalate it to `MEMORY.md` as a permanent rule

5. **Report**
   - Send a concise summary to Boss:
     - What was done
     - What was learned
     - Any open items or recommendations
