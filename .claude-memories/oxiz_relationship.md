---
name: adsmt ⇔ OxiZ relationship (Path A+B)
description: OxiZ as upstream dependency + collaborator; phased integration plan toward v1.0 unified vision
type: project
originSessionId: 32a1dc0d-7730-4862-8df4-6958199ce84f
---
OxiZ (https://github.com/cool-japan/oxiz, Apache 2.0) is a Pure-Rust
Z3 reimplementation at v0.2.1 (~408k LoC, 6,415 tests, 100% Z3
parity across 8 logics). Discovered 2026-05-13 during the v0.9 SAT
backend survey; user adopted **Path A+B** combining code-level
dependency with active collaboration.

## adsmt's redefined identity (Path B)

adsmt is **"Pure-Rust abductive layer + Lean4 frontend on top of
OxiZ"** — not a from-scratch Z3 alternative.

| | What stays unique to adsmt | What OxiZ provides |
|---|---|---|
| Abductive engine (SLD + minimize + rank + workflow) | ✓ | — |
| HOL+HKT kernel + Type-class layer | ✓ | — |
| Lean4 first-class (FFI + `smt`/`smt_abduce` tactic) | ✓ | — |
| lu-kb integration (logicutils v0.x-smt) | ✓ | — |
| SAT solver | (delegated) | `oxiz-sat` |
| Theory solvers (LIA/LRA/BV/Arrays/Datatypes/FP/Strings/NIA) | (delegated) | `oxiz-theories` |
| Math (Simplex, polynomial, CAD) | (delegated) | `oxiz-math`, `oxiz-nlsat` |
| DRAT/Alethe/LFSC proof export | (partial collab) | `oxiz-proof` |

## Phased integration plan (P1–P5)

| Phase | Cycle | Goal |
|---|---|---|
| **P1: Bridge** | v0.11 | `oxiz_backend` feature in `adsmt-engine` using `oxiz-sat`. Sit alongside `cadical_backend`. |
| **P2: Math** | v0.13 | Import `oxiz-math` for Simplex; retire our v0.9 hand-rolled LIA Fourier-Motzkin |
| **P3: Proof bridge** | v0.15 | Integrate `oxiz-proof` (DRAT/Alethe); our cert layer keeps `assumed` markers + Lean reflection |
| **P4: Coordination** | v0.17 | File issues/PRs on OxiZ — Lean4 binding, abduction trait. Be transparent about adsmt's role. |
| **P5: v1.0 decision** | v0.19 | Either (a) adsmt stays as "OxiZ + Lean4 abductive frontend" with adsmt+logicutils merge, or (b) fold adsmt entirely into OxiZ as `oxiz-lean` / `oxiz-abduce` extension crates |

## v1.0 unified vision

User confirmed 2026-05-13:

> adsmt v1.0 = **adsmt + logicutils + OxiZ** integrated form

This supersedes the earlier "adsmt + logicutils merge only" plan
(see `logicutils_version_rule.md`). The three-project merge resolves
when:
- adsmt's C ABI, SMT-LIB dialect, certificate format stabilize
- OxiZ has matching surface for abductive extensions (P4 outcome
  determines whether this needs PRs or stays in our crates)
- logicutils v0.x-smt branch retires (kb language folded into the
  unified workspace)

## Risk register

| Risk | Mitigation |
|---|---|
| OxiZ bug becomes adsmt bug | Pin specific OxiZ commit; cert layer (Lean kernel) re-verifies the verdict regardless |
| OxiZ breaking change cascades | Semver caution; fork/vendor escape hatch always available |
| OxiZ pivots in unwanted direction | P5 fork option preserved; our differentiated layers stay portable |
| Small-TCB philosophy weakens | Separate "solver TCB" (large, untrusted) from "Lean reflection TCB" (small). Lean kernel is final authority. |
| License compatibility | Apache 2.0 is compatible with our BSD-2-Clause for downstream usage; new contributions to OxiZ flow Apache-2.0; new contributions to adsmt stay BSD-2-Clause |

## How to apply

- When proposing new theory work in adsmt, **first check if OxiZ
  already has it** (it probably does). Default position: delegate
  to OxiZ unless our work needs Lean4-specific or abduction-aware
  modifications.
- New code in `adsmt-theory/` is a smell from v0.11 onward; prefer
  thin adapters over `oxiz-theories`.
- New code in `adsmt-engine/{sat, math, proof}` similar — default
  to OxiZ delegation.
- New code in `adsmt-engine/{abduce, quant}`, `adsmt-class`,
  `adsmt-core`, Lean4 binding: **encouraged**, these are our
  identity.
- Upstream collaboration: open issues on OxiZ describing what
  abductive/Lean4 hooks would help us. Be transparent about
  adsmt's existence and goals.
