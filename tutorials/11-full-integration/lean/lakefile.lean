import Lake
open Lake DSL

require aeneas from git
  "https://github.com/AeneasVerif/aeneas" / "backends" / "lean"

package FullIntegration where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib FullIntegration where
  srcDir := "."
