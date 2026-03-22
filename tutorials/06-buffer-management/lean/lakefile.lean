import Lake
open Lake DSL

-- This project depends on the Aeneas Lean library.
-- Clone the Aeneas repo and point this path to backends/lean,
-- or use the Aeneas Lean package from GitHub.
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" / "backends" / "lean"

package BufferManagement where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib BufferManagement where
  srcDir := "."
