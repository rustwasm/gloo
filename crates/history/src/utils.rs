use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) fn get_id() -> u32 {
    static ID_CTR: AtomicU32 = AtomicU32::new(0);

    ID_CTR.fetch_add(1, Ordering::SeqCst)
}
