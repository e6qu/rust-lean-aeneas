import Lake
open Lake DSL

require aeneas from git
  "https://github.com/AeneasVerif/aeneas" / "backends" / "lean"

package StateMachines where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib StateMachine where
  srcDir := "."
