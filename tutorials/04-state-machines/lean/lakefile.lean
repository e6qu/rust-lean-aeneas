import Lake
open Lake DSL

package StateMachines where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib StateMachine where
  srcDir := "."
