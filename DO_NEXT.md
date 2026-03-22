# What To Do Next

## Immediate: Tutorial 01 End-to-End Verification

Prove the concept works by fully verifying tutorial 01:

### Step 1: Restructure Lean project
```
tutorials/01-setup-hello-proof/lean/
├── lakefile.lean              # require aeneas from git @ "main" / "backends" / "lean"
├── lean-toolchain             # match Aeneas's toolchain (v4.28.0-rc1)
├── HelloProof.lean            # REAL Aeneas output (copy from generated/)
└── HelloProof/
    └── Proofs.lean            # Real proofs against real generated code
```

### Step 2: Write real proofs for tutorial 01
Using patterns from the Aeneas ICFP tutorial:

```lean
import HelloProof
open Aeneas Aeneas.Std Result hello_proof

-- clamp never panics and result is in [lo, hi]
@[pspec]
theorem clamp_spec (x lo hi : Std.I32) (h : lo ≤ hi) :
    ∃ r, clamp x lo hi = ok r ∧ lo ≤ r ∧ r ≤ hi := by
  rw [clamp]
  split <;> split <;> simp_all
  all_goals (constructor <;> scalar_tac)

-- safe_divide by zero returns Err
@[pspec]
theorem safe_divide_zero (x : Std.I64) :
    safe_divide x 0#i64 = ok (core.result.Result.Err ()) := by
  rw [safe_divide]; simp

-- checked_add never panics
@[pspec]
theorem checked_add_no_panic (x y : Std.U32) :
    ∃ r, checked_add x y = ok r := by
  rw [checked_add]
  progress as ⟨i⟩       -- U32.MAX - x
  split
  · progress as ⟨i1⟩    -- x + y
    exact ⟨some i1, rfl⟩
  · exact ⟨none, rfl⟩
```

### Step 3: Verify locally
```bash
cd tutorials/01-setup-hello-proof/lean
lake build  # Must succeed — this IS the formal verification
```

### Step 4: Update CI
Change Lean CI jobs to use real Aeneas library. Cache the Aeneas build (~1600 modules).

## Then: Fix Rust and Write Proofs for Tutorials 02-06

For each tutorial:
1. Fix any Rust patterns that Aeneas can't translate
2. Re-run `charon` + `aeneas` to get clean generated Lean
3. Set up lakefile with real Aeneas dependency
4. Write real proofs using `step`/`progress`/`scalar_tac`
5. Verify with `lake build`

## Then: Fix Tutorials 07-11

These need Rust refactoring to avoid unsupported Aeneas patterns (see GAP_ANALYSIS.md Gap 5).

## Key Aeneas Proof Patterns to Use

From the ICFP tutorial solutions:

```lean
-- Pattern 1: Unfold + progress through monadic steps
theorem foo_spec (x : U32) (h : ↑x + 1 ≤ U32.max) :
    ∃ y, foo x = ok y ∧ ↑y = ↑x + 1 := by
  rw [foo]
  progress as ⟨y⟩
  scalar_tac

-- Pattern 2: Branching (if/match)
theorem bar_spec (x : I32) :
    ∃ r, bar x = ok r := by
  rw [bar]
  split          -- case split on if-then-else
  · simp_all     -- true branch
  · simp_all     -- false branch

-- Pattern 3: Loop invariant
@[pspec]
theorem loop_spec (x : Vec U32) (i : Usize) (h : i ≤ x.length) :
    ∃ x', loop x i = ok x' ∧ x'.length = x.length := by
  rw [loop]
  split
  · progress as ⟨...⟩   -- loop body
    progress as ⟨x', hx'⟩  -- recursive call (uses this theorem via @[pspec])
    simp_all
  · simp_all
termination_by (x.length - i.val).toNat
decreasing_by scalar_decr_tac

-- Pattern 4: Composing specs
-- If foo has @[pspec], then `progress` automatically uses it:
theorem baz_spec (x : U32) (h : ...) :
    ∃ y, baz x = ok y ∧ ... := by
  rw [baz]
  progress as ⟨y, hy⟩   -- auto-applies foo_spec
  scalar_tac
```

## Definition of Done

A tutorial is "formally verified" when:
1. Rust code compiles and passes tests
2. `charon cargo --preset=aeneas` produces clean LLBC (no errors)
3. `aeneas -backend lean *.llbc` produces clean Lean (no errors)
4. Proof file contains real `theorem` (not `axiom`) with real tactic proofs (not `sorry`)
5. `lake build` with real Aeneas library succeeds
6. CI enforces all of the above
