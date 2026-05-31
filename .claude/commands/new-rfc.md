# /new-rfc

Create a new RFC and register it in Plane. RFC content lives in Plane pages; the repo only holds incorporated RFCs as permanent historical records.

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

2. **Derive the slug** (for the page name only — no file is created).
   Lowercase the title, replace spaces with hyphens, strip punctuation.
   Example: `Array literal syntax` → `rfc-0004-array-literal-syntax`

3. **Create a Plane page** with the RFC content:

   ```
   mcp__plane__create_project_page(
     project_id=METEL_PROJECT_ID,
     name="RFC-NNNN: <title>",
     description_html=<RFC body as HTML>
   )
   ```

   Use this template for the page content (convert to HTML):

   ```markdown
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

   Fill in sections if there is enough context from the conversation. Leave blank only when there is genuinely insufficient information.

4. **Create a Plane work item** linked to the page:

   ```
   mcp__plane__create_work_item(
     project_id=METEL_PROJECT_ID,
     name="RFC-NNNN: <title>",
     type_id=RFC_TYPE_ID,
     state=BACKLOG_STATE_ID,
     description_html="<p>See Plane page: <strong>RFC-NNNN: <title></strong></p>"
   )
   ```

5. **Set the RFC Status property** to `draft` on the new work item using the Plane REST API.
   Read the API key from `~/.claude.json` at runtime:

   ```bash
   PLANE_API_KEY=$(python3 -c "import json; print(json.load(open('/home/vlad/.claude.json'))['mcpServers']['plane']['headers']['Authorization'].split()[-1])")
   curl -s -X PATCH \
     -H "X-Api-Key: $PLANE_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{"custom_field_rfc-status": "draft"}' \
     "https://api.plane.so/api/v1/workspaces/vladastos/projects/ec7904a4-cd24-40bd-8089-19a5eb8875ab/work-items/<work_item_id>/properties/"
   ```

6. **Commit nothing** — no file is created. The RFC lives entirely in Plane until it is incorporated.

## Lifecycle

| Stage | Action |
|---|---|
| **Draft** | Content in Plane page. Work item RFC Status = `draft`. |
| **Accepted** | Update RFC Status = `accepted`. Assign to a Plane milestone for the target version. Update the Decision section in the Plane page. |
| **Incorporated** | Spec updated. Commit the RFC as a file to `docs/internal/rfcs/rfc-NNNN-<slug>.md` with minimal frontmatter (`id`, `title`, `date` only). Set RFC Status = `incorporated`, Spec Status = `done`, Doc Path = the repo file path. |
| **Superseded** | Set RFC Status = `superseded`. Add a note to the Plane page pointing to the superseding RFC. Create the superseding RFC as a new work item + page. |

## Incorporated RFC file template

Only used when an RFC is accepted and incorporated into the spec:

```markdown
---
id: rfc-NNNN
title: "<title>"
date: '<YYYY-MM-DD>'
---

## Summary

<one-paragraph summary>

---

<full RFC body>

---

## Decision

**Outcome:** Accepted
**Target:** vX.Y.Z

<decision rationale>
```

## Notes
- **No `status:`, `spec_status:`, or `target:` fields in the repo file** — lifecycle is tracked in Plane only.
- The RFC must be accepted and the relevant `docs/public/spec/` file updated before implementation begins.
- Milestones (not the Target Version property) track which release an RFC is scoped to.
- Spec Status options: `pending`, `done`.
- RFC Status options: `draft`, `accepted`, `incorporated`, `superseded`, `deferred`.
