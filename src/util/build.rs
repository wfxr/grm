//! Build information.
//!
//! Most of this information is generated by Cargo and the build script for this
//! crate.

use constcat::concat;

macro_rules! env_or_default {
    ($key:expr, $default:expr $(,)?) => {{
        match option_env!($key) {
            Some(v) => v,
            None => $default,
        }
    }};
    ($key:expr $(,)?) => {
        env_or_default!($key, "")
    };
}

/// This is the name defined in the Cargo manifest.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// This is the version defined in the Cargo manifest.
pub const CRATE_RELEASE: &str = env!("CARGO_PKG_VERSION");

/// The version including any available Git information.
#[allow(clippy::const_is_empty)]
pub const CRATE_VERSION: &str = {
    const GIT_COMMIT_DATE: &str = env_or_default!("GIT_COMMIT_DATE");
    const GIT_COMMIT_HASH: &str = env_or_default!("GIT_COMMIT_HASH");
    const GIT_COMMIT_SHORT_HASH: &str = env_or_default!("GIT_COMMIT_SHORT_HASH");
    match (
        GIT_COMMIT_DATE.is_empty(),
        GIT_COMMIT_HASH.is_empty(),
        GIT_COMMIT_SHORT_HASH.is_empty(),
    ) {
        (true, true, true) => CRATE_RELEASE,
        (false, false, false) => {
            concat!(CRATE_RELEASE, " (", GIT_COMMIT_SHORT_HASH, " ", GIT_COMMIT_DATE, ")",)
        }
        _ => panic!("unexpected git information"),
    }
};

/// The version with extra Git and Rustc information if available.
pub const CRATE_LONG_VERSION: &str = concat!(CRATE_VERSION, "\n", env!("RUSTC_VERSION_SUMMARY"));

/// The very verbose version.
pub const CRATE_VERBOSE_VERSION: &str = concat!(
    CRATE_VERSION,
    "\n\nDetails:",
    "\n  binary: ",
    CRATE_NAME,
    "\n  release: ",
    CRATE_RELEASE,
    "\n  commit-hash: ",
    env_or_default!("GIT_COMMIT_HASH", "unknown"),
    "\n  commit-date: ",
    env_or_default!("GIT_COMMIT_DATE", "unknown"),
    "\n  target: ",
    env!("TARGET"),
    "\n\nCompiled with:",
    "\n  binary: ",
    env!("RUSTC_VERSION_BINARY"),
    "\n  release: ",
    env!("RUSTC_VERSION_RELEASE"),
    "\n  commit-hash: ",
    env!("RUSTC_VERSION_COMMIT_HASH"),
    "\n  commit-date: ",
    env!("RUSTC_VERSION_COMMIT_DATE"),
    "\n  host: ",
    env!("RUSTC_VERSION_HOST"),
);
