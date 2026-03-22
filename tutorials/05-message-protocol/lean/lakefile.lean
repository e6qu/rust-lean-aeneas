import Lake
open Lake DSL

package MessageProtocol where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib MessageProtocol where
  srcDir := "."
