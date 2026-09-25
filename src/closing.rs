use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

static CLOSING: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));

const FAREWELL_GRACE: Duration = Duration::from_millis(4_500);

pub fn listen() {
    LazyLock::force(&CLOSING);
    platform::listen();
}

pub fn requested() -> bool {
    CLOSING.load(Ordering::SeqCst)
}

#[cfg(windows)]
mod platform {
    use std::sync::atomic::Ordering;
    use std::thread;

    use windows_sys::Win32::System::Console::{
        CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT, SetConsoleCtrlHandler,
    };
    use windows_sys::core::BOOL;

    const HANDLED: BOOL = 1;
    const PASSED_ON: BOOL = 0;
    const ADD_HANDLER: BOOL = 1;

    unsafe extern "system" fn on_console_event(event: u32) -> BOOL {
        if !matches!(
            event,
            CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT
        ) {
            return PASSED_ON;
        }
        super::CLOSING.store(true, Ordering::SeqCst);
        thread::sleep(super::FAREWELL_GRACE);
        HANDLED
    }

    pub fn listen() {
        unsafe {
            SetConsoleCtrlHandler(Some(on_console_event), ADD_HANDLER);
        }
    }
}

#[cfg(unix)]
mod platform {
    use std::process;
    use std::sync::atomic::Ordering;
    use std::thread;

    use signal_hook::consts::{SIGHUP, SIGTERM};
    use signal_hook::iterator::Signals;

    const FAREWELL_OVERDUE: i32 = 1;

    pub fn listen() {
        let Ok(mut signals) = Signals::new([SIGHUP, SIGTERM]) else {
            return;
        };
        thread::spawn(move || {
            if signals.forever().next().is_none() {
                return;
            }
            super::CLOSING.store(true, Ordering::SeqCst);
            thread::sleep(super::FAREWELL_GRACE);
            process::exit(FAREWELL_OVERDUE);
        });
    }
}

#[cfg(not(any(windows, unix)))]
mod platform {
    pub fn listen() {}
}
