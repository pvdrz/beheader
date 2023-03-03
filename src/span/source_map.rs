use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::span::Span;

/// Keeps track of all the source code being preprocessed. This not only includes files and text
/// provided by the user but also any source files included when processing `#include` directives.
#[derive(Default)]
pub(crate) struct SourceMap {
    inner: RefCell<SourceMapInner>,
}

#[derive(Default)]
struct SourceMapInner {
    buffer: Vec<u8>,
    map: HashMap<PathBuf, Span>,
}

impl SourceMap {
    /// Get the string representation of a region.
    ///
    /// As the value returned by this method is of type [`Ref`], it must be dropped before doing
    /// any write operation on the [`SourceMap`].
    pub(crate) fn get_bytes(&self, span: Span) -> Ref<'_, [u8]> {
        Ref::map(self.inner.borrow(), |inner| &inner.buffer[span.lo..span.hi])
    }

    /// Read a file, store its contents in the [`SourceMap`] and return the [`Span`] for the
    /// contents of the file.
    ///
    /// If the path of the file has already been seen by this method, the file is not read again.
    pub(crate) fn read_file<P: AsRef<Path>>(&self, path: &P) -> io::Result<Span> {
        let (mut map, mut buffer) = RefMut::map_split(self.inner.borrow_mut(), |inner| {
            (&mut inner.map, &mut inner.buffer)
        });
        match map.entry(path.as_ref().to_owned()) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(entry) => {
                let lo = buffer.len();
                let hi = lo + File::open(path)?.read_to_end(&mut buffer)?;
                let span = Span { lo, hi };
                entry.insert(span);
                Ok(span)
            }
        }
    }

    /// Store a sequence of bytes in the [`SourceMap`] and return the [`Span`] for it.
    ///
    /// The returned [`Span`] is not associated to any file path.
    pub(crate) fn store_bytes(&self, bytes: &[u8]) -> Span {
        let buffer = &mut self.inner.borrow_mut().buffer;

        let lo = buffer.len();
        buffer.extend_from_slice(bytes);
        let hi = buffer.len();

        Span { lo, hi }
    }

    /// Find the file path to which a [`Span`] belongs. Return `None` if the [`Span`] does not
    /// belong to any file.
    pub(crate) fn find_file(&self, target: Span) -> Option<PathBuf> {
        for (path, span) in &self.inner.borrow().map {
            if span.lo <= target.lo && span.hi >= target.hi {
                return Some(path.clone());
            }
        }
        None
    }
}
