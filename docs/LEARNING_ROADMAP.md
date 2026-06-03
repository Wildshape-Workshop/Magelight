# Learning Roadmap: a learning-first D&D DM companion (greenfield rebuild)

## Context

An earlier working tool was built in a fast-iteration mode where the AI generated most of the Rust. The result: ~3,900 LOC across 17
files the owner can read but never built a mental model for. The Rust side is "underlit."

This is a greenfield restart, new repo and name, to reach v1 **by hand-coding every line of
Rust** with the AI coaching (explain concepts, link docs, make it click) rather than typing.
The prize is the **skill**, not the product. A shipped tool is a welcome bonus.

This is a curriculum disguised as a project. The old repo stays put as a **reference
implementation** to diff against after each piece is written by hand (never copy-pasted from).

## Principles (the rules we hold each other to)

1. **You write all the Rust. I never type load-bearing Rust for you.** I write failing tests,
   function signatures, `todo!()` stubs, and prose. You fill the bodies.
2. **Compiler-error-driven learning.** rustc's errors are the syllabus. When the borrow
   checker rejects your code, that rejection *is* the lesson; I explain what it's protecting
   against, not just how to silence it.
3. **Plan-then-code for anything touching Rust.** Design and tradeoffs surface before you write.
4. **Language before framework.** The project is scaffolded with the Tauri skeleton from day
   one, but Phase 0 happens in an isolated `crates/veil` library with zero Tauri dependencies,
   tested on its own. The framework exists in the repo; it stays out of your way until Phase 1,
   so you never confuse "Rust is hard" with "Tauri is magic."
5. **Every rung runs.** No phase is "just learning" with nothing to show.
6. **I don't silently fix your code.** I point at a bug or non-idiom and explain; you fix it.
7. **No copy-paste from the old repo.** Write it, get it compiling, then compare. The diff is
   a debrief, not a source.

## How a session works: four targeted skills, one at a time

Stay in exactly one skill at a time so the thing being trained is never blurred. Each applies
to whichever domain is in play (`rust` / `svelte` / `docs`). Name the skill out loud on entry
("plan rust", "read svelte") and don't drift.

1. **PLAN** (design before any code or docs).
   - *You:* make the decisions - API surface, ownership (`self` vs `&self` vs `&mut self`),
     data shape, error strategy, which rune. Reason from what you already know.
   - *Me:* pose the decision points one at a time, name the tradeoff and one rejected
     alternative, push back on a weak choice. I write zero code and reveal zero answers.
   - *Done when:* you can state the plan in a few bullets and name the load-bearing decision.

2. **READ** (learn from primary sources).
   - *You:* read the one specific doc section I point to, then summarize back in your words the
     part that answers the open question; confirm or revise the plan.
   - *Me:* pick the minimal exact target and a focused question; check your summary, fill gaps.
   - *Done when:* you can explain the concept and apply it to our specific case.

3. **IMPLEMENT** (write it, fight the compiler).
   - *You:* fill the bodies / write the component; run `cargo test` / `clippy` / `bun run check`.
   - *Me:* hand you a failing `#[test]` / signature / `TODO(human)`, review for idiom, read
     compiler errors *with* you, never silently fix.
   - *Done when:* green and idiomatic.

4. **DEBRIEF** (consolidate; this is the phase gate).
   - *You:* explain back, unaided, the decision the unit forced.
   - *Me:* diff against the old repo, show where it chose differently and why; flag any comment
     that restates the code.
   - *Done when:* you can teach it back.

Default order PLAN -> READ -> IMPLEMENT -> DEBRIEF, but any skill can run alone on request
("today, only plan the next three signatures").

## Three learning threads (the spine carries all three)

- **Rust** - ownership, error handling, concurrency, async, traits. The reason for the rebuild.
- **Svelte 5, learned by contrast with React.** Map each idea onto its React equivalent:
  - `$state` vs `useState` - fine-grained signal, mutate in place, no setter.
  - `$derived` vs `useMemo` - no dependency array; the compiler tracks reads.
  - `$effect` vs `useEffect` - looks identical, anti-pattern far more often (most "recompute"
    cases belong in `$derived`); learn the *when-not-to* explicitly.
  - `$props` vs props/`forwardRef`; `bind:` two-way binding (no React analog).
  - `.svelte.ts` rune-stores vs Context/Redux/Zustand.
  - snippets vs `children`/render props; fine-grained reactivity vs VDOM diffing.
- **Documentation discipline, learned as a craft.**
  1. *Philosophy:* comment the **why, never the what**; default to no comment; one reason in
     one place; no em-dashes or bold.
  2. *Mechanics:* rustdoc (`///`, `//!`) and **doc-tests** (examples run under `cargo test`),
     `cargo doc --open`; TSDoc/JSDoc focused on the IPC client and store contracts. Decide what
     earns documentation (public API, IPC boundaries, invariants) vs what stands on its own.

## The concept ladder (Phase 0 -> v1)

Sequenced by conceptual coupling, lowest first.

### Phase 0 - Pure Rust, no framework: the fog-mask crate (a workspace member)
**Vehicle:** the fog-of-war bitmap mask, written as the workspace member `crates/veil` with zero
Tauri dependencies and tested in isolation (`cargo test -p magelight-veil`): a header (magic +
version + dimensions) plus one alpha byte per 8x8 block, with `encode`/`decode` and a
`reveal(rect)` mutation. It lives in the real project from day one; `src-tauri` only path-depends
on it at Phase 5.

**Rust concepts:** ownership and borrowing (`&self` vs `&mut self`, `&[u8]` vs `Vec<u8>`);
structs and enums; `match`; `Result` + a custom error enum; the `?` operator; one `From` impl;
`Option` combinators; modules and visibility; unit tests with `#[cfg(test)]`.

**Docs:** `///` on the public API, `//!` on the module, your first doc-test, `cargo doc --open`.

**Why first:** zero framework, pure language; produces the actual byte format wired in at Phase 5.

**Runs when:** `cargo test` green for an encode->decode round-trip and a reveal op.

### Phase 1 - The IPC trust boundary (on the already-scaffolded skeleton)
**Vehicle:** the skeleton already exists (Phase 0 setup). Now add the IPC: a single
`#[tauri::command]`; a second (player) window; then a one-way DM->player event pushing a
`{ mode, payload }` blob the player renders as text.

**Rust concepts:** the `#[tauri::command]` macro; serde across the JS<->Rust line; `AppHandle`;
`emit_to` / `EventTarget`; managed `State`; the one-way trust boundary (player never replies).

**Svelte (vs React):** first runes - `$state` (mutate in place, no setter), `$props`; the first
`$effect` decision (a Tauri event subscription is a *legitimate* `$effect`: external
subscription + cleanup), contrasted with cases that should be `$derived`.

**Docs:** TSDoc on the typed IPC client wrapper - documenting the contract that crosses the line.

**Runs when:** a DM-window button changes text in the player window, one-way.

### Phase 2 - Persistence: SQLite the right way
**Vehicle:** `rusqlite` (bundled) as managed `Mutex<Connection>`; a settings table and a maps
catalog; schema versioning via `PRAGMA user_version` with an ordered migration list.

**Rust concepts:** `Mutex<T>` and *why* (`Connection` is `Send` but not `Sync`);
`State<'_, Mutex<Connection>>` and the lifetime; lock poisoning; `query_row`/`execute`/
`query_map`; transactions; converting `rusqlite::Error` at the IPC boundary.

**Runs when:** a setting written from the DM window survives a restart.

### Phase 3 - Assets + custom protocol (the meaty Rust)
**Vehicle:** import a map image (dialog plugin) -> decode -> resize -> encode WebP -> write
artifacts -> store a row. Then a `map://<id>` custom URI scheme streaming the display variant.

**Rust concepts:** error enums with multiple `From` impls; the async/blocking split
(`spawn_blocking` so decode never stalls the webview); why you drop the Mutex before the heavy
work; closures moved into async tasks; `AppHandle` cloning into spawned work; the path-safety
trust gate (validate the id charset before building any path).

**Runs when:** an imported map renders in the player window from a `map://` URL.

### Phase 4 - The rendering hot path (performance is the feature)
**Vehicle:** Canvas2D render, source->screen transform math, pan/zoom camera, a count-bounded
LRU of `ImageBitmap`s in the player window, a dispatch->paint perf probe.

**Concepts:** mostly TS/Svelte + architecture; the 16ms cache-hit push budget lives here.

**Svelte (vs React), the Svelte-heavy phase:** `$derived` for the source->screen transform
(no dependency array, contrast `useMemo`); `.svelte.ts` rune-stores replacing Context/Redux;
fine-grained reactivity vs VDOM diffing and what it means for the render loop; the `$effect`
that owns the `requestAnimationFrame`/canvas draw, and why canvas rendering justifies `$effect`.

**Runs when:** pushing a cached map paints within one frame, measured by the probe.

### Phase 5 - Fog of war, end to end (the payoff)
**Vehicle:** wire the Phase 0 fog crate in: brush reveal on the DM side, streamed fog deltas
over a dedicated one-way event channel, bitmap mask compositing on the player.

**Concepts:** integrating a hand-built crate; ordered+accumulative vs latest-wins event
semantics; the mask as an opaque format Rust relays but doesn't parse.

**Runs when:** brushing on the DM map reveals the corresponding area on the player display.

### Phase 6 - Tokens (the last v1 rung, where it gets genuinely useful)
**Vehicle:** token CRUD on a map; a `token://<id>` protocol sibling of `map://`; hidden-on-DM
vs public-on-player visibility; a token portrait import pipeline reusing the Phase 3 encoder.

**Rust concepts (consolidation):** a second protocol reusing the Phase 3 trust gate; typed row
structs with `#[serde(rename_all = "camelCase")]`; an index read. Deliberately repeats Phases
2-3 so you prove the patterns are reflexive.

**Svelte (vs React):** `bind:` two-way binding for token drag (no React analog); a derived list
of tokens; the hidden/public split as DM-truth vs player-truth in reactive state.

**Docs:** documenting an invariant in code (the one-way token-move channel and visibility rule).

**Runs when:** a token dragged on the DM map moves on the player; a hidden token shows on the
DM working view but not on the player.

## v1 scope boundary

**In v1:** DM control surface + player window, map import, push map (`blank` | `map`),
pan/zoom camera, fog-of-war brush reveal, and tokens (Phase 6, added last).

**Post-v1 (cut from the learning ladder):** backup/restore, splash, handouts, auto-update,
display hotplug handling. They mostly repeat concepts already learned; add them once the
fundamentals are reflexive.

## Tooling & reading

- **Tooling:** rust-analyzer; `cargo clippy` (warnings are lessons); `cargo test`;
  `bun run tauri dev`.
- **References:** The Rust Book (chs. 4, 6, 9, 10, 13, 16); Rust by Example; Tauri v2 docs.
  Later: "Rust for Rustaceans" once Phases 0-3 feel comfortable.

## First session (concrete start)

0. Scaffold: `bun create tauri-app` (Svelte 5 + TS), choose the name, strip the sample command
   and boilerplate UI down to a minimal window that runs with `bun run tauri dev`.
1. Make it a Cargo workspace and add the empty `crates/veil` library (zero Tauri deps).
2. PLAN (rust): derive the `FogMask` API and the ownership of each method together (the
   `reveal()` self/&self/&mut self decision is the opener). Then READ to confirm against the Book.
3. IMPLEMENT: I write a failing encode->decode round-trip test and `todo!()` signatures; you fill
   the bodies until `cargo test -p magelight-veil` is green.
4. DEBRIEF: diff against the old `src-tauri/src/fog.rs`.

## Verification (both senses)

- **The app works:** each phase's "Runs when" line is a concrete manual check before advancing.
- **The learning landed:** at each phase gate you explain back, unaided, (a) the Rust
  ownership decision the phase forced, (b) any Svelte-vs-React contrast it surfaced (especially
  an `$effect` justify-or-reject call), and (c) what earned a comment/doc and what didn't. If
  any is shaky, don't advance; write one more small exercise on the weak spot first.
