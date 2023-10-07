use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use serde::{Deserialize, Serialize};

// Id: Struct that already stores a pre-hashed Uuid value to make HashTable lookups faster

#[derive(
    Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Id(u64);

impl Id {
    #[inline]
    pub fn new() -> Self {
        #[allow(deprecated)]
        let mut hasher = DefaultHasher::default();
        hasher.write(&uuid::Uuid::new_v4().into_bytes());
        Self(hasher.finish())
    }

    #[inline]
    pub fn empty() -> Self {
        Self(0)
    }
}
