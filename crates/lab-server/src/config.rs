use std::{
    env, fmt,
    net::{AddrParseError, SocketAddr},
};

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_MODE: RunMode = RunMode::PaperLive;
const DEFAULT_REPLAY_OUTPUT_PATH: &str = "artifacts/replay.csv";
const DEFAULT_EXECUTION_MODE: ExecutionMode = ExecutionMode::Paper;
const DEFAULT_LIVE_FEATURE_ENABLED: bool = false;
const DEFAULT_LAG_THRESHOLD_PCT: f64 = 0.3;
const DEFAULT_PER_TRADE_RISK_PCT: f64 = 0.5;
const DEFAULT_DAILY_LOSS_CAP_PCT: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    PaperLive,
    Sim,
}

impl RunMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "paper-live" => Some(Self::PaperLive),
            "sim" => Some(Self::Sim),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PaperLive => "paper-live",
            Self::Sim => "sim",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Paper,
    Live,
}

impl ExecutionMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "paper" => Some(Self::Paper),
            "live" => Some(Self::Live),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Paper => "paper",
            Self::Live => "live",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub mode: RunMode,
    pub replay_output_path: String,
    pub execution_mode: ExecutionMode,
    pub live_feature_enabled: bool,
    pub lag_threshold_pct: f64,
    pub per_trade_risk_pct: f64,
    pub daily_loss_cap_pct: f64,
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidListenAddr(AddrParseError),
    InvalidMode,
    InvalidReplayOutputPath,
    InvalidExecutionMode,
    InvalidLiveFeatureEnabled,
    InvalidLagThresholdPct,
    InvalidPerTradeRiskPct,
    InvalidDailyLossCapPct,
    NonUnicodeListenAddr,
    NonUnicodeMode,
    NonUnicodeReplayOutput,
    NonUnicodeExecutionMode,
    NonUnicodeLiveFeatureEnabled,
    NonUnicodeLagThresholdPct,
    NonUnicodePerTradeRiskPct,
    NonUnicodeDailyLossCapPct,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidListenAddr(err) => {
                write!(f, "LAB_SERVER_ADDR is not a valid socket address: {err}")
            }
            Self::InvalidMode => {
                write!(f, "LAB_SERVER_MODE must be one of: paper-live, sim")
            }
            Self::InvalidReplayOutputPath => {
                write!(
                    f,
                    "LAB_SERVER_REPLAY_OUTPUT must not be empty or whitespace"
                )
            }
            Self::InvalidExecutionMode => {
                write!(f, "LAB_EXECUTION_MODE must be one of: paper, live")
            }
            Self::InvalidLiveFeatureEnabled => {
                write!(f, "LAB_LIVE_FEATURE_ENABLED must be true or false")
            }
            Self::InvalidLagThresholdPct => {
                write!(
                    f,
                    "LAB_LAG_THRESHOLD_PCT must be a finite percentage between 0 and 100"
                )
            }
            Self::InvalidPerTradeRiskPct => {
                write!(
                    f,
                    "LAB_RISK_PER_TRADE_PCT must be a finite percentage between 0 and 100"
                )
            }
            Self::InvalidDailyLossCapPct => {
                write!(
                    f,
                    "LAB_DAILY_LOSS_CAP_PCT must be a finite percentage between 0 and 100"
                )
            }
            Self::NonUnicodeListenAddr => {
                write!(f, "LAB_SERVER_ADDR contains non-unicode data")
            }
            Self::NonUnicodeMode => {
                write!(f, "LAB_SERVER_MODE contains non-unicode data")
            }
            Self::NonUnicodeReplayOutput => {
                write!(f, "LAB_SERVER_REPLAY_OUTPUT contains non-unicode data")
            }
            Self::NonUnicodeExecutionMode => {
                write!(f, "LAB_EXECUTION_MODE contains non-unicode data")
            }
            Self::NonUnicodeLiveFeatureEnabled => {
                write!(f, "LAB_LIVE_FEATURE_ENABLED contains non-unicode data")
            }
            Self::NonUnicodeLagThresholdPct => {
                write!(f, "LAB_LAG_THRESHOLD_PCT contains non-unicode data")
            }
            Self::NonUnicodePerTradeRiskPct => {
                write!(f, "LAB_RISK_PER_TRADE_PCT contains non-unicode data")
            }
            Self::NonUnicodeDailyLossCapPct => {
                write!(f, "LAB_DAILY_LOSS_CAP_PCT contains non-unicode data")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidListenAddr(err) => Some(err),
            Self::InvalidMode => None,
            Self::InvalidReplayOutputPath => None,
            Self::InvalidExecutionMode => None,
            Self::InvalidLiveFeatureEnabled => None,
            Self::InvalidLagThresholdPct => None,
            Self::InvalidPerTradeRiskPct => None,
            Self::InvalidDailyLossCapPct => None,
            Self::NonUnicodeListenAddr => None,
            Self::NonUnicodeMode => None,
            Self::NonUnicodeReplayOutput => None,
            Self::NonUnicodeExecutionMode => None,
            Self::NonUnicodeLiveFeatureEnabled => None,
            Self::NonUnicodeLagThresholdPct => None,
            Self::NonUnicodePerTradeRiskPct => None,
            Self::NonUnicodeDailyLossCapPct => None,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let listen_addr = match env::var("LAB_SERVER_ADDR") {
            Ok(value) => value.parse().map_err(ConfigError::InvalidListenAddr)?,
            Err(env::VarError::NotPresent) => DEFAULT_LISTEN_ADDR
                .parse()
                .expect("default listen address must be valid"),
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::NonUnicodeListenAddr);
            }
        };

        let mode = match env::var("LAB_SERVER_MODE") {
            Ok(value) => RunMode::parse(value.as_str()).ok_or(ConfigError::InvalidMode)?,
            Err(env::VarError::NotPresent) => DEFAULT_MODE,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::NonUnicodeMode);
            }
        };

        let replay_output_path = match env::var("LAB_SERVER_REPLAY_OUTPUT") {
            Ok(value) => {
                if value.trim().is_empty() {
                    return Err(ConfigError::InvalidReplayOutputPath);
                }
                value
            }
            Err(env::VarError::NotPresent) => DEFAULT_REPLAY_OUTPUT_PATH.to_owned(),
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::NonUnicodeReplayOutput);
            }
        };

        let execution_mode = match env::var("LAB_EXECUTION_MODE") {
            Ok(value) => {
                ExecutionMode::parse(value.as_str()).ok_or(ConfigError::InvalidExecutionMode)?
            }
            Err(env::VarError::NotPresent) => DEFAULT_EXECUTION_MODE,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::NonUnicodeExecutionMode);
            }
        };

        let live_feature_enabled = match env::var("LAB_LIVE_FEATURE_ENABLED") {
            Ok(value) => {
                parse_bool(value.as_str()).ok_or(ConfigError::InvalidLiveFeatureEnabled)?
            }
            Err(env::VarError::NotPresent) => DEFAULT_LIVE_FEATURE_ENABLED,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::NonUnicodeLiveFeatureEnabled);
            }
        };

        let lag_threshold_pct = parse_percentage_env(
            "LAB_LAG_THRESHOLD_PCT",
            DEFAULT_LAG_THRESHOLD_PCT,
            ConfigError::InvalidLagThresholdPct,
            ConfigError::NonUnicodeLagThresholdPct,
        )?;

        let per_trade_risk_pct = parse_percentage_env(
            "LAB_RISK_PER_TRADE_PCT",
            DEFAULT_PER_TRADE_RISK_PCT,
            ConfigError::InvalidPerTradeRiskPct,
            ConfigError::NonUnicodePerTradeRiskPct,
        )?;

        let daily_loss_cap_pct = parse_percentage_env(
            "LAB_DAILY_LOSS_CAP_PCT",
            DEFAULT_DAILY_LOSS_CAP_PCT,
            ConfigError::InvalidDailyLossCapPct,
            ConfigError::NonUnicodeDailyLossCapPct,
        )?;

        Ok(Self {
            listen_addr,
            mode,
            replay_output_path,
            execution_mode,
            live_feature_enabled,
            lag_threshold_pct,
            per_trade_risk_pct,
            daily_loss_cap_pct,
        })
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_percentage_env(
    key: &str,
    default_value: f64,
    invalid_error: ConfigError,
    non_unicode_error: ConfigError,
) -> Result<f64, ConfigError> {
    match env::var(key) {
        Ok(value) => {
            let parsed = match value.parse::<f64>() {
                Ok(parsed) => parsed,
                Err(_) => return Err(invalid_error),
            };
            if !parsed.is_finite() || parsed <= 0.0 || parsed > 100.0 {
                return Err(invalid_error);
            }
            Ok(parsed)
        }
        Err(env::VarError::NotPresent) => Ok(default_value),
        Err(env::VarError::NotUnicode(_)) => Err(non_unicode_error),
    }
}

#[cfg(test)]
mod tests {
    use std::{env, sync::Mutex};

    use super::{Config, ConfigError, ExecutionMode, RunMode};

    static ENV_LOCK: Mutex<()> = Mutex::new(());
    const ENV_ADDR_KEY: &str = "LAB_SERVER_ADDR";
    const ENV_MODE_KEY: &str = "LAB_SERVER_MODE";
    const ENV_REPLAY_KEY: &str = "LAB_SERVER_REPLAY_OUTPUT";

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var_os(key);
            env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = env::var_os(key);
            env::remove_var(key);
            Self { key, previous }
        }

        #[cfg(unix)]
        fn set_os(key: &'static str, value: std::ffi::OsString) -> Self {
            let previous = env::var_os(key);
            env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    fn reset_config_env_baseline() -> [EnvVarGuard; 3] {
        [
            EnvVarGuard::unset(ENV_ADDR_KEY),
            EnvVarGuard::unset(ENV_MODE_KEY),
            EnvVarGuard::unset(ENV_REPLAY_KEY),
        ]
    }

    #[test]
    fn defaults_listen_address_when_env_is_unset() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();

        let config = Config::from_env().unwrap();

        assert_eq!(config.listen_addr, "0.0.0.0:8080".parse().unwrap());
    }

    #[test]
    fn defaults_listen_address_ignores_unrelated_mode_and_replay_env_vars() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _ambient_mode_guard = EnvVarGuard::set(ENV_MODE_KEY, "invalid");
        let _ambient_replay_guard = EnvVarGuard::set(ENV_REPLAY_KEY, "  ");
        let _baseline = reset_config_env_baseline();

        let config = Config::from_env().unwrap();

        assert_eq!(config.listen_addr, "0.0.0.0:8080".parse().unwrap());
    }

    #[test]
    fn uses_listen_address_override_from_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set(ENV_ADDR_KEY, "127.0.0.1:9090");

        let config = Config::from_env().unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:9090".parse().unwrap());
    }

    #[test]
    fn returns_error_for_invalid_listen_address_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set(ENV_ADDR_KEY, "not-an-addr");

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::InvalidListenAddr(_)));
    }

    #[test]
    fn defaults_to_paper_live_mode() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();

        let cfg = Config::from_env().unwrap();

        assert_eq!(cfg.mode, RunMode::PaperLive);
    }

    #[test]
    fn defaults_execution_mode_to_paper() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();

        let cfg = Config::from_env().unwrap();

        assert_eq!(cfg.execution_mode, ExecutionMode::Paper);
    }

    #[test]
    fn defaults_lag_threshold_and_risk_caps() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();

        let cfg = Config::from_env().unwrap();

        assert_eq!(cfg.lag_threshold_pct, 0.3);
        assert_eq!(cfg.per_trade_risk_pct, 0.5);
        assert_eq!(cfg.daily_loss_cap_pct, 2.0);
    }

    #[test]
    fn uses_mode_override_from_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set(ENV_MODE_KEY, "sim");

        let cfg = Config::from_env().unwrap();

        assert_eq!(cfg.mode, RunMode::Sim);
    }

    #[test]
    fn returns_error_for_invalid_mode_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set(ENV_MODE_KEY, "invalid");

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::InvalidMode));
    }

    #[cfg(unix)]
    #[test]
    fn returns_error_for_non_unicode_mode_env_var() {
        use std::os::unix::ffi::OsStringExt;

        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set_os(
            ENV_MODE_KEY,
            std::ffi::OsString::from_vec(vec![0x66, 0x6f, 0x80]),
        );

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::NonUnicodeMode));
    }

    #[cfg(unix)]
    #[test]
    fn returns_error_for_non_unicode_env_var() {
        use std::os::unix::ffi::OsStringExt;

        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set_os(
            ENV_ADDR_KEY,
            std::ffi::OsString::from_vec(vec![0x66, 0x6f, 0x80]),
        );

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::NonUnicodeListenAddr));
    }

    #[test]
    fn defaults_replay_output_path_when_env_is_unset() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();

        let config = Config::from_env().unwrap();

        assert_eq!(config.replay_output_path, "artifacts/replay.csv");
    }

    #[test]
    fn uses_replay_output_path_override_from_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set(ENV_REPLAY_KEY, "artifacts/custom.csv");

        let config = Config::from_env().unwrap();

        assert_eq!(config.replay_output_path, "artifacts/custom.csv");
    }

    #[cfg(unix)]
    #[test]
    fn returns_error_for_non_unicode_replay_output_env_var() {
        use std::os::unix::ffi::OsStringExt;

        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set_os(
            ENV_REPLAY_KEY,
            std::ffi::OsString::from_vec(vec![0x66, 0x6f, 0x80]),
        );

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::NonUnicodeReplayOutput));
    }

    #[test]
    fn returns_error_for_empty_replay_output_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set(ENV_REPLAY_KEY, "");

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::InvalidReplayOutputPath));
    }

    #[test]
    fn returns_error_for_whitespace_replay_output_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _baseline = reset_config_env_baseline();
        let _guard = EnvVarGuard::set(ENV_REPLAY_KEY, "   ");

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::InvalidReplayOutputPath));
    }
}
