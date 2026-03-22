import Lake
open Lake DSL


package MultiAgent where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib MultiAgent where
  srcDir := "."
