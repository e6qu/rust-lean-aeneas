import Lake
open Lake DSL

require aeneas from git
  "https://github.com/AeneasVerif/aeneas" / "backends" / "lean"

package MessageProtocol where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib MessageProtocol where
  srcDir := "."
