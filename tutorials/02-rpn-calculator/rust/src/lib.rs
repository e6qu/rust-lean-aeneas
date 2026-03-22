// Tutorial 02: RPN Calculator
//
// A pure, Aeneas-friendly Reverse Polish Notation calculator.
// All data structures are functional (no Vec, no mutation) so that
// Aeneas produces clean inductive types and recursive functions in Lean.

/// RPN token: either a numeric literal or a binary operator.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Num(i64),
    Plus,
    Minus,
    Mul,
    Div,
}

/// Errors that can occur during RPN evaluation.
#[derive(Clone, Debug, PartialEq)]
pub enum EvalError {
    DivisionByZero,
    StackUnderflow,
    TooManyValues,
    InvalidToken,
}

/// Functional linked-list stack.
///
/// We use an algebraic data type instead of Vec so that Aeneas translates
/// it to a Lean inductive type with clean structural recursion.
#[derive(Clone, Debug, PartialEq)]
pub enum Stack {
    Empty,
    Push(i64, Box<Stack>),
}

impl Stack {
    /// Create an empty stack.
    pub fn new() -> Self {
        Stack::Empty
    }

    /// Push a value onto the stack, returning the new stack.
    pub fn push(self, val: i64) -> Stack {
        Stack::Push(val, Box::new(self))
    }

    /// Pop the top value off the stack.
    /// Returns the value and the remaining stack, or StackUnderflow if empty.
    pub fn pop(self) -> Result<(i64, Stack), EvalError> {
        match self {
            Stack::Push(val, rest) => Ok((val, *rest)),
            Stack::Empty => Err(EvalError::StackUnderflow),
        }
    }

    /// Peek at the top value without consuming the stack.
    pub fn peek(&self) -> Result<i64, EvalError> {
        match self {
            Stack::Push(val, _) => Ok(*val),
            Stack::Empty => Err(EvalError::StackUnderflow),
        }
    }

    /// Return the number of elements on the stack.
    pub fn len(&self) -> usize {
        match self {
            Stack::Empty => 0,
            Stack::Push(_, rest) => 1 + rest.len(),
        }
    }

    /// Check whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Stack::Empty)
    }
}

/// Tokenize a single whitespace-delimited word (given as a byte slice).
///
/// Single-character operators (+, -, *, /) are recognized directly.
/// Everything else is parsed as a non-negative integer.
pub fn tokenize_word(word: &[u8]) -> Result<Token, EvalError> {
    if word.len() == 1 {
        match word[0] {
            b'+' => return Ok(Token::Plus),
            b'-' => return Ok(Token::Minus),
            b'*' => return Ok(Token::Mul),
            b'/' => return Ok(Token::Div),
            c if c >= b'0' && c <= b'9' => return Ok(Token::Num((c - b'0') as i64)),
            _ => return Err(EvalError::InvalidToken),
        }
    }
    parse_number(word)
}

/// Parse a byte slice as a non-negative decimal integer.
///
/// Uses a simple accumulator loop that Aeneas can translate to a
/// recursive function with a loop fixpoint.
pub fn parse_number(bytes: &[u8]) -> Result<Token, EvalError> {
    if bytes.is_empty() {
        return Err(EvalError::InvalidToken);
    }
    let mut acc: i64 = 0;
    let mut i: usize = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c < b'0' || c > b'9' {
            return Err(EvalError::InvalidToken);
        }
        acc = acc * 10 + (c - b'0') as i64;
        i += 1;
    }
    Ok(Token::Num(acc))
}

/// Apply a binary operator to two operands.
///
/// Separated from eval_step so that we avoid unreachable!() in the
/// operator dispatch. Aeneas translates this to a clean pattern match.
fn apply_binop(op: &Token, a: i64, b: i64) -> Result<i64, EvalError> {
    match op {
        Token::Plus => Ok(a + b),
        Token::Minus => Ok(a - b),
        Token::Mul => Ok(a * b),
        Token::Div => {
            if b == 0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(a / b)
            }
        }
        Token::Num(_) => Err(EvalError::InvalidToken),
    }
}

/// Evaluate one token against the current stack.
///
/// - If the token is a number, push it.
/// - If the token is an operator, pop two operands, apply, and push the result.
pub fn eval_step(stack: Stack, token: &Token) -> Result<Stack, EvalError> {
    match token {
        Token::Num(n) => Ok(stack.push(*n)),
        op => {
            let (b, stack) = stack.pop()?;
            let (a, stack) = stack.pop()?;
            let result = apply_binop(op, a, b)?;
            Ok(stack.push(result))
        }
    }
}

/// Evaluate a complete sequence of RPN tokens.
///
/// Folds eval_step over the token slice. After processing all tokens,
/// the stack must contain exactly one value (the result).
pub fn evaluate(tokens: &[Token]) -> Result<i64, EvalError> {
    let mut stack = Stack::new();
    let mut i: usize = 0;
    while i < tokens.len() {
        stack = eval_step(stack, &tokens[i])?;
        i += 1;
    }
    match stack {
        Stack::Push(val, rest) if rest.is_empty() => Ok(val),
        Stack::Empty => Err(EvalError::StackUnderflow),
        _ => Err(EvalError::TooManyValues),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Stack operations --

    #[test]
    fn test_stack_operations() {
        let s = Stack::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);

        let s = s.push(10);
        assert!(!s.is_empty());
        assert_eq!(s.len(), 1);
        assert_eq!(s.peek(), Ok(10));

        let s = s.push(20);
        assert_eq!(s.len(), 2);
        assert_eq!(s.peek(), Ok(20));

        let (val, s) = s.pop().unwrap();
        assert_eq!(val, 20);
        assert_eq!(s.len(), 1);

        let (val, s) = s.pop().unwrap();
        assert_eq!(val, 10);
        assert!(s.is_empty());

        assert_eq!(s.pop(), Err(EvalError::StackUnderflow));
    }

    // -- Tokenizer --

    #[test]
    fn test_tokenize_operators() {
        assert_eq!(tokenize_word(b"+"), Ok(Token::Plus));
        assert_eq!(tokenize_word(b"-"), Ok(Token::Minus));
        assert_eq!(tokenize_word(b"*"), Ok(Token::Mul));
        assert_eq!(tokenize_word(b"/"), Ok(Token::Div));
    }

    #[test]
    fn test_tokenize_single_digit() {
        assert_eq!(tokenize_word(b"0"), Ok(Token::Num(0)));
        assert_eq!(tokenize_word(b"5"), Ok(Token::Num(5)));
        assert_eq!(tokenize_word(b"9"), Ok(Token::Num(9)));
    }

    #[test]
    fn test_tokenize_multi_digit() {
        assert_eq!(tokenize_word(b"42"), Ok(Token::Num(42)));
        assert_eq!(tokenize_word(b"100"), Ok(Token::Num(100)));
        assert_eq!(tokenize_word(b"999"), Ok(Token::Num(999)));
    }

    #[test]
    fn test_tokenize_invalid() {
        assert_eq!(tokenize_word(b"abc"), Err(EvalError::InvalidToken));
        assert_eq!(tokenize_word(b""), Err(EvalError::InvalidToken));
        assert_eq!(tokenize_word(b"!"), Err(EvalError::InvalidToken));
    }

    // -- Evaluate: simple expressions --

    #[test]
    fn test_evaluate_simple() {
        // 3 4 + = 7
        let tokens = vec![Token::Num(3), Token::Num(4), Token::Plus];
        assert_eq!(evaluate(&tokens), Ok(7));

        // 10 3 - = 7
        let tokens = vec![Token::Num(10), Token::Num(3), Token::Minus];
        assert_eq!(evaluate(&tokens), Ok(7));

        // 6 7 * = 42
        let tokens = vec![Token::Num(6), Token::Num(7), Token::Mul];
        assert_eq!(evaluate(&tokens), Ok(42));

        // 20 4 / = 5
        let tokens = vec![Token::Num(20), Token::Num(4), Token::Div];
        assert_eq!(evaluate(&tokens), Ok(5));
    }

    // -- Evaluate: complex expressions --

    #[test]
    fn test_evaluate_complex() {
        // 3 4 + 2 * = (3 + 4) * 2 = 14
        let tokens = vec![
            Token::Num(3), Token::Num(4), Token::Plus,
            Token::Num(2), Token::Mul,
        ];
        assert_eq!(evaluate(&tokens), Ok(14));

        // 5 1 2 + 4 * + 3 - = 5 + ((1 + 2) * 4) - 3 = 14
        let tokens = vec![
            Token::Num(5), Token::Num(1), Token::Num(2), Token::Plus,
            Token::Num(4), Token::Mul, Token::Plus, Token::Num(3), Token::Minus,
        ];
        assert_eq!(evaluate(&tokens), Ok(14));

        // Single number
        let tokens = vec![Token::Num(42)];
        assert_eq!(evaluate(&tokens), Ok(42));
    }

    // -- Error cases --

    #[test]
    fn test_division_by_zero() {
        let tokens = vec![Token::Num(10), Token::Num(0), Token::Div];
        assert_eq!(evaluate(&tokens), Err(EvalError::DivisionByZero));
    }

    #[test]
    fn test_stack_underflow() {
        // Operator with no operands
        let tokens = vec![Token::Plus];
        assert_eq!(evaluate(&tokens), Err(EvalError::StackUnderflow));

        // Operator with only one operand
        let tokens = vec![Token::Num(5), Token::Plus];
        assert_eq!(evaluate(&tokens), Err(EvalError::StackUnderflow));

        // Empty expression
        let tokens: Vec<Token> = vec![];
        assert_eq!(evaluate(&tokens), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_too_many_values() {
        // Two numbers, no operator
        let tokens = vec![Token::Num(1), Token::Num(2)];
        assert_eq!(evaluate(&tokens), Err(EvalError::TooManyValues));

        // Three numbers, one operator (leaves two values)
        let tokens = vec![Token::Num(1), Token::Num(2), Token::Num(3), Token::Plus];
        assert_eq!(evaluate(&tokens), Err(EvalError::TooManyValues));
    }
}
