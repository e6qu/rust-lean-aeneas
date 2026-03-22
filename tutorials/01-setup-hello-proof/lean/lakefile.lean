import Lake
open Lake DSL

-- This project depends on the Aeneas Lean library.
-- To use this, clone the Aeneas repo and point this path to backends/lean.
-- Alternatively, use the Aeneas Lean package from GitHub.
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" / "backends" / "lean"

package HelloProof where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib HelloProof where
  srcDir := "."
