use std::process::ExitCode;

use nuubot4::Result;
use nuubot4::btrunner::BtRunner;
use nuubot4::common::logging::{bot_log_name, logger};
use nuubot4::common::program::{Arguments, arguments, exit};

const PROGRAM: &str = "nuubot-btrunner";

#[derive(Clone, Copy)]
struct BotIdentity {
    sweep_id: u64,
    bot_id: u64,
}

// Program flow

/// Run one identity-only BtRunner process.
fn main() -> ExitCode {
    let arguments = arguments();
    exit(PROGRAM, program(arguments))
}

/// Parse identity and run one BtRunner.
fn program(arguments: Arguments) -> Result<()> {
    let identity = parser(&arguments)?;
    run(identity)
}

/// Parse program arguments.
fn parser(arguments: &[String]) -> Result<BotIdentity> {
    if arguments.len() != 2 {
        return Err(format!(
            "invalid arguments {arguments:?}: usage: nuubot-btrunner <sweep_id> <bot_id>"
        ));
    }

    Ok(BotIdentity {
        sweep_id: parse_id(&arguments[0])?,
        bot_id: parse_id(&arguments[1])?,
    })
}

/// Run one BtRunner lifecycle.
fn run(identity: BotIdentity) -> Result<()> {
    let log = logger(Some(&bot_log_name(identity.sweep_id, identity.bot_id)))?;
    let mut runner = BtRunner::init(log, identity.sweep_id, identity.bot_id)?;

    runner.start()?;
    let run_result = runner.run();
    let stop_result = runner.stop();
    run_result.and(stop_result)
}

// Helpers

/// Parse one positive numeric ID.
fn parse_id(value: &str) -> Result<u64> {
    let id = value
        .parse::<u64>()
        .map_err(|_| format!("invalid ID: {value}"))?;

    if id == 0 {
        return Err("ID must be greater than zero".into());
    }

    Ok(id)
}
