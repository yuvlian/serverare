use crate::manager::FirewallManager;

#[cfg(target_os = "windows")]
pub use windows::WindowsFirewallManager;

#[cfg(target_os = "linux")]
pub use linux::LinuxFirewallManager;

#[cfg(target_os = "windows")]
pub fn new_firewall_manager() -> impl FirewallManager {
    WindowsFirewallManager
}

#[cfg(target_os = "linux")]
pub fn new_firewall_manager() -> impl FirewallManager {
    LinuxFirewallManager
}

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

pub fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0 }
    }
    #[cfg(target_os = "linux")]
    {
        unsafe { libc::geteuid() == 0 }
    }
}
