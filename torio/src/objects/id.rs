use std::sync::atomic::{AtomicU32, Ordering};


#[derive(Debug)]
pub struct GlobalId {
    id: AtomicU32,
}

static GLOBAL_ID: GlobalId = GlobalId {
    id: AtomicU32::new(2),
};

impl GlobalId {
    pub fn next() -> u32 {
        GLOBAL_ID.id.fetch_add(1, Ordering::Relaxed)
    }
}


