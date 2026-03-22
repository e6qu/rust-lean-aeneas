import Lake
open Lake DSL


package AgentReasoning where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib AgentReasoning where
  srcDir := "."
