use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Returns the next unique identifier for a WebDriver BiDi command.
pub fn get_next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
