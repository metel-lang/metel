# /sprint-start

Open a new sprint: create the sprint branch, create the Plane cycle, assign work items, and mark them in-progress.

**Arguments:** `$ARGUMENTS` — sprint number and goal, e.g. `16 "Implement expression evaluation"`

## Steps

1. **Parse arguments.** Extract the sprint number (integer) and the sprint goal (quoted string).

2. **Determine the active milestone.** Read `CLAUDE.md` for the current development focus milestone (e.g. `v0.6.5`). All planned work items must carry this milestone. If the milestone is ambiguous, ask the user before continuing.

3. **Show the current backlog** in Plane so the user can decide what goes into the sprint:
```
mcp__plane__list_work_items  →  filter by backlog state
```

4. **Ask the user** which work item identifiers to include in this sprint before proceeding.

5. **Create and push the sprint branch:**
```bash
git checkout main && git pull
git checkout -b sprint/<N>
git push -u origin sprint/<N>
```
All sprint work must be committed to `sprint/<N>`. Nothing goes directly to `main` during the sprint.

6. **Create the Plane cycle** for this sprint:
```
mcp__plane__create_cycle  →  name "Sprint <N>: <goal>", set start/end dates if known
```

7. **Add work items to the cycle** and move them to In Progress state:
```
mcp__plane__add_work_items_to_cycle  →  add all planned items
mcp__plane__update_work_item  →  set state to In Progress for each
```

8. **Report** the sprint branch name, milestone, cycle URL, and list of work items now in the sprint.

## Notes
- Sprint numbers are sequential integers (Sprint 1, Sprint 2, …).
- The sprint goal should be one sentence.
- Only work items in the backlog state should be moved into a sprint.
- Check CLAUDE.md for the active epic and milestone before proceeding.
- All work items created or touched during the sprint must carry the active milestone.
- Remind the user: all commits must go on `sprint/<N>`, not on `main`.
- Commit messages must follow: `type(METEL-<N>): description`
