use std::process::ExitCode;

use nuubot4::btrunner::{BtRunSummary, BtRunner};
use nuubot4::common::logging::BotIdentity;
use nuubot4::setup::program_logger;
use nuubot4::{NuuError, Result};

/// Run one identity-only BtRunner process.
fn main() -> ExitCode {
    // parse arguments
    let identity = match parser() {
        Ok(identity) => identity,
        Err(error) => return log_failure(None, error),
    };

    // run and log results
    let result = run(identity.0, identity.1);
    match result {
        Ok(summary) => log_success(identity, &summary),
        Err(error) => log_failure(Some(identity), error),
    }
}

/// Parse program arguments.
fn parser() -> Result<(u64, u64)> {
    // validate sweep_id and bot_id
    let mut args = std::env::args().skip(1);
    let sweep_id = args
        .next()
        .ok_or_else(|| NuuError::Config("usage: nuubot-btrunner <sweep_id> <bot_id>".into()))?
        .parse::<u64>()
        .map_err(|error| NuuError::Config(format!("invalid sweep_id: {error}")))?;
    let bot_id = args
        .next()
        .ok_or_else(|| NuuError::Config("usage: nuubot-btrunner <sweep_id> <bot_id>".into()))?
        .parse::<u64>()
        .map_err(|error| NuuError::Config(format!("invalid bot_id: {error}")))?;
    if sweep_id == 0 || bot_id == 0 || args.next().is_some() {
        return Err(NuuError::Config(
            "usage: nuubot-btrunner <sweep_id> <bot_id>".into(),
        ));
    }
    Ok((sweep_id, bot_id))
}

/// Run the lifecycle and preserve the first failure.
fn run(sweep_id: u64, bot_id: u64) -> Result<BtRunSummary> {
    // init
    let mut runner = BtRunner::init(sweep_id, bot_id)?;

    // start and run
    let result = runner.start().and_then(|_| runner.run());

    // stop and log; preserve first failure
    let stop_result = runner.stop();
    match result {
        Err(error) => Err(error),
        Ok(summary) => {
            stop_result?;
            Ok(summary)
        }
    }
}

/// Log one successful program outcome.
fn log_success(identity: (u64, u64), summary: &BtRunSummary) -> ExitCode {
    let identity = BotIdentity {
        sweep_id: identity.0,
        bot_id: identity.1,
    };
    let result = program_logger(Some(identity)).and_then(|log| {
        log.info(
            "program",
            &format!(
                "PASS loader={} ticks={} callbacks={} first_ts_ms={} last_ts_ms={} completed_cycles={} stop_reason={} elapsed_seconds={:.3}",
                summary.loader,
                summary.ticks,
                summary.callbacks,
                summary.first_ts_ms,
                summary.last_ts_ms,
                summary.completed_cycles,
                summary.stop_reason.unwrap_or("replay_end"),
                summary.elapsed_seconds
            ),
        )
    });
    if let Err(error) = result {
        eprintln!("success logging failed: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Log one failed program outcome.
fn log_failure(identity: Option<(u64, u64)>, error: NuuError) -> ExitCode {
    let identity = identity.map(|(sweep_id, bot_id)| BotIdentity { sweep_id, bot_id });
    if let Err(log_error) =
        program_logger(identity).and_then(|log| log.error("program", &format!("FAIL: {error}")))
    {
        eprintln!("failure logging failed: {log_error}; FAIL: {error}");
    }
    ExitCode::FAILURE
}
