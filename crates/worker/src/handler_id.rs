use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

/// Id of responses handler.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
pub struct HandlerId(usize);

impl HandlerId {
    pub(crate) const fn new(id: usize) -> Self {
        HandlerId(id)
    }
    pub(crate) fn raw_id(self) -> usize {
        self.0
    }

    pub fn new_inc() -> Self {
        static CTR: AtomicUsize = AtomicUsize::new(0);

        let id = CTR.fetch_add(1, Ordering::SeqCst);

        HandlerId::new(id)
    }
}
