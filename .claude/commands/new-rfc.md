# /new-rfc

Create a new RFC. The markdown file in the docs repo is the source of truth for RFC content; Plane tracks lifecycle state only.

**Arguments:** `$ARGUMENTS` — the RFC title (e.g. `Array literal syntax`)

## Steps

1. **Determine the next RFC number.**
   Fetch all RFC-type work items from Plane and find the highest RFC-NNNN number. Increment by one. Zero-pad to four digits.

   ```
   mcp__plane__list_work_items(project_id=METEL_PROJECT_ID, type_ids=[RFC_TYPE_ID])
   ```

   Plane IDs:
   - Project: `ec7904a4-cd24-40bd-8089-19a5eb8875ab`
   - RFC type: `6b00ca94-017d-45e2-81eb-f7b6bed6ac89`
   - Backlog state: `db7c9b8f-cc28-4cd3-8cf1-42092afcef6c`

2. **Derive the slug** (used for the file name).
   Lowercase the title, replace spaces with hyphens, strip punctuation.
   Example: `Array literal syntax` → `rfc-0004-array-literal-syntax`

3. **Create the RFC file** at `docs/internal/rfcs/0-draft/rfc-NNNN-<slug>.md`.

   Use this template, filling in sections from conversation context. Leave a section body blank only when there is genuinely insufficient information.

   ```markdown
   ---
   id: rfc-NNNN
   title: "<title>"
   date: '<YYYY-MM-DD>'
   status: draft
   ---

   ## Summary


   ---

   ## Motivation


   ---

   ## Proposal


   ---

   ## Alternatives Considered


   ---

   ## Open Questions


   ---

   ## Timing Recommendation


   ---

   ## References

   - Language spec: `docs/public/spec.md`

   ---

   ## Decision

   **Outcome:** *(pending)*
   **Target:** *(set when accepted)*

   *(Decision rationale goes here when the RFC is evaluated.)*
   ```

4. **Commit the file.**
   Commit message: `docs(RFC-NNNN): add draft RFC — <title>`

5. **Create a Plane work item.**

   ```
   mcp__plane__create_work_item(
     project_id=METEL_PROJECT_ID,
     name="RFC-NNNN: <title>",
     type_id=RFC_TYPE_ID,
     state=BACKLOG_STATE_ID,
     description_html='<p>RFC file: <code>docs/internal/rfcs/0-draft/rfc-NNNN-&lt;slug&gt;.md</code></p>'
   )
   ```

6. **Set the RFC Status property** to `draft` on the new work item using the Plane REST API.
   Read the API key from `~/.claude.json` at runtime:

   ```bash
   PLANE_API_KEY=$(python3 -c "import json; print(json.load(open('/home/vlad/.claude.json'))['mcpServers']['plane']['headers']['Authorization'].split()[-1])")
   curl -s -X PATCH \
     -H "X-Api-Key: $PLANE_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{"custom_field_rfc-status": "draft"}' \
     "https://api.plane.so/api/v1/workspaces/vladastos/projects/ec7904a4-cd24-40bd-8089-19a5eb8875ab/work-items/<work_item_id>/properties/"
   ```

## Lifecycle

| RFC Status | Plane state | Action |
|---|---|---|
| `draft` | Backlog | File at `docs/internal/rfcs/0-draft/`. Frontmatter `status: draft`. |
| `accepted` | Todo | Update Decision section in file. Set frontmatter `status: accepted`. Assign to Plane milestone. |
| `incorporated` | Done | Spec updated. Set frontmatter `status: incorporated`, add `spec_status: done`. |
| `implemented` | Done | Feature working. Move file to `docs/internal/rfcs/3-implemented/`. Set frontmatter `status: implemented`. |
| `superseded` | Cancelled | Move file to `docs/internal/rfcs/4-superseded/`. Add note pointing to superseding RFC. Set frontmatter `status: superseded`. |
| `deferred` | Cancelled | Set frontmatter `status: deferred`. No other action. |

When transitioning an RFC's status, always update **both** the frontmatter `status` field in the file (commit it) and the RFC Status custom property in Plane to keep them in sync. The Plane state is a coarse queue signal for board visibility; the frontmatter is the authoritative lifecycle record.

## Notes
- **No `target:` field in the file** — milestone is tracked in Plane only.
- The RFC must be accepted and the relevant `docs/public/spec/` file updated before implementation begins.
- Milestones (not a Target Version property) track which release an RFC is scoped to.
- RFC Status options: `draft`, `accepted`, `incorporated`, `implemented`, `superseded`, `deferred`.
