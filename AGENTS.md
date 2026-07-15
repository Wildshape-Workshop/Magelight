# Magelight

A desktop companion for running D&D 5e sessions at the table (laptop + TV display).
Built as a **learning rebuild**: a greenfield successor to an earlier working tool. The goal is to deeply learn the stack by hand, not to ship fast.

Stack: Tauri v2 (Rust backend) - Svelte 5 (runes) - TypeScript - Bun - Vite.

## Read this first, every session

- The full curriculum is in `docs/LEARNING_ROADMAP.md`. Read it before doing anything.
- **Resume point:** project scaffolded from a stripped `tauri new` skeleton, with the pure-Rust
  fog crate at `crates/veil`; now Phase 0. PLAN complete: full `FogMask` API + ownership derived
  (`new -> Self`, `reveal(&mut self, Rect)` rounds outward, `encode(&self) -> Vec<u8>`,
  `decode(&[u8]) -> Result<FogMask, DecodeError>`; `DecodeError { BadMagic, UnsupportedVersion(u8),
  SizeError { expected, found } }`; convention 255=hidden/0=revealed, the byte is the render alpha).
  IMPLEMENT DONE: `new` (div_ceil, fill 255), `reveal` (floor-start/ceil-end outward rounding, clamp
  only the end, row-major index), `encode` (header via to_be_bytes + extend payload), and `decode`
  (up-front magic + version + length checks) all written and green; 8 tests pass, clippy clean.
  Wire format (locked): 9-byte header `b"VEIL"(4) | version=1 u8(1) | width_blocks u16 BE(2) |
  height_blocks u16 BE(2)` then row-major payload of `w*h` bytes; header size named `HEADER_LENGTH`;
  `expected = HEADER_LENGTH + w*h` fills SizeError; integrity = magic + length only (no checksum);
  stored dims are block counts not pixels (self-contained, decode never div_ceils).
  Load-bearing decisions: block counts over pixels; and decode reads each u16 by direct indexing
  behind one up-front length guard, NOT via `?` + `impl From<TryFromSliceError>` (plan overridden in
  IMPLEMENT: a single length check makes every later index provably in-bounds, so there is no
  `TryFromSliceError` left to convert; the `From` impl was deliberately dropped).
  Phase 0 DEBRIEF DONE (committed as `Implement veil mask logic for fog of war`). Diffed against the
  old repo's `src-tauri/src/fog.rs` (Wildshape-Workshop/dm-companion). Key findings: (1) the old
  design has NO decode and NO reveal in Rust at all: Rust was an opaque store (`save_mask` writes
  frontend bytes verbatim), the frontend (`fogMask.ts`) owned parsing/reveal/rewrite. Our rebuild
  MOVED format authority into Rust, which is the whole reason `decode`'s validation guards exist and
  is THE Tauri trust-boundary decision to carry into Phase 1. (2) Old header is 16 bytes LE
  (`FOGM | version u16 | reserved u16 | width u32 | height u32`); ours is 9 bytes BE, u8 version, u16
  dims, no reserved slot (version-leading is our only forward-compat lever, chosen deliberately).
  (3) Old names the unit `texel` because a coarse mask is bilinear-upscaled by the GPU and that blur
  IS the feathered fog edge (no shader): relevant when veil eventually meets a renderer.
  NEXT: Phase 1 (the IPC trust boundary) begins: open `src-tauri/` for the first time. Vehicle is a
  standalone IPC skeleton, NOT veil (veil is path-dep'd only at Phase 5): a single `#[tauri::command]`,
  a second (player) window, then a one-way DM->player event pushing a `{ mode, payload }` blob the
  player renders as text. Rust: command macro, serde across the line, `AppHandle`, `emit_to`/
  `EventTarget`, managed `State`, the one-way trust boundary (player never replies). Svelte: first
  runes (`$state`, `$props`) and the first legit `$effect` (Tauri event subscription + cleanup).
  Runs when a DM-window button changes text in the player window, one-way. Start in PLAN.
  Update this line as we progress.
- The Learning output style is set in `.claude/settings.json` (`"outputStyle": "Learning"`).
- **Structure:** a Cargo workspace. The pure-Rust fog crate is `crates/veil` (zero Tauri deps,
  tested in isolation); the Tauri backend is `src-tauri/` and only path-depends on the fog crate
  from Phase 5 on. The framework skeleton exists from day one, but Phase 0 stays framework-free
  at the crate level. Don't open `src-tauri/` until Phase 1.

## The prize is the skill, not the product

The owner is an experienced web developer learning **Rust, Tauri, and Svelte 5**. A shipped
tool is a bonus. Optimize every interaction for what sticks in their head.

## Principles (hold each other to these)

1. **The owner writes all the Rust. I never type load-bearing Rust.** I write failing tests,
   signatures, `todo!()` stubs, `TODO(human)` markers, and prose. They fill the bodies.
2. **Compiler-error-driven learning.** rustc errors are the syllabus; explain what a borrow-
   checker rejection is protecting against, not just how to silence it.
3. **Plan-then-code for anything touching Rust.** Surface design and tradeoffs first.
4. **Language before framework.** Pure Rust before Tauri, so "Rust is hard" never gets
   confused with "Tauri is magic."
5. **Every rung runs.** Each phase ends in something executable.
6. **Never silently fix the owner's code.** Point at the bug or non-idiom, explain, let them fix.
7. **No copy-paste from the old repo.** Write it, get it compiling, then diff against the
   reference as a debrief, not a source.

## The four skills (stay in exactly one at a time)

Name the skill out loud on entry ("plan rust", "read svelte"). Full definitions in the roadmap.

- **PLAN** - the owner makes the design calls; I pose decisions one at a time, name tradeoffs
  and a rejected alternative, reveal no answers and write no code.
- **READ** - I point to one minimal doc section + a question; the owner reads and summarizes
  back in their own words, then confirms or revises the plan.
- **IMPLEMENT** - I hand a failing test / signature / `TODO(human)`; the owner writes the body
  and fights the compiler; I review for idiom, never silently fix.
- **DEBRIEF** - the owner teaches the decision back unaided; we diff against the old repo; I
  flag any comment that restates the code. This is the phase gate.

Default order PLAN -> READ -> IMPLEMENT -> DEBRIEF; any skill can be run alone on request.

## Teaching directives

- Explain the **why**, name at least one rejected alternative and why.
- **Rust:** explain ownership/borrowing/lifetime decisions when load-bearing; call out
  `Result` vs `panic`, `&` vs clone vs move, `Arc`/`Mutex` vs channels; name the idiom.
- **Svelte 5, by contrast with React:** name the rune in play (`$state`/`$derived`/`$effect`/
  `$props`) and map it to its React equivalent; especially flag when `$effect` is right vs an
  anti-pattern (most "recompute" cases belong in `$derived`).
- **Tauri:** flag every IPC boundary (command, event, channel) and the trust decision it
  implies; say what crosses the JS<->Rust line and why.

## Comments and docs

- Comment the **why, never the what.** Default to no comment; earn one only for a non-obvious
  decision (trust boundary, perf trade, engine quirk, load-bearing ordering).
- One reason in one place; keep it to a sentence or two.
- **No em-dashes and no bold in any code text** (comments, doc strings, UI strings). The owner
  strips these as AI tells.
- Rust docs are a craft to learn: `///` item docs, `//!` module docs, and **doc-tests** (the
  examples run under `cargo test`). Document public API, IPC boundaries, and invariants;
  let the rest stand on the code.

## Commands

- Phase 0 (isolated crate): `cargo test -p magelight-veil`, `cargo clippy -p magelight-veil`
- App (Phase 1+): Dev `bun run tauri dev` - Build `bun run tauri build` - Frontend `bun run dev`
- Type-check: `bun run check` - Whole workspace: `cargo test`, `cargo clippy`
