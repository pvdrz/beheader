//! All functions related to lexing.
//!
//! This code is heavily inspired by the
//! [`proc_macro2::parse`](https://github.com/dtolnay/proc-macro2/blob/a3fbb7de911db5964dcec00b009ec4a4d5868af5/src/parse.rs)
//! module.
mod token;

#[cfg(test)]
mod tests;

use std::path::Path;

pub(crate) use token::{Token, TokenKind};

use crate::{
    buffer::TokenBuffer,
    span::{SourceMap, Span},
};

impl SourceMap {
    /// Read a file and tokenize it.
    pub(crate) fn tokenize_file<P: AsRef<Path>>(&self, path: &P) -> std::io::Result<TokenBuffer> {
        let span = self.read_file(path)?;
        Ok(self.tokenize_region(span))
    }

    /// Read a sequence of bytes and tokenize it.
    pub(crate) fn tokenize_bytes(&self, source: &[u8]) -> TokenBuffer {
        let span = self.store_bytes(source);
        self.tokenize_region(span)
    }

    /// Tokenize a region.
    ///
    /// Panic if the region contains invalid tokens.
    fn tokenize_region(&self, span: Span) -> TokenBuffer {
        let rest = &*self.get_bytes(span);

        let mut lexer = Lexer {
            rest,
            offset: span.lo,
        };

        let mut buffer = TokenBuffer::default();

        while !lexer.is_empty() {
            match lexer.next_token() {
                Ok((rest, token)) => {
                    buffer.push(token);
                    lexer = rest;
                }
                Err(Reject) => {
                    let span = lexer.get_span(lexer.len());
                    let rest = &*self.get_bytes(span);
                    let rest_short = String::from_utf8_lossy(rest.get(..80).unwrap_or(rest));

                    if let Some(path) = self.find_file(span) {
                        panic!(
                            "Invalid token at {}:{} \"{}\"",
                            path.display(),
                            lexer.offset,
                            rest_short
                        );
                    } else {
                        panic!("Invalid token in input \"{}\"", rest_short);
                    }
                }
            }
        }

        buffer
    }
}

type Result<'a, T> = std::result::Result<(Lexer<'a>, T), Reject>;
#[cfg_attr(test, derive(Debug))]
struct Reject;

macro_rules! must_match {
    ($($tokens:tt)*) => {
        if !matches!($($tokens)*) {
            return Err(Reject);
        }
    };
}

#[derive(Clone, Copy)]
struct Lexer<'a> {
    /// The remaining region to be tokenized.
    rest: &'a [u8],
    /// The start of `rest`, relative to the start of the region being tokenized.
    offset: usize,
}

impl<'a> Lexer<'a> {
    fn next_token(self) -> Result<'a, Token> {
        let (rest, token) = if let Ok((rest, header)) = header(self) {
            (rest, header)
        } else if let Ok((rest, ident)) = ident(self) {
            (rest, ident)
        } else if let Ok((rest, number)) = number(self) {
            (rest, number)
        } else {
            return Err(Reject);
        };

        Ok((rest, token))
    }

    /// Move this lexer to the desired index.
    ///
    /// Panic if the index is out of bounds.
    fn advance(self, index: usize) -> Self {
        let (head, rest) = self.rest.split_at(index);
        Self {
            offset: self.offset + head.len(),
            rest,
        }
    }

    /// Return a new span that starts at the current offset and has `len` length.
    fn get_span(&self, len: usize) -> Span {
        Span {
            lo: self.offset,
            hi: self.offset + len,
        }
    }

    /// Get the length of the remaining text region.
    fn len(&self) -> usize {
        self.rest.len()
    }

    /// Return an iterator over the remaining bytes.
    fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.rest.iter().copied()
    }

    /// Return an iterator over the remaining bytes and their positions.
    fn byte_indices(&self) -> impl Iterator<Item = (usize, u8)> + '_ {
        self.bytes().enumerate()
    }

    /// Check if the remaining text starts with `tag` and consume it if it does.
    fn parse_bytes(self, tag: &[u8]) -> std::result::Result<Self, Reject> {
        if self.rest.starts_with(tag) {
            Ok(self.advance(tag.len()))
        } else {
            Err(Reject)
        }
    }

    /// Check if the next remaining byte matches `pattern` and consume it if it does.
    fn parse_byte(self, pattern: impl BytePattern) -> std::result::Result<Self, Reject> {
        if self
            .rest
            .first()
            .map(|byte| pattern.matches(*byte))
            .unwrap_or_default()
        {
            Ok(self.advance(1))
        } else {
            Err(Reject)
        }
    }

    fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }
}

trait BytePattern {
    fn matches(self, byte: u8) -> bool;
}

impl BytePattern for u8 {
    fn matches(self, byte: u8) -> bool {
        byte == self
    }
}

impl<F: Fn(u8) -> bool> BytePattern for F {
    fn matches(self, byte: u8) -> bool {
        (self)(byte)
    }
}

/// Produce a `header-name` as defined in section 6.4.7 of C17.
fn header(input: Lexer<'_>) -> Result<'_, Token> {
    if let Ok(rest) = h_header(input) {
        Ok(rest)
    } else {
        q_header(input)
    }
}

/// Produce an `<h-char-sequence>` as defined in section 6.4.7 of C17.
fn h_header(input: Lexer<'_>) -> Result<'_, Token> {
    // It has to start with a `<`.
    let rest = input.parse_byte(b'<')?;

    let mut bytes = rest.bytes().enumerate().peekable();

    // Now we try to parse a `q-char-sequence`.
    while let Some((i, byte)) = bytes.next() {
        match byte {
            // new-line characters are not valid `h-char`s
            // FIXME: what about `\r`?
            b'\n' => {}
            // if we find `’`, `\`, `"` ,`//`, or `/*`, the behavior is undefined. We will
            // reject.
            b'\'' | b'\\' | b'"' => {}
            b'/' if matches!(bytes.peek(), Some(&(_, b'/' | b'*'))) => {}
            // if we find `>` then we are done
            b'>' => {
                let len = i + 2;
                return Ok((
                    input.advance(len),
                    Token {
                        kind: TokenKind::Header,
                        span: input.get_span(len),
                    },
                ));
            }
            // any other character is a valid `h-char`
            _ => continue,
        }
        break;
    }

    Err(Reject)
}

/// Produce a `"q-char-sequence"` as defined in section 6.4.7 of C17.
fn q_header(input: Lexer<'_>) -> Result<'_, Token> {
    // It has to start with a `"`.
    let rest = input.parse_byte(b'"')?;

    let mut bytes = rest.bytes().enumerate().peekable();

    // Now we try to parse a `q-char-sequence`.
    while let Some((i, byte)) = bytes.next() {
        match byte {
            // new-line characters are not valid `q-char`s
            // FIXME: what about `\r`?
            b'\n' => {}
            // if we find `’`, `\`, `//`, or `/*`, the behavior is undefined. We will
            // reject.
            b'\'' | b'\\' => {}
            b'/' if matches!(bytes.peek(), Some(&(_, b'/' | b'*'))) => {}
            // if we find `"` then we are done
            b'"' => {
                let len = i + 2;
                return Ok((
                    input.advance(len),
                    Token {
                        kind: TokenKind::Header,
                        span: input.get_span(len),
                    },
                ));
            }
            // any other character is a valid `q-char`
            _ => continue,
        }
        break;
    }

    Err(Reject)
}

/// Produce an `identifier` as defined in section 6.4.2 of C17.
fn ident(input: Lexer<'_>) -> Result<'_, Token> {
    let mut chars = input.byte_indices();
    // The first char of an `identifier` must be an `identifier-nondigit`.
    must_match!(chars.next(), Some((_, c)) if is_ident_nondigit(c));

    // This is the length of the `identifier`.
    let mut len = input.len();
    for (i, ch) in chars {
        // A valid `identifier` can be followed by either an `identifier-nondigit` or a `digit`.
        // Otherwise, this character does not belong to the `identifier` and its position is the
        // same as the length of the `identifier`.
        if !(is_ident_nondigit(ch) || ch.is_ascii_digit()) {
            len = i;
            break;
        }
    }

    Ok((
        input.advance(len),
        Token {
            kind: TokenKind::Ident,
            span: input.get_span(len),
        },
    ))
}

/// Check if `byte` is an `identifier-nondigit` as defined in section 6.4.2 of C17.
fn is_ident_nondigit(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

/// Produce a `pp-number` as defined in section 6.4.8 of C17.
fn number(input: Lexer<'_>) -> Result<'_, Token> {
    // A `pp-number` optionally starts with `.`
    let (rest, prefix_len) = input
        .parse_byte(b'.')
        .map(|rest| (rest, 1))
        .unwrap_or((input, 0));

    let mut bytes = rest.byte_indices().peekable();
    // The next character must be a `digit`.
    must_match!(bytes.next(), Some((_, c)) if c.is_ascii_digit());

    // This is the length of the `pp-number`.
    let mut len = input.len();

    while let Some((i, byte)) = bytes.next() {
        // A valid `pp-number` can be followed by a `.`, a `digit`, an `identifier-nondigit`, or it
        // can also be followed by `e`, `E`, `p` or `P` immediately followed by a `sign`.
        match byte {
            // We do exponents first because the exponents are `identifier-nondigit`s.
            b'e' | b'E' | b'p' | b'P' if matches!(bytes.peek(), Some((_, b'+' | b'-'))) => {
                bytes.next().unwrap();
                continue;
            }
            byte if byte == b'.' || byte.is_ascii_digit() || is_ident_nondigit(byte) => {
                continue;
            }
            _ => {}
        }
        // Otherwise, this character does not belong to the `number` and its position is the same
        // as the length of the `number`.
        len = i + prefix_len;
        break;
    }

    Ok((
        input.advance(len),
        Token {
            kind: TokenKind::Number,
            span: input.get_span(len),
        },
    ))
}
