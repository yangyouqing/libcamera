use core_interfaces::Timer;
use std::time::Instant;
use std::sync::Once;

static mut BOOT_INSTANT: Option<Instant> = None;
static INIT: Once = Once::new();

fn boot_instant() -> Instant {
    INIT.call_once(|| {
        // SAFETY: written once behind Once
        unsafe { BOOT_INSTANT = Some(Instant::now()); }
    });
    // SAFETY: guaranteed initialized after call_once
    unsafe { BOOT_INSTANT.unwrap() }
}

pub struct LinuxTimer;

impl LinuxTimer {
    pub fn new() -> Self {
        let _ = boot_instant(); // ensure init
        Self
    }
}

impl Timer for LinuxTimer {
    fn monotonic_ms(&self) -> u64 {
        boot_instant().elapsed().as_millis() as u64
    }

    fn sleep_ms(&self, ms: u32) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
}
