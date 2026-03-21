// Core system information modules for zfetch.

pub mod init;
pub mod kernel;
pub mod os;
pub mod os_age;
pub mod uptime;
pub mod platform;

pub use init::init;
pub use kernel::kernel;
pub use os::os;
pub use os_age::os_age;
pub use uptime::uptime;
pub use platform::platform;