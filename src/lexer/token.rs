use crate::span::Span;

/// A preprocessing token, as defined in the section 6.4 of C17.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}

/// The differen kinds of preprocessing tokens. The description for each kind can be found at the
/// section 6.4 of C17 using the identifier shown in the documentation of each variant of this
/// `enum`.
///
/// We include sequences of white-space characters and new-line characters as tokens even though
/// they are not represented as preprocessing tokens in C17. This is done because new-line
/// characters are important delimiters to parse preprocessing directives (An example of this can
/// be found in the syntax definition in 6.10) and the presence of white-space characters changes
/// the semantics of some preprocessing directives (This can be infered from section 6.10.3, as an
/// example, `#define FOO()` is a function-like macro and `#define FOO ()` is an object-like macro).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
    // A `header-name`.
    Header,
    // An `identifier`.
    Ident,
    // A `pp-number`.
    Number,
    // A `character-constant`.
    Char,
    // A `string-literal`.
    Str,
    // A `punctuator`.
    Punct,
    // Any non-white-space character that cannot be one of the above.
    Any,
    // A sequence of white-space characters possibly including comments.
    Space,
    // A single new-line character.
    Newline,
}
