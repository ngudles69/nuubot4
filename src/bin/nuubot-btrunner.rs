use std::process::ExitCode;

use nuubot4::btrunner::{BtRunSummary, BtRunner};
use nuubot4::common::logging::BotIdentity;
use nuubot4::setup::failure_logger;
use nuubot4::{NuuError, Result};

/// Run one identity-only BtRunner process.
fn main() -> ExitCode {
    // parse arguments
    let identity = match parse_identity() {
        Ok(identity) => identity,
        Err(error) => return fail(None, error),
    };

    // run and log results
    let result = run(identity.0, identity.1);
    match result {
        Ok(summary) => {
            print_summary(&summary);
            ExitCode::SUCCESS
        }
        Err(error) => fail(Some(identity), error),
    }
}

/// Parse identity and preserve the first lifecycle failure.
fn run(sweep_id: u64, bot_id: u64) -> Result<BtRunSummary> {
    // runner init
    let mut runner = BtRunner::init(sweep_id, bot_id)?;

    // runner start and run
    let result = runner.start().and_then(|_| runner.run());

    // stop; preserve first failure
    let stop_result = runner.stop();
    match result {
        Err(error) => Err(error),
        Ok(summary) => {
            stop_result?;
            Ok(summary)
        }
    }
}

/// Preserve fatal evidence outside lifecycle ownership.
fn fail(identity: Option<(u64, u64)>, error: NuuError) -> ExitCode {
    // handle fail
    let identity = identity.map(|(sweep_id, bot_id)| BotIdentity { sweep_id, bot_id });
    let log = failure_logger(identity);
    if let Err(log_error) = log.and_then(|log| log.error("program", &error.to_string())) {
        eprintln!("failure logging failed: {log_error}");
    }
    eprintln!("FAIL: {error}");
    ExitCode::FAILURE
}

/// Accept exactly one positive Sweep and Bot identity.
fn parse_identity() -> Result<(u64, u64)> {
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

/// Print one machine-readable operator result.
fn print_summary(summary: &BtRunSummary) {
    println!(
        "PASS loader={} ticks={} callbacks={} first_ts_ms={} last_ts_ms={} completed_cycles={} stop_reason={} elapsed_seconds={:.3}",
        summary.loader,
        summary.ticks,
        summary.callbacks,
        summary.first_ts_ms,
        summary.last_ts_ms,
        summary.completed_cycles,
        summary.stop_reason.unwrap_or("replay_end"),
        summary.elapsed_seconds
    );
}
