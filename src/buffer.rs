use std::{borrow::Borrow, ops::Deref};

use crate::lexer::Token;

/// A buffer of [`Token`]s.
#[derive(Default)]
pub(crate) struct TokenBuffer {
    rest: Vec<Token>,
}

impl TokenBuffer {
    /// Push a [`Token`] into the buffer.
    pub(crate) fn push(&mut self, token: Token) {
        self.rest.push(token)
    }
}

impl Deref for TokenBuffer {
    type Target = TokenSlice;

    fn deref(&self) -> &Self::Target {
        let ptr = self.rest.as_slice() as *const [Token] as *const TokenSlice;
        // SAFETY: This pointer is valid because `TokenSlice` and `Token` have the same layout.
        unsafe { &*ptr }
    }
}

impl Borrow<TokenSlice> for TokenBuffer {
    fn borrow(&self) -> &TokenSlice {
        self
    }
}

/// A slice of [`Token`]s.
#[repr(transparent)]
pub(crate) struct TokenSlice {
    rest: [Token],
}

impl ToOwned for TokenSlice {
    type Owned = TokenBuffer;

    fn to_owned(&self) -> Self::Owned {
        TokenBuffer {
            rest: self.rest.to_owned(),
        }
    }
}
