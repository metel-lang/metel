# /sprint-end

Close a sprint: run tests, carry over incomplete issues, build and publish the sprint review issue and PR, then hand off to the user.

**Arguments:** `$ARGUMENTS` — sprint number, e.g. `3`

## Steps

1. **Fetch the sprint kickoff issue** to retrieve the sprint goal, planned issues, and kickoff issue number:
```bash
wsl gh issue list --repo Vladastos/Yoloscript \
  --label "sprint:kickoff" \
  --search "Sprint <N> Kickoff" \
  --json number,title,body
```

2. **Categorise planned issues** into completed and carried-over:
```bash
wsl gh issue list --repo Vladastos/Yoloscript \
  --label "status:in-progress" \
  --json number,title,state,milestone
```
Issues still open → carried over. Issues closed during the sprint → completed.

3. **Move carried-over issues back to backlog:**
```bash
wsl gh issue edit <N> --repo Vladastos/Yoloscript \
  --remove-label "status:in-progress" \
  --add-label "status:backlog"
```

4. **Ensure all tests pass on the sprint branch:**
```bash
cd tree-walk-interpreter && cargo test
```
If any tests fail, do not proceed — fix them first.

5. **Gather Epic Progress data.**
Determine which milestone(s) the sprint issues belong to (from step 2). For each milestone, fetch its open and closed issue counts:
```bash
wsl gh api repos/Vladastos/Yoloscript/milestones \
  --jq '.[] | select(.title == "<milestone>") | {title: .title, open: .open_issues, closed: .closed_issues}'
```
Format as: `<milestone>: <closed>/<closed+open> issues closed`.

6. **Gather Spec Notes.**
Find all commits on the sprint branch that touched the `docs/` submodule:
```bash
git log main..HEAD --oneline -- docs/
```
Also check for any RFC status changes committed during the sprint:
```bash
git log main..HEAD --oneline -- docs/internal/rfcs/
```
If there are no such commits, write "No spec changes this sprint."

7. **Create the sprint review issue** using all data gathered above:
```bash
wsl gh issue create \
  --repo Vladastos/Yoloscript \
  --title "Sprint <N> Review" \
  --label "sprint:review" \
  --body "## Sprint Goal
<goal from kickoff issue>

## Completed
<for each closed issue: - [x] #N Title>

## Carried Over
<for each open issue: - [ ] #N Title>

## Epic Progress
<milestone progress lines from step 5>

## Spec Notes
<doc commit summaries from step 6, or 'No spec changes this sprint.'>

## Next Sprint Seeds
<!-- Add ideas for the next sprint here -->"
```
Note the issue number returned — it is needed for the PR body.

8. **Open a pull request** from `sprint/<N>` → `main`:
```bash
gh pr create \
  --repo Vladastos/Yoloscript \
  --base main \
  --head sprint/<N> \
  --title "Sprint <N> — <theme>" \
  --body "$(cat <<'EOF'
Sprint review: #<review-issue-number>

Closes #<review-issue-number>
Closes #<kickoff-issue-number>
EOF
)"
```
Both `Closes` lines are required. On merge, GitHub automatically closes the sprint review issue and the kickoff issue.

9. **Leave a note for the user:**

> **Sprint <N> is wrapped up.**
>
> - Review issue: #<review-issue-number> — all sections are filled in. Add **Next Sprint Seeds** if you have ideas.
> - **Merge the PR** on GitHub — this automatically closes the review issue and the kickoff issue.
> - After merging, delete the `sprint/<N>` branch on GitHub.

## Notes
- A sprint with 0 completed issues should still produce a review issue — record why in Completed.
- Do not close the kickoff or review issues manually — both close via `Closes #N` on PR merge.
- If spec ambiguities surfaced (visible in Spec Notes), prompt the user to open a `/new-rfc`.
- The sprint branch must not be deleted until after the PR is merged.
