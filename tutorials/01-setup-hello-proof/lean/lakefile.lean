import Lake
open Lake DSL

package HelloProof where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib HelloProof where
  srcDir := "."
