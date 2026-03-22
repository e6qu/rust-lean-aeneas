import Lean
/-!
  # Aeneas Prelude

  Self-contained replacement for `import Aeneas`. Provides the types,
  operations, coercions, and tactic stubs that Aeneas-generated Lean files
  expect. Bounded integer types wrap `Nat` / `Int` without carrying proof
  obligations, keeping this prelude simple and portable.
-/
set_option autoImplicit true

-- ============================================================================
-- Custom attributes (no-ops in standalone mode)
-- ============================================================================

initialize stepAttr : Lean.TagAttribute ←
  Lean.registerTagAttribute `step "Aeneas step lemma (no-op in standalone mode)"

initialize rustLoopAttr : Lean.TagAttribute ←
  Lean.registerTagAttribute `rust_loop "Aeneas rust_loop marker (no-op in standalone mode)"

-- ============================================================================
-- Result type
-- ============================================================================

namespace Aeneas

inductive Error where
  | panic
  deriving Repr, BEq

inductive Result (α : Type u) where
  | ok   : α → Result α
  | fail : Error → Result α
  deriving Repr, BEq

namespace Result

@[inline] def bind (r : Result α) (f : α → Result β) : Result β :=
  match r with
  | ok v   => f v
  | fail e => fail e

instance : Monad Result where
  pure := ok
  bind := bind

instance : Inhabited (Result α) where
  default := fail .panic

end Result

export Result (ok fail)

-- ============================================================================
-- Bounded integer types (lightweight wrappers — no proof obligations)
-- ============================================================================

structure U8    where val : Nat   deriving Repr, DecidableEq, BEq, Inhabited
structure U16   where val : Nat   deriving Repr, DecidableEq, BEq, Inhabited
structure U32   where val : Nat   deriving Repr, DecidableEq, BEq, Inhabited
structure U64   where val : Nat   deriving Repr, DecidableEq, BEq, Inhabited
structure Usize where val : Nat   deriving Repr, DecidableEq, BEq, Inhabited
structure I8    where val : Int   deriving Repr, DecidableEq, BEq, Inhabited
structure I16   where val : Int   deriving Repr, DecidableEq, BEq, Inhabited
structure I32   where val : Int   deriving Repr, DecidableEq, BEq, Inhabited
structure I64   where val : Int   deriving Repr, DecidableEq, BEq, Inhabited

-- ============================================================================
-- Bounds constants
-- ============================================================================

def U8.max  : Nat := 255
def U16.max : Nat := 65535
def U32.max : Nat := 4294967295
def U32.MAX : U32 := ⟨4294967295⟩
def U64.max : Nat := 18446744073709551615
def Usize.max : Nat := 18446744073709551615

def I8.min  : Int := -128
def I8.max  : Int := 127
def I16.min : Int := -32768
def I16.max : Int := 32767
def I32.min : Int := -2147483648
def I32.max : Int := 2147483647
def I64.min : Int := -9223372036854775808
def I64.max : Int := 9223372036854775807
def I64.MIN : I64 := ⟨I64.min⟩

-- ============================================================================
-- Coercions
-- ============================================================================

instance : Coe U8    Nat where coe x := x.val
instance : Coe U16   Nat where coe x := x.val
instance : Coe U32   Nat where coe x := x.val
instance : Coe U64   Nat where coe x := x.val
instance : Coe Usize Nat where coe x := x.val
instance : Coe I8    Int where coe x := x.val
instance : Coe I16   Int where coe x := x.val
instance : Coe I32   Int where coe x := x.val
instance : Coe I64   Int where coe x := x.val

instance : OfNat U8    n where ofNat := ⟨n % 256⟩
instance : OfNat U16   n where ofNat := ⟨n % 65536⟩
instance : OfNat U32   n where ofNat := ⟨n % 4294967296⟩
instance : OfNat U64   n where ofNat := ⟨n % 18446744073709551616⟩
instance : OfNat Usize n where ofNat := ⟨n % 18446744073709551616⟩
instance : OfNat I8    n where ofNat := ⟨(n : Int)⟩
instance : OfNat I16   n where ofNat := ⟨(n : Int)⟩
instance : OfNat I32   n where ofNat := ⟨(n : Int)⟩
instance : OfNat I64   n where ofNat := ⟨(n : Int)⟩

-- ============================================================================
-- Ordering / comparison
-- ============================================================================

instance : LT U8    where lt a b := a.val < b.val
instance : LT U16   where lt a b := a.val < b.val
instance : LT U32   where lt a b := a.val < b.val
instance : LT U64   where lt a b := a.val < b.val
instance : LT Usize where lt a b := a.val < b.val
instance : LT I32   where lt a b := a.val < b.val
instance : LT I64   where lt a b := a.val < b.val

instance : LE U8    where le a b := a.val ≤ b.val
instance : LE U16   where le a b := a.val ≤ b.val
instance : LE U32   where le a b := a.val ≤ b.val
instance : LE U64   where le a b := a.val ≤ b.val
instance : LE Usize where le a b := a.val ≤ b.val
instance : LE I32   where le a b := a.val ≤ b.val
instance : LE I64   where le a b := a.val ≤ b.val

instance (a b : U8)    : Decidable (a < b) := Nat.decLt _ _
instance (a b : U16)   : Decidable (a < b) := Nat.decLt _ _
instance (a b : U32)   : Decidable (a < b) := Nat.decLt _ _
instance (a b : U64)   : Decidable (a < b) := Nat.decLt _ _
instance (a b : Usize) : Decidable (a < b) := Nat.decLt _ _
instance (a b : I32)   : Decidable (a < b) := Int.decLt _ _
instance (a b : I64)   : Decidable (a < b) := Int.decLt _ _

instance (a b : U8)    : Decidable (a ≤ b) := Nat.decLe _ _
instance (a b : U16)   : Decidable (a ≤ b) := Nat.decLe _ _
instance (a b : U32)   : Decidable (a ≤ b) := Nat.decLe _ _
instance (a b : U64)   : Decidable (a ≤ b) := Nat.decLe _ _
instance (a b : Usize) : Decidable (a ≤ b) := Nat.decLe _ _
instance (a b : I32)   : Decidable (a ≤ b) := Int.decLe _ _
instance (a b : I64)   : Decidable (a ≤ b) := Int.decLe _ _

-- ============================================================================
-- Arithmetic returning Result (overflow-checked)
-- ============================================================================

-- Unsigned
instance : HAdd U8  U8  (Result U8)  where hAdd a b := let r := a.val + b.val; if r < 256 then .ok ⟨r⟩ else .fail .panic
instance : HSub U8  U8  (Result U8)  where hSub a b := if b.val ≤ a.val then .ok ⟨a.val - b.val⟩ else .fail .panic
instance : HAdd U16 U16 (Result U16) where hAdd a b := let r := a.val + b.val; if r < 65536 then .ok ⟨r⟩ else .fail .panic
instance : HSub U16 U16 (Result U16) where hSub a b := if b.val ≤ a.val then .ok ⟨a.val - b.val⟩ else .fail .panic
instance : HMul U16 U16 (Result U16) where hMul a b := let r := a.val * b.val; if r < 65536 then .ok ⟨r⟩ else .fail .panic
instance : HDiv U16 U16 (Result U16) where hDiv a b := if b.val = 0 then .fail .panic else .ok ⟨a.val / b.val⟩
instance : HAdd U32 U32 (Result U32) where hAdd a b := let r := a.val + b.val; if r < 4294967296 then .ok ⟨r⟩ else .fail .panic
instance : HSub U32 U32 (Result U32) where hSub a b := if b.val ≤ a.val then .ok ⟨a.val - b.val⟩ else .fail .panic
instance : HMul U32 U32 (Result U32) where hMul a b := let r := a.val * b.val; if r < 4294967296 then .ok ⟨r⟩ else .fail .panic
instance : HAdd U64 U64 (Result U64) where hAdd a b := let r := a.val + b.val; if r < 18446744073709551616 then .ok ⟨r⟩ else .fail .panic
instance : HSub U64 U64 (Result U64) where hSub a b := if b.val ≤ a.val then .ok ⟨a.val - b.val⟩ else .fail .panic
instance : HAdd Usize Usize (Result Usize) where hAdd a b := let r := a.val + b.val; if r < 18446744073709551616 then .ok ⟨r⟩ else .fail .panic
instance : HSub Usize Usize (Result Usize) where hSub a b := if b.val ≤ a.val then .ok ⟨a.val - b.val⟩ else .fail .panic
instance : HMod Usize Usize (Result Usize) where hMod a b := if b.val = 0 then .fail .panic else .ok ⟨a.val % b.val⟩

-- Signed
instance : HAdd I32 I32 (Result I32) where hAdd a b := let r := a.val + b.val; if I32.min ≤ r ∧ r ≤ I32.max then .ok ⟨r⟩ else .fail .panic
instance : HSub I32 I32 (Result I32) where hSub a b := let r := a.val - b.val; if I32.min ≤ r ∧ r ≤ I32.max then .ok ⟨r⟩ else .fail .panic
instance : HMul I32 I32 (Result I32) where hMul a b := let r := a.val * b.val; if I32.min ≤ r ∧ r ≤ I32.max then .ok ⟨r⟩ else .fail .panic
instance : HAdd I64 I64 (Result I64) where hAdd a b := let r := a.val + b.val; if I64.min ≤ r ∧ r ≤ I64.max then .ok ⟨r⟩ else .fail .panic
instance : HSub I64 I64 (Result I64) where hSub a b := let r := a.val - b.val; if I64.min ≤ r ∧ r ≤ I64.max then .ok ⟨r⟩ else .fail .panic
instance : HMul I64 I64 (Result I64) where hMul a b := let r := a.val * b.val; if I64.min ≤ r ∧ r ≤ I64.max then .ok ⟨r⟩ else .fail .panic
instance : HDiv I64 I64 (Result I64) where
  hDiv a b := if b.val = 0 then .fail .panic else let r := a.val / b.val; if I64.min ≤ r ∧ r ≤ I64.max then .ok ⟨r⟩ else .fail .panic

def I64.neg (a : I64) : Result I64 :=
  let r := -a.val; if I64.min ≤ r ∧ r ≤ I64.max then .ok ⟨r⟩ else .fail .panic

instance : Neg I32 where neg a := ⟨-a.val⟩
instance : Neg I64 where neg a := ⟨-a.val⟩

-- ============================================================================
-- Conversion helpers
-- ============================================================================

def I64.ofU8 (x : U8) : Result I64 := .ok ⟨x.val⟩
def U8.toI64 (x : U8) : Result I64 := .ok ⟨x.val⟩

-- ============================================================================
-- Bitwise operations (plain, no Result — used in serialization)
-- ============================================================================

instance : HShiftLeft  U8  Nat U8  where hShiftLeft  a n := ⟨(a.val <<< n) % 256⟩
instance : HShiftRight U8  Nat U8  where hShiftRight a n := ⟨a.val >>> n⟩
instance : HShiftLeft  U32 Nat U32 where hShiftLeft  a n := ⟨(a.val <<< n) % 4294967296⟩
instance : HShiftRight U32 Nat U32 where hShiftRight a n := ⟨a.val >>> n⟩
instance : HShiftLeft  U64 Nat U64 where hShiftLeft  a n := ⟨(a.val <<< n) % 18446744073709551616⟩
instance : HShiftRight U64 Nat U64 where hShiftRight a n := ⟨a.val >>> n⟩

instance : HOr  U8  U8  U8  where hOr  a b := ⟨a.val ||| b.val⟩
instance : HOr  U32 U32 U32 where hOr  a b := ⟨a.val ||| b.val⟩
instance : HOr  U64 U64 U64 where hOr  a b := ⟨a.val ||| b.val⟩
instance : HAnd U8  U8  U8  where hAnd a b := ⟨a.val &&& b.val⟩
instance : HAnd U32 U32 U32 where hAnd a b := ⟨a.val &&& b.val⟩

-- ============================================================================
-- Vec type
-- ============================================================================

structure Vec (α : Type) where
  val : Array α
  deriving Repr

namespace Vec
instance : Inhabited (Vec α) where default := ⟨#[]⟩
def new : Vec α := ⟨#[]⟩
def push (v : Vec α) (x : α) : Result (Vec α) := .ok ⟨v.val.push x⟩
def length (v : Vec α) : Usize := ⟨v.val.size⟩
def len (v : Vec α) : Usize := ⟨v.val.size⟩
def size (v : Vec α) : Nat := v.val.size
def index (v : Vec α) (i : Usize) : Result α :=
  if h : i.val < v.val.size then .ok v.val[i.val] else .fail .panic
def index_mut_update (v : Vec α) (i : Usize) (x : α) : Result (Vec α) :=
  if h : i.val < v.val.size then .ok ⟨v.val.set (Fin.mk i.val h) x⟩ else .fail .panic
end Vec

-- ============================================================================
-- Slice type
-- ============================================================================

structure Slice (α : Type) where
  val : Array α
  deriving Repr

namespace Slice
instance : Inhabited (Slice α) where default := ⟨#[]⟩
def len (s : Slice α) : Usize := ⟨s.val.size⟩
def index (s : Slice α) (i : Usize) : Result α :=
  if h : i.val < s.val.size then .ok s.val[i.val] else .fail .panic
end Slice

-- ============================================================================
-- core.result.Result (Rust's own Result, distinct from Aeneas.Result)
-- ============================================================================

namespace core.result
inductive Result (α β : Type) where
  | ok  : α → Result α β
  | err : β → Result α β
  deriving Repr, BEq, DecidableEq
end core.result

-- ============================================================================
-- Simp lemma
-- ============================================================================

@[simp] theorem bind_ok (a : α) (f : α → Result β) :
    Result.bind (Result.ok a) f = f a := rfl

-- ============================================================================
-- Namespaces for compatibility with various `open` patterns
-- ============================================================================

namespace Std
  export Aeneas (U8 U16 U32 U64 Usize I8 I16 I32 I64 Result Vec Slice)
end Std

end Aeneas

namespace Primitives
  export Aeneas (U8 U16 U32 U64 Usize I8 I16 I32 I64 Result Vec Slice Error)
  export Aeneas.Result (ok fail bind)
end Primitives

export Aeneas (Result)
export Aeneas.Result (ok fail)

-- ============================================================================
-- Tactic stubs
-- ============================================================================

syntax "progress" : tactic
syntax "progress" "as" "⟨" ident,* "⟩" : tactic
macro_rules
  | `(tactic| progress) => `(tactic| sorry)
  | `(tactic| progress as ⟨$_,*⟩) => `(tactic| sorry)

syntax "scalar_tac" : tactic
macro_rules
  | `(tactic| scalar_tac) => `(tactic| omega)
