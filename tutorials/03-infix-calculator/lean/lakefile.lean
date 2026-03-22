import Lake
open Lake DSL

package InfixCalc where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib InfixCalc where
  srcDir := "."
