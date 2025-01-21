/// A set of gRPC custom metadata entries.
#[derive(Clone, Debug, Default)]
pub struct MetadataMap {
    headers: http::HeaderMap,
}

// ===== impl MetadataMap =====
impl MetadataMap {
    /// Create an empty `MetadataMap`.
    ///
    /// The map will be created without any capacity. This function will not
    /// allocate.
    pub fn new() -> Self {
        MetadataMap::with_capacity(0)
    }

    /// Create an empty `MetadataMap` with the specified capacity.
    ///
    /// The returned map will allocate internal storage in order to hold about
    /// `capacity` elements without reallocating. However, this is a "best
    /// effort" as there are usage patterns that could cause additional
    /// allocations before `capacity` metadata entries are stored in the map.
    ///
    /// More capacity than requested may be allocated.
    pub fn with_capacity(capacity: usize) -> MetadataMap {
        MetadataMap {
            headers: http::HeaderMap::with_capacity(capacity),
        }
    }
}
