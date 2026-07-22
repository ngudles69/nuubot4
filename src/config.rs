use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{NuuError, Result};

/// Hold the complete non-secret installation configuration.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub environment: EnvironmentConfig,
    pub paths: PathsConfig,
    pub btrunner: BtRunnerConfig,
    pub runtime: RuntimeConfig,
    pub simulator: SimulatorConfig,
}

/// Select one named operating environment.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentConfig {
    pub name: String,
}

/// Locate Nuubot4-owned and shared read-only inputs.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathsConfig {
    pub shared_data: PathBuf,
    pub sweep_database: PathBuf,
    pub credentials: PathBuf,
    pub logs: PathBuf,
}

/// Configure the standalone replay driver.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BtRunnerConfig {
    pub loader: LoaderKind,
    pub timer_interval_ms: u64,
    pub parquet_batch_size: usize,
    pub window_pause_ms: u64,
}

/// Select one admitted replay encoding.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoaderKind {
    Csv,
    Parquet,
}

/// Configure the temporary Runtime control assembly.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub max_cycles: u64,
    pub signaler: SignalerConfig,
    pub executors: Vec<ExecutorConfig>,
    #[serde(default)]
    pub risks: Vec<RiskConfig>,
}

/// Select and configure one Signaler.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalerConfig {
    pub kind: String,
    pub ma_kind: MaKind,
    pub fast_ma: usize,
    pub slow_ma: usize,
}

/// Select one supported moving average.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MaKind {
    Ema,
    Sma,
}

/// Select and configure one Executor.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutorConfig {
    pub kind: String,
    pub max_ticks: Option<u64>,
    pub max_bars: Option<u64>,
}

/// Select and configure one Risk implementation.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiskConfig {
    pub kind: String,
}

/// Preserve future Simulator defaults without wiring a Venue.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulatorConfig {
    pub starting_equity: f64,
    pub fees_pct: f64,
    pub slippage_pct: f64,
}

/// Load and validate the root non-secret configuration.
pub fn load_config(path: &Path) -> Result<AppConfig> {
    // Read config text.
    let text = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&text)
        .map_err(|error| NuuError::Config(format!("{}: {error}", path.display())))?;

    // Validate core settings.
    validate_config(&config)?;
    Ok(config)
}

/// Reject contradictory admitted configuration.
fn validate_config(config: &AppConfig) -> Result<()> {
    // Check required strings.
    if config.environment.name.trim().is_empty() {
        return Err(NuuError::Config("environment.name is empty".into()));
    }
    if config.paths.shared_data.as_os_str().is_empty()
        || config.paths.sweep_database.as_os_str().is_empty()
        || config.paths.credentials.as_os_str().is_empty()
        || config.paths.logs.as_os_str().is_empty()
    {
        return Err(NuuError::Config("one configured path is empty".into()));
    }

    // Check replay limits.
    if config.btrunner.timer_interval_ms == 0 || config.btrunner.parquet_batch_size == 0 {
        return Err(NuuError::Config(
            "BtRunner timer and batch size must be positive".into(),
        ));
    }

    // Check Runtime shape.
    if config.runtime.max_cycles == 0 || config.runtime.executors.is_empty() {
        return Err(NuuError::Config(
            "Runtime requires max_cycles and at least one Executor".into(),
        ));
    }
    if config.runtime.signaler.kind != "macross" {
        return Err(NuuError::Config(format!(
            "unknown Signaler: {}",
            config.runtime.signaler.kind
        )));
    }
    if config.runtime.signaler.fast_ma == 0
        || config.runtime.signaler.fast_ma >= config.runtime.signaler.slow_ma
    {
        return Err(NuuError::Config(
            "Macross fast_ma must be positive and less than slow_ma".into(),
        ));
    }

    // Check child selections.
    for executor in &config.runtime.executors {
        if executor.kind != "observer" {
            return Err(NuuError::Config(format!(
                "unknown Executor: {}",
                executor.kind
            )));
        }
        if executor.max_ticks.is_none() && executor.max_bars.is_none() {
            return Err(NuuError::Config(
                "ObserverExecutor requires max_ticks or max_bars".into(),
            ));
        }
        if executor.max_ticks == Some(0) || executor.max_bars == Some(0) {
            return Err(NuuError::Config(
                "ObserverExecutor limits must be positive".into(),
            ));
        }
    }
    for risk in &config.runtime.risks {
        if risk.kind != "balanced" {
            return Err(NuuError::Config(format!("unknown Risk: {}", risk.kind)));
        }
    }

    // Check Simulator defaults.
    if !config.simulator.starting_equity.is_finite()
        || config.simulator.starting_equity <= 0.0
        || !config.simulator.fees_pct.is_finite()
        || config.simulator.fees_pct < 0.0
        || !config.simulator.slippage_pct.is_finite()
        || config.simulator.slippage_pct < 0.0
    {
        return Err(NuuError::Config("invalid Simulator defaults".into()));
    }
    Ok(())
}
