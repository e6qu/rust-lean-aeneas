import Lake
open Lake DSL


package FullIntegration where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib FullIntegration where
  srcDir := "."
