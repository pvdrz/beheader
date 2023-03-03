mod source_map;
pub(crate) use source_map::SourceMap;

/// A region of code. The position of a span is *not* guaranteed to be relative to the start of the
/// file that includes the region. The methods inside [`SourceMap`] can be used to extract the
/// string representation of this region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) lo: usize,
    pub(crate) hi: usize,
}
