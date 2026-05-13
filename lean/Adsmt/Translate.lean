import Lean

/-!
# Lean Expr → SMT-LIB translation

v0.3 first slice: translate a propositional Lean fragment (atoms,
`Not`, `And`, `Or`, conditionals) into SMT-LIB s-expression strings.
The tactic uses this both to generate a witness for the adsmt solver
and to drive the contradiction-detection logic in pure Lean.

Atoms are identified by their canonical Lean expression form
(`toString`), so two structurally equal Props collapse to the same
atom name.
-/

open Lean

namespace Adsmt.Translate

/-- One atom seen during translation. -/
structure Atom where
  name : String
  expr : Expr
deriving Inhabited

/-- Mutable translation state: assignments of canonical atom names. -/
structure State where
  /-- Canonical atom name in source order. -/
  atoms : Array Atom := #[]
  /-- Reverse lookup so repeated atoms collapse to one name. -/
  index : Std.HashMap String Nat := {}
deriving Inhabited

abbrev TM := StateM State

/-- Register `e` as an atom, returning its canonical name. -/
def atomFor (e : Expr) : TM String := do
  let key := toString e
  let s ← get
  if let some i := s.index.get? key then
    return s.atoms[i]!.name
  let idx := s.atoms.size
  let name := s!"a{idx}"
  let next := { s with
    atoms := s.atoms.push { name, expr := e },
    index := s.index.insert key idx,
  }
  set next
  return name

/-- Translate a Prop expression to an SMT-LIB s-expression string.
    Recognises `Not`, `And`, `Or`, `Iff`, and treats anything else as
    an opaque atom. -/
partial def translate (e : Expr) : TM String := do
  match e with
  | .app (.const ``Not _) p => do
      let s ← translate p
      return s!"(not {s})"
  | .app (.app (.const ``And _) p) q => do
      let sp ← translate p
      let sq ← translate q
      return s!"(and {sp} {sq})"
  | .app (.app (.const ``Or _) p) q => do
      let sp ← translate p
      let sq ← translate q
      return s!"(or {sp} {sq})"
  | .app (.app (.const ``Iff _) p) q => do
      let sp ← translate p
      let sq ← translate q
      return s!"(= {sp} {sq})"
  | _ => atomFor e

/-- Run translation on a top-level expression, returning the
    rendered SMT string and the final atom table. -/
def runTranslate (e : Expr) : (String × State) :=
  let m := translate e
  StateT.run m {} |>.run

end Adsmt.Translate
