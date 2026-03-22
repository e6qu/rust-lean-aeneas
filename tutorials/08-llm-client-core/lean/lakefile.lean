import Lake
open Lake DSL


package LlmClientCore where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib LlmClientCore where
  srcDir := "."
