use crate::{buffer::TokenBuffer, lexer::TokenKind, span::Span};

use super::{Lexer, Reject, Token};

fn single_token(
    bytes: &[u8],
    f: impl Fn(Lexer<'_>) -> super::Result<'_, Token>,
) -> super::Result<'_, Token> {
    f(Lexer {
        rest: bytes,
        offset: 0,
    })
}

#[track_caller]
fn tokenize_one(bytes: &[u8], kind: TokenKind, f: impl Fn(Lexer<'_>) -> super::Result<'_, Token>) {
    let (rest, token) = single_token(bytes, f).unwrap();
    let expected_token = Token {
        kind,
        span: Span {
            lo: 0,
            hi: bytes.len(),
        },
    };
    println!("Parsed token was: {:?}", token);
    assert!(rest.is_empty(), "Remainder: {:?}", String::from_utf8_lossy(rest.rest));
    assert_eq!(expected_token, token);
}

#[test]
fn ident_alphabetic() {
    tokenize_one(b"hello", TokenKind::Ident, super::ident);
}

#[test]
#[should_panic]
fn ident_empty() {
    tokenize_one(b"", TokenKind::Ident, super::ident);
}

#[test]
#[should_panic]
fn ident_starts_with_digit() {
    tokenize_one(b"12345seven", TokenKind::Ident, super::ident);
}

#[test]
fn ident_alphanumeric() {
    tokenize_one(b"e1m1", TokenKind::Ident, super::ident);
}

#[test]
fn ident_surrounded_by_underscore() {
    tokenize_one(b"_foo_", TokenKind::Ident, super::ident);
}

#[test]
fn ident_snake_case() {
    tokenize_one(b"sneaky_snake", TokenKind::Ident, super::ident);
}

#[test]
fn ident_camel_case() {
    tokenize_one(b"CamellyCamel", TokenKind::Ident, super::ident);
}

#[test]
fn ident_mixed_case() {
    tokenize_one(b"sneaky_Camel", TokenKind::Ident, super::ident);
}

#[test]
fn number_digits() {
    tokenize_one(b"42", TokenKind::Number, super::number);
}

#[test]
fn number_begins_with_dot() {
    tokenize_one(b".42", TokenKind::Number, super::number);
}

#[test]
fn number_ends_with_dot() {
    tokenize_one(b"42.", TokenKind::Number, super::number);
}

#[test]
fn number_surrounded_by_dots() {
    tokenize_one(b".42.", TokenKind::Number, super::number);
}

#[test]
fn number_with_exponent() {
    tokenize_one(b"42e+", TokenKind::Number, super::number);
}

#[test]
fn number_with_ident() {
    tokenize_one(b"42HELLO_10", TokenKind::Number, super::number);
}

#[test]
fn number_no_sign() {
    tokenize_one(b".1e", TokenKind::Number, super::number);
}

#[test]
#[should_panic]
fn number_with_sign_no_exponent() {
    tokenize_one(b"1+", TokenKind::Number, super::number);
}

#[test]
#[should_panic]
fn number_empty() {
    tokenize_one(b"", TokenKind::Number, super::number);
}

#[test]
#[should_panic]
fn number_ident_nondigit() {
    tokenize_one(b"e", TokenKind::Number, super::number);
}
