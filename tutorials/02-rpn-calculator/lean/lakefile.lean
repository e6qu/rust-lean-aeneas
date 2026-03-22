import Lake
open Lake DSL

-- This project depends on the Aeneas Lean library.
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" / "backends" / "lean"

package RpnCalc where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib RpnCalc where
  srcDir := "."
