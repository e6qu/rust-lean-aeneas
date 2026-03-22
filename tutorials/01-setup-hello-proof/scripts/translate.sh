#!/bin/bash
# translate.sh — Run the Charon + Aeneas pipeline for Tutorial 01
#
# This script automates the two-step translation process:
# 1. Charon extracts Rust MIR into LLBC format
# 2. Aeneas translates LLBC into pure Lean 4 code
#
# Prerequisites:
#   - Charon installed (via Nix or from source)
#   - Aeneas installed (via Nix or from source)
#   - Rust toolchain (rustup)
#
# Usage:
#   cd tutorials/01-setup-hello-proof
#   ./scripts/translate.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TUTORIAL_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$TUTORIAL_DIR/rust"
LEAN_DIR="$TUTORIAL_DIR/lean"

echo "=== Tutorial 01: Translating Rust to Lean ==="
echo ""

# Step 1: Extract MIR with Charon
echo "Step 1: Running Charon to extract Rust MIR..."
echo "  Command: charon cargo --preset=aeneas"
echo "  Working directory: $RUST_DIR"
echo ""

cd "$RUST_DIR"

# If using Nix:
# nix run github:aeneasverif/aeneas#charon -L -- cargo --preset=aeneas
# If installed locally:
charon cargo --preset=aeneas

LLBC_FILE=$(find "$RUST_DIR" -name "*.llbc" -type f | head -1)

if [ -z "$LLBC_FILE" ]; then
    echo "ERROR: No .llbc file produced. Check Charon output above."
    exit 1
fi

echo "  Produced: $LLBC_FILE"
echo ""

# Step 2: Translate LLBC to Lean with Aeneas
echo "Step 2: Running Aeneas to translate to Lean..."
echo "  Command: aeneas -backend lean $LLBC_FILE"
echo ""

# If using Nix:
# nix run github:aeneasverif/aeneas -L -- -backend lean "$LLBC_FILE"
# If installed locally:
aeneas -backend lean "$LLBC_FILE"

echo ""
echo "=== Translation complete! ==="
echo ""
echo "Generated Lean files are in: $LEAN_DIR/"
echo "You can now:"
echo "  1. Inspect the generated code in lean/HelloProof/Types.lean and lean/HelloProof/Funs.lean"
echo "  2. Write proofs in lean/HelloProof/Proofs.lean"
echo "  3. Build and check proofs: cd lean/ && lake build"
