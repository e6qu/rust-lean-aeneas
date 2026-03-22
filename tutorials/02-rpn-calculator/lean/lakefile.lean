import Lake
open Lake DSL

package RpnCalc where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib RpnCalc where
  srcDir := "."
