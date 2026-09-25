use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

static CLOSING: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));

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
    use std::time::Duration;

    use windows_sys::Win32::System::Console::{
        CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT, SetConsoleCtrlHandler,
    };
    use windows_sys::core::BOOL;

    const HANDLED: BOOL = 1;
    const PASSED_ON: BOOL = 0;
    const ADD_HANDLER: BOOL = 1;
    const FAREWELL_GRACE: Duration = Duration::from_millis(4_500);

    unsafe extern "system" fn on_console_event(event: u32) -> BOOL {
        if !matches!(
            event,
            CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT
        ) {
            return PASSED_ON;
        }
        super::CLOSING.store(true, Ordering::SeqCst);
        thread::sleep(FAREWELL_GRACE);
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
    use std::sync::Arc;

    use signal_hook::consts::{SIGHUP, SIGTERM};

    pub fn listen() {
        for signal in [SIGHUP, SIGTERM] {
            let _ = signal_hook::flag::register(signal, Arc::clone(&super::CLOSING));
        }
    }
}

#[cfg(not(any(windows, unix)))]
mod platform {
    pub fn listen() {}
}
