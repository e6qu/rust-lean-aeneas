import Lake
open Lake DSL

-- This project depends on the Aeneas Lean library.
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" / "backends" / "lean"

package InfixCalc where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib InfixCalc where
  srcDir := "."
