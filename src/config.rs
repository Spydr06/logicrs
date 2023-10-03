pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ID: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const COPYRIGHT: &str = "© 2022 - 2023 Spydr06";
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[cfg(debug_assertions)]
pub const APP_ICON_NAME: &str = "com.spydr06.logicrs.Devel";

#[cfg(not(debug_assertions))]
pub const APP_ICON_NAME: &'static str = "com.spydr06.logicrs";

pub const MAX_ACTION_STACK_SIZE: usize = 100;
