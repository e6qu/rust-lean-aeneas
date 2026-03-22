import Lake
open Lake DSL


package TuiCore where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib TuiCore where
  srcDir := "."
