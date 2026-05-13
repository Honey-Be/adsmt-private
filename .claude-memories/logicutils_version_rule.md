---
name: adsmt ⇔ logicutils v0.x-smt relationship
description: Versioning rule, immediate-sync rule for kb syntax, and the v1.x merge plan
type: project
originSessionId: 32a1dc0d-7730-4862-8df4-6958199ce84f
---
The adsmt project and the `v0.x-smt` branch of logicutils (vendored at
`external/logicutils/`) follow three coordinated rules during the
pre-1.0 cycle. All three were set by the user; do not deviate without
asking.

## 1. Version offset (set 2026-05-12, confirmed 2026-05-13)

  logicutils v0.x-smt minor = adsmt minor + 2

Examples:
- adsmt v0.1.x  ⇔  logicutils v0.3.x
- adsmt v0.3.x  ⇔  logicutils v0.5.x   (current as of 2026-05-13)
- adsmt v0.5.x  ⇔  logicutils v0.7.x
- adsmt v0.9.x  ⇔  logicutils v0.11.x

**How to apply**: when bumping adsmt's workspace version, also bump
`external/logicutils/Cargo.toml`'s `[workspace.package].version` so
the offset holds. The inline comment in that file documents it.

## 2. Immediate kb-syntax sync (set 2026-05-13)

When an adsmt version bump introduces any lu-kb surface change, the
corresponding logicutils v0.x-smt commit lands **in the same cycle**,
not deferred:

- New kb keyword, AST item, or parser shape  →  add to
  `lu-common/src/kb/{lexer,ast,parser}.rs`.
- New lu-kb block or directive               →  document in
  `docs/man/lu-kb.5`.
- New surface that lu-query/lu-rule should ignore safely → add a
  no-op arm in `lu-query/src/engine.rs` so the workspace still builds.
- Bump logicutils version per rule (1) in the same commit.

**Why**: keeps adsmt and lu-kb in lockstep so downstream tools that
consume kb files never see a syntax error caused by an adsmt feature
that hasn't been mirrored yet. Past breach example: v0.3 adsmt
shipped Boolean/quantifier/datatype work without lu-kb reflection,
which we corrected by adding `enum` syntax in the v0.5 logicutils
bump (option C).

**How to apply**: include the logicutils submodule change in the
same conceptual unit as the adsmt change. Update the parent repo's
submodule pointer afterward so the two repos move together.

## 3. Merge plan at adsmt v1.x (set 2026-05-13)

Once adsmt reaches v1.x stability — the point at which the C ABI,
SMT-LIB dialect, and proof certificate format are committed (per
sec 34 Q68 / Q66) — the two repositories merge into a single
project. The `v0.x-smt` branch is retired; lu-kb and adsmt become
one workspace.

The merge target is a single workspace where:
- the kb language is the user-facing surface,
- the adsmt engine is the canonical kb backend,
- the unified version drops the "+2" offset (becomes 1.0.0 in both).

**How to apply**: don't preemptively merge before v1.x. Until then,
maintain the separation and apply rules (1) and (2). When v1.x lands,
plan the merge as a coordinated release rather than incremental
folding.

**Tracking**: this plan should be re-checked at every v0.x → v0.(x+1)
bump to confirm it's still the chosen path.
