use std::{
    env, fmt,
    net::{AddrParseError, SocketAddr},
};

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_REPLAY_OUTPUT_PATH: &str = "artifacts/replay.csv";

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub replay_output_path: String,
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidListenAddr(AddrParseError),
    InvalidReplayOutputPath,
    NonUnicodeListenAddr,
    NonUnicodeReplayOutput,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidListenAddr(err) => {
                write!(f, "LAB_SERVER_ADDR is not a valid socket address: {err}")
            }
            Self::InvalidReplayOutputPath => {
                write!(f, "LAB_SERVER_REPLAY_OUTPUT must not be empty or whitespace")
            }
            Self::NonUnicodeListenAddr => {
                write!(f, "LAB_SERVER_ADDR contains non-unicode data")
            }
            Self::NonUnicodeReplayOutput => {
                write!(f, "LAB_SERVER_REPLAY_OUTPUT contains non-unicode data")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidListenAddr(err) => Some(err),
            Self::InvalidReplayOutputPath => None,
            Self::NonUnicodeListenAddr => None,
            Self::NonUnicodeReplayOutput => None,
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

        Ok(Self {
            listen_addr,
            replay_output_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{env, sync::Mutex};

    use super::{Config, ConfigError};

    static ENV_LOCK: Mutex<()> = Mutex::new(());
    const ENV_ADDR_KEY: &str = "LAB_SERVER_ADDR";
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

    #[test]
    fn defaults_listen_address_when_env_is_unset() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::unset(ENV_ADDR_KEY);

        let config = Config::from_env().unwrap();

        assert_eq!(config.listen_addr, "0.0.0.0:8080".parse().unwrap());
    }

    #[test]
    fn uses_listen_address_override_from_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::set(ENV_ADDR_KEY, "127.0.0.1:9090");

        let config = Config::from_env().unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:9090".parse().unwrap());
    }

    #[test]
    fn returns_error_for_invalid_listen_address_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::set(ENV_ADDR_KEY, "not-an-addr");

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::InvalidListenAddr(_)));
    }

    #[cfg(unix)]
    #[test]
    fn returns_error_for_non_unicode_env_var() {
        use std::os::unix::ffi::OsStringExt;

        let _lock = ENV_LOCK.lock().unwrap();
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
        let _guard = EnvVarGuard::unset(ENV_REPLAY_KEY);

        let config = Config::from_env().unwrap();

        assert_eq!(config.replay_output_path, "artifacts/replay.csv");
    }

    #[test]
    fn uses_replay_output_path_override_from_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::set(ENV_REPLAY_KEY, "artifacts/custom.csv");

        let config = Config::from_env().unwrap();

        assert_eq!(config.replay_output_path, "artifacts/custom.csv");
    }

    #[cfg(unix)]
    #[test]
    fn returns_error_for_non_unicode_replay_output_env_var() {
        use std::os::unix::ffi::OsStringExt;

        let _lock = ENV_LOCK.lock().unwrap();
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
        let _guard = EnvVarGuard::set(ENV_REPLAY_KEY, "");

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::InvalidReplayOutputPath));
    }

    #[test]
    fn returns_error_for_whitespace_replay_output_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::set(ENV_REPLAY_KEY, "   ");

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::InvalidReplayOutputPath));
    }
}
