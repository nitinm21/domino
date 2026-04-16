use anyhow::Result;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn shutdown_flag() -> Result<Arc<AtomicBool>> {
    let flag = Arc::new(AtomicBool::new(false));
    flag::register(SIGTERM, Arc::clone(&flag))?;
    flag::register(SIGINT, Arc::clone(&flag))?;
    Ok(flag)
}

pub fn is_shutdown(flag: &AtomicBool) -> bool {
    flag.load(Ordering::Relaxed)
}
