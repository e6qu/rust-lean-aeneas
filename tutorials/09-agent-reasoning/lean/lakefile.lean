import Lake
open Lake DSL

require aeneas from git
  "https://github.com/AeneasVerif/aeneas" / "backends" / "lean"

package AgentReasoning where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib AgentReasoning where
  srcDir := "."
