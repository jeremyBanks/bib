use std::vec;

use super::ResponseRecord;
use super::Storage;

#[derive(Debug, Clone)]
struct WebStorage {
    pub base_url: String,
}

impl Storage for WebStorage {
}
