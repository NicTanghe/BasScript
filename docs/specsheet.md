Basscript Fountain Editor

Bevy 0.18 screenplay editor with line-aware formatting (core/ui split)

Epic A - Workspace & foundations (NON-NEGOTIABLE)
A1. Workspace layout

Depends: -
Status: todo

Cargo workspace:

core/ - document model, buffer, parser interfaces

ui/ - Bevy UI plugin, rendering, input

app/ - App entrypoint (wires core + ui)

docs/ - Specs and architecture notes

Acceptance

cargo run launches the app (via workspace default)

No UI types leak into core

A2. Bevy 0.18 baseline app

Depends: A1
Status: todo

Minimal Bevy app with default plugins

Single window and camera setup

Acceptance

App opens a window on desktop platforms

A3. Core/UI contract surface

Depends: A1
Status: todo

Core exposes:

Document API (text + lines)

Cursor + Selection model

Parser output model (per-line nodes)

UI consumes core only through public types

Acceptance

core builds without Bevy

ui builds without direct access to parser internals

Epic B - Document model & editing
B1. Text buffer (line-first)

Depends: A3
Status: todo

Rope-based storage (ropey or equivalent)

Line index + char index operations

Acceptance

Insert/delete are sublinear

Line lookup is stable for large docs

B2. Cursor & selection model

Depends: B1
Status: todo

Cursor { line, column, preferred_column }

Selection { anchor, head }

Acceptance

Cursor always valid after edits

Selection supports multi-line ranges

B3. Edit operations

Depends: B2
Status: todo

Insert char

Backspace/delete

Newline + line join

Dirty-line tracking for parser + renderer

Acceptance

Edits update cursor and dirty ranges correctly

Epic C - Fountain parsing (custom, line-based)
C1. Parser interface (placeholder)

Depends: A3, B1
Status: todo

Line-oriented parser entrypoint

Returns per-line node classification

Acceptance

Parser can be stubbed without breaking UI

C2. Incremental parse rules

Depends: C1
Status: todo

Reparse edited line + neighbors

Dialogue blocks resolved via adjacent lines

Acceptance

No full reparse on every keystroke

C3. Fountain grammar subset v0

Depends: C1
Status: todo

Scene headings

Action

Character

Dialogue

Parenthetical

Transition

Empty / Unknown

Acceptance

Unknown lines fall back to Action

Epic D - Rendering & view modes
D1. View modes

Depends: A2, B2, C1
Status: todo

Focus mode: current line raw, others formatted

Raw mode: full raw text

Processed mode: full formatted text

Split mode: raw + processed

Acceptance

Mode switch is instant without reparse

D2. Line rendering pipeline

Depends: D1
Status: todo

Render per-line entities (virtualized window)

Line layout cache (positions + bounds)

Per-node styling

Acceptance

Only visible lines are spawned/rendered

D3. Screenplay styling

Depends: D2
Status: todo

Monospace font (v1)

Indent + spacing constants

Style map per node type

Acceptance

Formatting is consistent across lines

Epic E - Cursor metrics & hit-testing
E1. Glyph advance cache (authoritative)

Depends: A2, B1
Status: todo

Compute glyph advances from Bevy font metrics

Apply DPI scaling

No font_size * aspect_ratio approximation

Acceptance

Caret aligns exactly with rendered glyphs

E2. Mouse hit-testing

Depends: E1
Status: todo

Map mouse Y to line bounds

Binary-search X in glyph positions

Acceptance

Clicks place cursor in correct column

E3. Caret + selection rendering

Depends: E1, B2
Status: todo

Caret quad with blink timer

Selection rectangles per line

Acceptance

Caret and selection render at correct positions

Epic F - UI shell & input
F1. Input handling

Depends: B3, E2
Status: todo

Keyboard input (text + navigation)

Mouse click + drag selection

Acceptance

Typing and navigation never block rendering

F2. Tabs / view switcher

Depends: D1
Status: todo

Tab bar: Focus / Raw / Processed / Split

Optional command palette later

Acceptance

View state is preserved across switches

Constraints (explicit)

- No Fountain library (custom parser provided later)

- Bevy 0.18

- Core and UI are separate crates

- Cursor positioning uses real glyph advances

- No IME / bidi / shaping in v1

Suggested implementation order (fastest to "feels real")
A1 + A2 + A3
B1 + B2
E1 + E2
D1 + D2
C1 + C2
B3
F1 + F2
C3 + D3
Polish: parsing rules, styling, performance tuning