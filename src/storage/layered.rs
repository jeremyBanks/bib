use std::sync::Arc;

use super::Storage;

/// Storage backed by a configuration of two or more different [`Storage`]
/// implementations.
#[derive(Debug, Clone)]
pub struct LayeredStorage<Inner: Storage, Next: Storage> {
    inner: Arc<Inner>,
    next: Arc<Next>,
}

// Configurable!
// What are the options?

// For later! Just use sqlite for now.
