//! Shared executable entrypoint.

use std::process::ExitCode;

use crate::Result;
use crate::common::logging::log_error;

pub type Arguments = Vec<String>;

/// Read raw program arguments.
pub fn arguments() -> Arguments {
    std::env::args().skip(1).collect()
}

/// Handle one final program result and return its exit code.
pub fn exit(program: &str, result: Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log_error(program, error);
            ExitCode::FAILURE
        }
    }
}
