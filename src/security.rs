use windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent;
use std::process::exit;

/// Checks if the application is being debugged.
/// If a debugger is detected, the application exits immediately.
pub fn anti_debug_check() {
    unsafe {
        if IsDebuggerPresent().as_bool() {
            // Exit silently to confuse the person debugging
            exit(0);
        }
    }
}
