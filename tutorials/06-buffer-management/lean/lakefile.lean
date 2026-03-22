import Lake
open Lake DSL

package BufferManagement where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

lean_lib Aeneas where
  srcDir := "."

@[default_target]
lean_lib BufferManagement where
  srcDir := "."
