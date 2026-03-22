// Tutorial 03: Infix Calculator
//
// A recursive descent parser and evaluator for infix arithmetic expressions.
// Written in Aeneas-friendly style: index-passing, no iterators, no closures.
//
// Grammar:
//   expr   = term (('+' | '-') term)*
//   term   = factor (('*' | '/') factor)*
//   factor = Num | '(' expr ')'

/// Binary operators with standard precedence.
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

/// Lexer tokens including parentheses.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Num(i64),
    Operator(Op),
    LParen,
    RParen,
}

/// Abstract syntax tree for expressions.
/// Box<Expr> becomes just Expr in Lean (Aeneas erases the indirection).
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(i64),
    BinOp(Op, Box<Expr>, Box<Expr>),
}

/// Parser error variants.
#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken,
    UnexpectedEnd,
    TrailingTokens,
}

/// Evaluator error variants.
#[derive(Clone, Debug, PartialEq)]
pub enum EvalError {
    DivisionByZero,
    Overflow,
}

// ============================================================================
// Lexer
// ============================================================================

/// Tokenize a byte slice into a list of tokens.
/// Uses explicit while loop with index — Aeneas-friendly.
#[allow(clippy::manual_is_ascii_check)]
pub fn lex(input: &[u8]) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut i: usize = 0;

    while i < input.len() {
        let ch = input[i];

        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
            // Skip whitespace
            i += 1;
        } else if ch == b'+' {
            tokens.push(Token::Operator(Op::Add));
            i += 1;
        } else if ch == b'-' {
            // Check if this is a negative number: '-' followed by a digit,
            // and either at the start or after an operator or '('
            let is_unary = (i + 1 < input.len())
                && input[i + 1] >= b'0'
                && input[i + 1] <= b'9'
                && (tokens.is_empty()
                    || matches!(
                        tokens.last(),
                        Some(Token::Operator(_)) | Some(Token::LParen)
                    ));

            if is_unary {
                // Parse as negative number
                i += 1; // skip the '-'
                let mut value: i64 = 0;
                while i < input.len() && input[i] >= b'0' && input[i] <= b'9' {
                    value = value * 10 + (input[i] - b'0') as i64;
                    i += 1;
                }
                tokens.push(Token::Num(-value));
            } else {
                tokens.push(Token::Operator(Op::Sub));
                i += 1;
            }
        } else if ch == b'*' {
            tokens.push(Token::Operator(Op::Mul));
            i += 1;
        } else if ch == b'/' {
            tokens.push(Token::Operator(Op::Div));
            i += 1;
        } else if ch == b'(' {
            tokens.push(Token::LParen);
            i += 1;
        } else if ch == b')' {
            tokens.push(Token::RParen);
            i += 1;
        } else if (b'0'..=b'9').contains(&ch) {
            // Parse multi-digit number
            let mut value: i64 = 0;
            while i < input.len() && input[i] >= b'0' && input[i] <= b'9' {
                value = value * 10 + (input[i] - b'0') as i64;
                i += 1;
            }
            tokens.push(Token::Num(value));
        } else {
            return Err(ParseError::UnexpectedToken);
        }
    }

    Ok(tokens)
}

// ============================================================================
// Parser — recursive descent, index-passing style
// ============================================================================

/// Parse an additive expression: term (('+' | '-') term)*
/// Returns (AST, next position).
pub fn parse_expr(tokens: &[Token], pos: usize) -> Result<(Expr, usize), ParseError> {
    let (mut left, mut pos) = parse_term(tokens, pos)?;

    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Operator(Op::Add) => {
                let (right, next) = parse_term(tokens, pos + 1)?;
                left = Expr::BinOp(Op::Add, Box::new(left), Box::new(right));
                pos = next;
            }
            Token::Operator(Op::Sub) => {
                let (right, next) = parse_term(tokens, pos + 1)?;
                left = Expr::BinOp(Op::Sub, Box::new(left), Box::new(right));
                pos = next;
            }
            _ => break,
        }
    }

    Ok((left, pos))
}

/// Parse a multiplicative expression: factor (('*' | '/') factor)*
/// Returns (AST, next position).
pub fn parse_term(tokens: &[Token], pos: usize) -> Result<(Expr, usize), ParseError> {
    let (mut left, mut pos) = parse_factor(tokens, pos)?;

    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Operator(Op::Mul) => {
                let (right, next) = parse_factor(tokens, pos + 1)?;
                left = Expr::BinOp(Op::Mul, Box::new(left), Box::new(right));
                pos = next;
            }
            Token::Operator(Op::Div) => {
                let (right, next) = parse_factor(tokens, pos + 1)?;
                left = Expr::BinOp(Op::Div, Box::new(left), Box::new(right));
                pos = next;
            }
            _ => break,
        }
    }

    Ok((left, pos))
}

/// Parse an atomic expression: a number or a parenthesized sub-expression.
pub fn parse_factor(tokens: &[Token], pos: usize) -> Result<(Expr, usize), ParseError> {
    if pos >= tokens.len() {
        return Err(ParseError::UnexpectedEnd);
    }

    match &tokens[pos] {
        Token::Num(n) => Ok((Expr::Num(*n), pos + 1)),
        Token::LParen => {
            let (expr, next) = parse_expr(tokens, pos + 1)?;
            if next >= tokens.len() {
                return Err(ParseError::UnexpectedEnd);
            }
            match &tokens[next] {
                Token::RParen => Ok((expr, next + 1)),
                _ => Err(ParseError::UnexpectedToken),
            }
        }
        _ => Err(ParseError::UnexpectedToken),
    }
}

/// Entry point for parsing. Checks that all tokens are consumed.
pub fn parse(tokens: &[Token]) -> Result<Expr, ParseError> {
    let (expr, pos) = parse_expr(tokens, 0)?;
    if pos != tokens.len() {
        return Err(ParseError::TrailingTokens);
    }
    Ok(expr)
}

// ============================================================================
// Evaluator
// ============================================================================

/// Evaluate an expression tree. Fails on division by zero or overflow.
pub fn eval(expr: &Expr) -> Result<i64, EvalError> {
    match expr {
        Expr::Num(n) => Ok(*n),
        Expr::BinOp(op, left, right) => {
            let l = eval(left)?;
            let r = eval(right)?;
            match op {
                Op::Add => l.checked_add(r).ok_or(EvalError::Overflow),
                Op::Sub => l.checked_sub(r).ok_or(EvalError::Overflow),
                Op::Mul => l.checked_mul(r).ok_or(EvalError::Overflow),
                Op::Div => {
                    if r == 0 {
                        Err(EvalError::DivisionByZero)
                    } else {
                        l.checked_div(r).ok_or(EvalError::Overflow)
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex() {
        let tokens = lex(b"12 + 34").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Num(12), Token::Operator(Op::Add), Token::Num(34),]
        );
    }

    #[test]
    fn test_lex_parens() {
        let tokens = lex(b"(1+2)*3").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Num(1),
                Token::Operator(Op::Add),
                Token::Num(2),
                Token::RParen,
                Token::Operator(Op::Mul),
                Token::Num(3),
            ]
        );
    }

    #[test]
    fn test_parse_simple() {
        let tokens = lex(b"2 + 3").unwrap();
        let expr = parse(&tokens).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(Op::Add, Box::new(Expr::Num(2)), Box::new(Expr::Num(3)))
        );
    }

    #[test]
    fn test_parse_precedence() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        let tokens = lex(b"2 + 3 * 4").unwrap();
        let expr = parse(&tokens).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Op::Add,
                Box::new(Expr::Num(2)),
                Box::new(Expr::BinOp(
                    Op::Mul,
                    Box::new(Expr::Num(3)),
                    Box::new(Expr::Num(4))
                ))
            )
        );
    }

    #[test]
    fn test_parse_parens() {
        // (2 + 3) * 4 should override precedence
        let tokens = lex(b"(2 + 3) * 4").unwrap();
        let expr = parse(&tokens).unwrap();
        assert_eq!(
            expr,
            Expr::BinOp(
                Op::Mul,
                Box::new(Expr::BinOp(
                    Op::Add,
                    Box::new(Expr::Num(2)),
                    Box::new(Expr::Num(3))
                )),
                Box::new(Expr::Num(4))
            )
        );
    }

    #[test]
    fn test_eval() {
        let tokens = lex(b"2 + 3").unwrap();
        let expr = parse(&tokens).unwrap();
        assert_eq!(eval(&expr), Ok(5));
    }

    #[test]
    fn test_div_zero() {
        let tokens = lex(b"10 / 0").unwrap();
        let expr = parse(&tokens).unwrap();
        assert_eq!(eval(&expr), Err(EvalError::DivisionByZero));
    }

    #[test]
    fn test_complex_expression() {
        // (2 + 3) * 4 - 10 / 2 = 5 * 4 - 5 = 20 - 5 = 15
        let tokens = lex(b"(2 + 3) * 4 - 10 / 2").unwrap();
        let expr = parse(&tokens).unwrap();
        assert_eq!(eval(&expr), Ok(15));
    }

    #[test]
    fn test_nested_parens() {
        // ((1 + 2)) = 3
        let tokens = lex(b"((1 + 2))").unwrap();
        let expr = parse(&tokens).unwrap();
        assert_eq!(eval(&expr), Ok(3));
    }

    #[test]
    fn test_single_number() {
        let tokens = lex(b"42").unwrap();
        let expr = parse(&tokens).unwrap();
        assert_eq!(expr, Expr::Num(42));
        assert_eq!(eval(&expr), Ok(42));
    }

    #[test]
    fn test_trailing_tokens() {
        let tokens = lex(b"1 2").unwrap();
        assert_eq!(parse(&tokens), Err(ParseError::TrailingTokens));
    }

    #[test]
    fn test_empty_input() {
        let tokens = lex(b"").unwrap();
        assert_eq!(parse(&tokens), Err(ParseError::UnexpectedEnd));
    }
}
