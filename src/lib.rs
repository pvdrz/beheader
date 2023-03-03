//! A preprocessing library for the C programming language.
//!
//! This library was written trying to follow the ISO/IEC 9899:2018 standard, also known as C17.
//! Because of this, the documentation contains references to specific senctions of this document
//! whose most recent free draft can be found
//! [here](https://web.archive.org/web/20181230041359if_/http://www.open-std.org/jtc1/sc22/wg14/www/abq/c17_updated_proposed_fdis.pdf).

mod buffer;
mod lexer;
mod span;

use span::SourceMap;

pub fn preprocess(source: &[u8]) {
    let map = SourceMap::default();
    map.tokenize_bytes(source);
}
