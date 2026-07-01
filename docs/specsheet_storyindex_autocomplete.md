Basscript Specsheet - Story Index Database and Link Autocomplete

Goal

Add Q-style link autocomplete for known story entities and a persistent local Story Index Database for answering workspace-level story questions.

Autocomplete should appear where it makes sense while writing, not only after typing `[`. If the user types `eo`, `Eo`, `EOG`, or `eog`, Basscript should be able to suggest the known character `Eoghan` and insert a correct script link with simple keyboard selection.

The story index should support questions such as:

- all props in the current scene
- all scenes two characters have together
- all places a character has been
- all appearances of an entity
- all characters in the current scene

Reference behavior

Q-style writing tools keep a saved story index that feels like a database of the project. Basscript should do the same: each workspace should have a local, persistent Story Index Database that is available after indexing and reused across app launches.

Basscript project files remain the readable project data:

- `.fountain` scripts
- `.md` / `.markdown` entity files
- `.canvas` boards
- explicit script links
- entity front matter

The database is the app-managed story index used for autocomplete, query, navigation, and fast lookups. It must be rebuildable from project files if it is missing or stale, but this rebuild behavior is an implementation safety property, not a separate user-facing workflow.

Existing link syntax

- `[Eoghan]`
- `[EOGHAN](eoghan)`
- `[that door](door-kitchen-main)`

Existing entity metadata

- `target`
- `type`
- `name`
- `aliases`
- `status`

Common entity types

- `character`
- `prop`
- `place`
- `faction`
- `concept`
- `scene`

Entity types must not be hardcoded as the only allowed types. The database should store whatever `type` is declared in front matter, while giving common types better display and query behavior.

Epic Q - Story Index Database

Q1. Story index storage

Status: implemented v1

Create one persistent local Story Index Database per workspace.

Rules

- Use an embedded local database in v1, preferably SQLite unless there is a strong reason to choose another embedded store.
- The database is app-managed and requires no external server.
- The selected database path must be deterministic for a workspace and reused across app launches.
- The exact database location must be documented in code and status/debug output.
- The database has a schema version.
- Schema changes must migrate, rebuild, or clearly invalidate the index without corrupting project files.
- Deleting the database must not delete story data, because story data lives in the workspace files.
- A missing database is rebuilt from the workspace files.
- A corrupt or unreadable database shows a useful status message and falls back to rebuilding or creating a fresh index.

Acceptance

- Opening a workspace creates or reuses its Story Index Database.
- Closing and reopening Basscript keeps autocomplete and story queries available without a full manual setup step.
- Deleting the database causes Basscript to rebuild it from project files.
- Schema version mismatch does not crash app startup.
- Database errors do not mutate or delete `.fountain`, `.md`, `.markdown`, or `.canvas` files.

Q2. Workspace scan and invalidation

Status: partially implemented v1 file scan

Index the files that can contribute story knowledge.

Rules

- Scan supported workspace files:
  - `.fountain`
  - `.md`
  - `.markdown`
  - `.canvas`
- Entity Markdown files are identified by valid YAML front matter with at least `target`, `type`, `name`, and `aliases`.
- Fountain files are parsed for scene headings, line kinds, script links, character cues, dialogue blocks, and scene boundaries.
- Canvas files are indexed for text node contents, file node paths, link URLs, and referenced script/Markdown files where practical.
- The index stores file path, file kind, modification metadata, and a content hash or equivalent stale-check signal.
- Saving a file from Basscript updates the relevant index records.
- Creating, deleting, or renaming files through the explorer updates or invalidates relevant index records.
- External file changes may be detected by file watching or by a rescan on focus/open/save in v1.
- Indexing work must be debounced so typing does not trigger expensive full scans.
- Indexing must not block normal editing or rendering.

Acceptance

- Saving a new entity Markdown file makes it appear in autocomplete.
- Saving a Fountain file updates scenes, appearances, props, places, and co-appearance queries.
- Renaming or deleting an entity file removes or invalidates its autocomplete entry.
- Editing one script file does not force a full workspace reindex unless required.
- Status messages distinguish `Indexing`, `Index ready`, and `Index failed`.

Q3. Entity records

Status: implemented v1

Store known story entities from Markdown front matter.

Suggested database fields

- `target`
- `type`
- `name`
- `aliases`
- `status`
- `path`
- `source_file_id`
- `updated_at` or equivalent file metadata

Rules

- `target` remains the canonical link key.
- `target` must use the existing valid target-key rules.
- `name` is the preferred display label.
- `aliases` are searchable autocomplete terms.
- `type` controls display category, type color, and query grouping.
- Duplicate targets are reported clearly.
- Invalid front matter is reported clearly and excluded from autocomplete until fixed.
- Ambiguous aliases are stored and surfaced as multiple suggestions rather than guessed silently.

Acceptance

- An entity file for `eoghan.md` with `name: Eoghan` is indexed as target `eoghan`.
- Aliases such as `Eo` or `EOG` can match the same entity.
- Entity types such as `character`, `prop`, and `place` display distinctly.
- Unknown custom entity types still appear in autocomplete and search.
- Duplicate or invalid entity files do not crash indexing.

Q4. Scene records

Status: implemented v1

Make Fountain scenes queryable even when the user has not created explicit scene entity files.

Rules

- Every Fountain scene heading creates an automatic scene record.
- A scene record stores:
  - source script path
  - scene ordinal within the script
  - heading text
  - normalized heading text
  - start line
  - end line
  - script order
  - inferred location text when practical
  - inferred time-of-day text when practical
- Scene records may later be linked to explicit scene entity files.
- Automatic scene records can appear in story queries.
- Scene records may appear in autocomplete only when the context calls for scene links or when scene suggestions are enabled.
- Reordering scenes updates script order on the next index pass.

Acceptance

- `INT. KITCHEN - NIGHT` becomes a queryable scene record.
- The current cursor line can be mapped to its containing scene.
- Queries can return scenes even if there is no `scene` Markdown entity file.
- Scene indexing survives duplicate scene headings by using file path and ordinal or another stable internal key.

Q5. Appearance records

Status: todo

Index where entities appear in scripts and canvases.

Suggested database fields

- `target`
- `entity_type`
- `source_path`
- `scene_id`
- `line`
- `column`
- `line_kind`
- `appearance_role`
- `raw_snippet`

Appearance roles

- `character_cue`
- `dialogue_speaker`
- `action_mention`
- `dialogue_mention`
- `parenthetical_mention`
- `scene_heading`
- `canvas_text`
- `canvas_file`
- `canvas_link`

Rules

- Explicit script links create appearance records.
- A linked Fountain character cue creates a `character_cue` appearance.
- Dialogue lines following a linked character cue create `dialogue_speaker` appearances for the current speaker.
- Linked mentions inside action lines create `action_mention` appearances.
- Linked mentions inside dialogue lines create `dialogue_mention` appearances.
- Linked mentions inside parentheticals create `parenthetical_mention` appearances.
- Place matches inferred from scene headings may create place appearances, but they must be marked as inferred.
- Potential unlinked mentions may be stored separately as weak candidates, but query answers must distinguish them from explicit links.

Acceptance

- `[EOGHAN](eoghan)` on a character cue line records Eoghan as present in that scene.
- Dialogue following `[EOGHAN](eoghan)` counts as Eoghan dialogue until the dialogue block ends.
- `[knife](knife)` in action records that prop in the current scene.
- Unlinked text does not silently become a confirmed entity appearance.
- Query results can navigate back to the source path and line.

Q6. Query API

Status: todo

Expose story queries through a core/index API without Bevy UI types.

Required queries

- entities matching text
- appearances of one entity
- entities in current scene
- props in current scene
- characters in current scene
- scenes containing one entity
- scenes containing all selected entities
- places associated with one character
- backlinks to one entity

Rules

- Query results include enough navigation data to open the source file and line.
- Query results preserve script order when the answer is scene-based.
- Query results include entity display names and types where available.
- Queries over missing or stale index data return a clear partial/stale status.
- Core query APIs must not depend on Bevy.

Acceptance

- The UI can ask for all props in the current scene and receive prop records with source lines.
- The UI can ask for all scenes shared by Eoghan and another character and receive ordered scene results.
- The UI can ask for places Eoghan has been and receive places grouped by scene.
- Query code can be unit-tested without launching the Bevy app.

Q7. Index consistency and performance

Status: todo

Keep the story index fast and trustworthy.

Rules

- Writes to the database should use transactions.
- Index refreshes should be interruptible or coalesced when the user saves repeatedly.
- The editor must remain usable while indexing is in progress.
- Read queries should use the last complete index snapshot when a refresh is still running.
- Large workspaces should avoid full reparses on every keystroke.
- Indexing should share existing parser/link logic instead of reimplementing Fountain and link parsing.

Acceptance

- Typing in a large file does not stall because the story database is updating.
- Repeated saves do not queue an unbounded number of index jobs.
- The database is not left half-updated after an indexing failure.
- Existing parser tests remain authoritative for link and line classification behavior.

Epic R - Link Autocomplete

R1. Autocomplete trigger contexts

Status: todo

Show link suggestions where they make sense while writing.

Rules

- Autocomplete applies when the main text editor has text input focus.
- Autocomplete may also apply in canvas text nodes after the canvas text editor supports normal caret placement.
- Autocomplete does not apply in:
  - explorer path prompts
  - delete prompts
  - settings fields
  - command menu input unless explicitly requested later
  - native dialogs
- Typing `[` opens link autocomplete immediately.
- Typing a normal word prefix opens inline autocomplete when the prefix plausibly matches known entities.
- Default inline prefix length is 2 visible characters.
- Prefix matching is case-insensitive.
- `eo`, `Eo`, `EOG`, and `eog` can all suggest `Eoghan`.
- The suggestion menu closes when the cursor leaves the trigger range, the user types a hard delimiter, or no candidate remains.
- Autocomplete should be disabled while Vim Normal or Visual mode is consuming command keys.
- Autocomplete should work normally in Vim Insert mode.

Acceptance

- Typing `eo` in script text can show `Eoghan`.
- Typing `[` shows link suggestions without needing two characters.
- Autocomplete does not appear while typing a file path in the explorer create prompt.
- Autocomplete does not steal Normal-mode Vim movement keys.

R2. Candidate matching

Status: todo

Build suggestions from the Story Index Database.

Rules

- Candidate sources include:
  - entity `name`
  - entity `target`
  - entity `aliases`
  - automatic scene records when scene suggestions are enabled or contextually relevant
  - recently used entities in the current document
- Exact prefix matches rank above fuzzy matches.
- Name and alias matches rank above raw target matches when scores are otherwise equal.
- Current-scene and current-document entities may receive a small ranking boost.
- Character cue context should prefer `character` entities.
- Scene heading context should prefer `place` and `scene` entities when suggestions are shown.
- Action and dialogue contexts can suggest all entity types.
- Ambiguous aliases show each matching entity as a separate row.
- Suggestions must keep enough data to insert the correct link target even when display labels are identical.

Acceptance

- `eog` ranks `Eoghan` above unrelated fuzzy matches.
- A prop alias can suggest its prop entity.
- A place alias can suggest its place entity.
- A character cue line prefers character candidates over props with similar names.
- Ambiguous aliases do not insert a target until the user selects one.

R3. Suggestion dropdown UI

Status: todo

Render a small keyboard-driven menu near the caret.

Rules

- The menu appears close to the caret or active text insertion point.
- The menu should avoid covering the active typed prefix when practical.
- Rows show at least:
  - display name
  - entity type
  - target key when needed to disambiguate
- Type color should reuse the existing processed-link color mapping where practical.
- The selected row is visibly distinct.
- The menu has a maximum visible row count and scrolls internally after that.
- Menu dimensions should be stable so changing selection does not shift text layout.
- Mouse selection is optional in v1, but keyboard selection is required.
- The menu must render above processed/raw text and not be clipped by normal line rendering.

Acceptance

- The user can see which suggestion is selected.
- Long names or targets do not overflow the menu.
- Suggestions remain readable in raw, focus, split, and processed editing contexts.
- The menu does not move document text or affect line wrapping.

R4. Keyboard selection

Status: todo

Use simple editor-style autocomplete controls.

Rules

- `Down` selects the next suggestion.
- `Up` selects the previous suggestion.
- `Enter` accepts the selected suggestion.
- `Esc` closes the menu without changing text.
- If the menu is closed, `Up`, `Down`, `Enter`, and `Esc` keep their normal editor behavior.
- While the menu is open, `Up` and `Down` must not move the document cursor.
- Holding `Up` or `Down` may repeat using the existing key repeat behavior.
- `Tab` acceptance is optional and should not be required in v1.

Acceptance

- The user can type `eog`, press `Down` if needed, and press `Enter` to insert the link.
- Pressing `Esc` closes suggestions and leaves the typed text unchanged.
- Arrow navigation returns to normal after the menu closes.

R5. Inserting accepted links

Status: todo

Accepting a suggestion replaces the active trigger text with valid script-link syntax.

Rules

- Accepting from an inline prefix replaces only the current trigger word or trigger range.
- Accepting from `[` replaces the incomplete bracket trigger.
- If the selected entity can be represented safely as a target-only title-case link, insert `[Name]`.
- If the display text differs from the target or the current context requires a specific display form, insert `[Display](target)`.
- In a Fountain character cue context, insert uppercase display text with an explicit target, for example `[EOGHAN](eoghan)`, so the cue remains valid Fountain while linking to the canonical target.
- In normal action or dialogue text, prefer the entity name casing from front matter, for example `[Eoghan]`.
- If the user has already typed a casing that should be preserved as display text, insert `[typed text](target)` when that is clearer than replacing with the canonical name.
- Insertions participate in undo history as a single edit.
- Insertions mark the document dirty like normal typing.
- Insertions update parser output and processed link rendering.

Acceptance

- Typing `eog` and accepting Eoghan in action text produces `[Eoghan]` or an equivalent valid link.
- Typing `EOG` and accepting Eoghan on a character cue line produces `[EOGHAN](eoghan)`.
- Accepting a prop with display name `Kitchen main door` and target `door-kitchen-main` can produce `[Kitchen main door](door-kitchen-main)`.
- Undo removes the whole inserted link in one step.
- Fountain classification remains correct after inserting a linked character cue.

R6. Autocomplete boundaries and false positives

Status: todo

Avoid intrusive suggestions while still helping normal writing.

Rules

- Inline suggestions should trigger only for a plausible word prefix near the caret.
- Do not trigger inside existing complete link targets unless target editing support is explicitly added.
- Do not trigger inside Markdown image syntax targets.
- Do not trigger inside Markdown fenced code blocks.
- Do not trigger after every single character in normal prose unless the user enabled aggressive suggestions.
- Do not automatically convert typed text into a link without explicit user acceptance.
- Weak potential matches may be shown lower in the list but must not outrank clear prefix matches.

Acceptance

- Writing normal prose does not constantly open irrelevant suggestions.
- Editing `![door](refs/door.png)` does not show script-entity suggestions for the image path.
- The user remains in control of when text becomes a link.

R7. Autocomplete settings

Status: todo

Expose enough configuration to tune interruption level.

Suggested settings

- Enable inline autocomplete
- Enable bracket-trigger autocomplete
- Minimum inline prefix length
- Maximum visible suggestions
- Include automatic scene records in suggestions
- Prefer current document entities

Rules

- Inline autocomplete is enabled by default.
- Bracket-trigger autocomplete is enabled by default.
- Settings persist across app launches.
- Settings changes apply without restarting Basscript.

Acceptance

- The user can disable inline suggestions while keeping `[` suggestions.
- The user can raise the inline prefix length if suggestions appear too often.
- Restarting Basscript preserves autocomplete settings.

Epic S - Story Queries and Navigation

S1. Story query command surface

Status: todo

Provide a way to ask common story-index questions from inside Basscript.

Rules

- The first version can use command-menu entries or a compact story query panel.
- Natural-language AI parsing is not required in v1.
- Commands should cover:
  - props in current scene
  - characters in current scene
  - scenes with selected entities
  - scenes with two characters
  - places for selected character
  - appearances of selected entity
  - backlinks to selected entity
- Commands should use the current cursor scene or selected link when practical.
- If required input is missing, show a small prompt that uses the same autocomplete candidate source.

Acceptance

- From inside a scene, the user can ask for props in that scene.
- From a selected or hovered character link, the user can ask for that character's appearances.
- The user can choose two characters and list scenes they share.

S2. Current scene context

Status: todo

Map editor position to the active scene.

Rules

- The current scene is the nearest containing Fountain scene record for the cursor line.
- Lines before the first scene heading have no current scene.
- Current-scene queries are disabled or show a useful status outside Fountain scenes.
- Split/focus/processed views must all map back to the same raw document line.

Acceptance

- Moving the cursor into a different scene changes current-scene query results.
- Props in current scene uses the scene containing the cursor, not only the visible top scene.
- Query behavior is predictable in split and processed modes.

S3. Result list and navigation

Status: todo

Show query results in a navigable list.

Rules

- Results are grouped by scene, entity, or file depending on query type.
- Results include display name, type, source path, line number, and snippet where useful.
- Pressing `Enter` on a result opens the source file and moves the cursor to the source line.
- Results should be keyboard navigable.
- Empty result sets show a clear status message.
- Stale or partial results are marked as such.

Acceptance

- Selecting an appearance result jumps to the exact script line.
- Scenes shared by two characters are shown in script order.
- Empty results do not look like an indexing failure.

S4. Props in scene

Status: todo

Answer which props are used or mentioned in a scene.

Rules

- Confirmed props come from explicit links to entities with `type: prop`.
- Potential unlinked prop mentions may be shown separately only if weak mention indexing exists.
- Results include the source line where each prop appears.
- Duplicate prop appearances can be collapsed by prop with expandable occurrence details.

Acceptance

- A scene containing `[knife](knife)` lists `knife` as a prop.
- A scene with no linked props returns an empty props result, not guessed data.
- Multiple appearances of the same prop can be traced back to their lines.

S5. Character co-appearance

Status: todo

List scenes that two or more characters share.

Rules

- A character is present in a scene if the index has a confirmed appearance for that character in the scene.
- Character cues count as presence.
- Dialogue spoken by a linked cue counts as presence.
- Action and dialogue mentions count as presence when explicitly linked.
- Results are ordered by script path and script order.
- The query supports at least two characters in v1 and should not prevent more later.

Acceptance

- Querying Eoghan plus another character returns scenes where both are present.
- Scenes where only one of the characters appears are excluded.
- Clicking a scene result navigates to that scene heading or the first matching appearance.

S6. Places for character

Status: todo

List places associated with a character's appearances.

Rules

- For every scene where the character is present, collect:
  - explicit linked place entities in that scene
  - inferred place entity from the scene heading when a place entity matches the heading
  - raw scene-heading location text as a fallback
- Explicit place links rank above inferred heading matches.
- Fallback raw locations must be clearly marked as raw scene locations, not confirmed place entities.
- Results preserve story order and can also provide a unique-place summary.

Acceptance

- If Eoghan appears in `INT. KITCHEN - NIGHT`, the query can report `KITCHEN` as a raw place fallback.
- If `KITCHEN` matches a `place` entity, the query reports the place entity.
- The result can show every scene where the character visited each place.

S7. Backlinks and appearances

Status: todo

Make every entity traceable across the project.

Rules

- Backlinks list every confirmed link to the target across scripts, Markdown files, and canvas text where indexed.
- Appearance results include role and context.
- Backlink navigation opens the source file at the source line where possible.
- Broken links are queryable as unresolved references.

Acceptance

- Opening an entity can show where it is used.
- Broken links can be found from the story index.
- Canvas references appear in backlinks once canvas text indexing is implemented.

Epic T - Verification and Test Coverage

T1. Database and index tests

Status: todo

Acceptance

- Database creation test covers a new workspace.
- Reopen test proves records persist across app sessions or simulated app restarts.
- Rebuild test proves deleting the database reconstructs index data from files.
- Entity indexing tests cover name, target, aliases, type, duplicate target, and invalid front matter.
- Scene indexing tests cover scene heading extraction, scene boundaries, duplicate headings, and cursor-to-scene lookup.
- Appearance indexing tests cover character cues, dialogue speakers, action links, prop links, place links, and unresolved links.

T2. Autocomplete tests

Status: todo

Acceptance

- Matching tests prove `eo`, `Eo`, `EOG`, and `eog` can suggest `Eoghan`.
- Ranking tests prove exact prefix matches outrank weaker fuzzy matches.
- Context tests prove character cue context prefers character entities.
- Insertion tests prove action text, dialogue text, and character cue insert the correct link syntax.
- Undo tests prove accepted suggestions undo as one edit.
- Boundary tests prove autocomplete does not trigger inside Markdown image targets or fenced code blocks.

T3. Query tests

Status: todo

Acceptance

- Props-in-scene query returns linked props in the current scene.
- Character co-appearance query returns only scenes containing both selected characters.
- Places-for-character query returns explicit places, inferred heading places, and raw fallback locations correctly marked.
- Backlink query returns appearances across indexed supported file types.
- Navigation data in query results points to the expected source path and line.

T4. Manual QA checklist

Status: todo

Checklist

- Create or open a workspace with entity Markdown files for a character, prop, and place.
- Type `eo` in action text and verify Eoghan is suggested.
- Type `EOG` on a character cue line and verify accepting the suggestion creates a linked Fountain character cue.
- Use `Up`, `Down`, `Enter`, and `Esc` in the suggestion menu.
- Verify autocomplete does not appear in explorer prompts or settings fields.
- Save a new entity file and verify it appears in autocomplete.
- Rename or delete an entity file and verify the index updates.
- Ask for props in the current scene.
- Ask for scenes shared by two characters.
- Ask for places a character has been.
- Open an entity and inspect backlinks or appearances.
- Delete the story index database and verify Basscript rebuilds it from workspace files.

Implementation notes

Likely implementation areas:

- `core/src/links/entities.rs` for extending entity catalog data used by indexing and suggestions.
- `core/src/parser/fountain.rs` for scene boundaries, character cue behavior, and appearance extraction helpers.
- `core/src/parser/markdown.rs` for Markdown link/backlink indexing boundaries.
- `core/src/canvas.rs` for canvas text/file/link reference indexing.
- A new core module or crate such as `core/src/story_index.rs` or `story_index/` for database schema, indexing, and query APIs.
- `ui/src/editor/core.rs` for editor state, autocomplete state, story-index status, and wiring.
- `ui/src/editor/editing.rs` for trigger detection, accepting suggestions, and undo integration.
- `ui/src/editor/autocomplete.rs` for autocomplete candidate state and keyboard handling if split into a new module.
- `ui/src/editor/command_menu.rs` for story query commands.
- `ui/src/editor/ui_setup.rs` or rendering modules for the suggestion dropdown and query result panel.
- `ui/src/editor/linking/navigation.rs` for opening query results and links.
- `ui/src/pannels/text/explorer_actions.rs` for index updates after create/delete/rename/open actions.
- `settings/state.ron` or app state storage for autocomplete settings and index path metadata.

Design constraints

- Core indexing and query APIs must not depend on Bevy.
- UI rendering of suggestions belongs in `ui`, not `core`.
- Existing script-link syntax remains valid and should not be broken.
- Existing processed-view link colors should be reused where practical.
- Autocomplete must not insert text unless the user accepts a suggestion.
- The Story Index Database is persistent and user-visible through behavior, but project files remain readable and rebuildable.

Suggested implementation order

1. Story index database path, schema version, and empty DB creation.
2. Entity indexing from Markdown front matter.
3. Fountain scene record extraction.
4. Script link and appearance indexing.
5. Basic query API for entities, appearances, current scene, and scenes containing entities.
6. Autocomplete candidate matching from indexed entities.
7. Autocomplete trigger detection in editor text contexts.
8. Suggestion dropdown UI with `Up`, `Down`, `Enter`, and `Esc`.
9. Link insertion rules for action/dialogue text and Fountain character cues.
10. Current-scene query commands.
11. Props-in-scene, character co-appearance, places-for-character, and backlink result views.
12. Incremental index refresh after save/create/delete/rename.
13. Canvas and Markdown backlink expansion.
