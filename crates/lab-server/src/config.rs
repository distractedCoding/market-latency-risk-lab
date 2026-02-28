use std::{
    env, fmt,
    net::{AddrParseError, SocketAddr},
};

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8080";

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub listen_addr: SocketAddr,
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidListenAddr(AddrParseError),
    NonUnicodeListenAddr,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidListenAddr(err) => {
                write!(f, "LAB_SERVER_ADDR is not a valid socket address: {err}")
            }
            Self::NonUnicodeListenAddr => {
                write!(f, "LAB_SERVER_ADDR contains non-unicode data")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidListenAddr(err) => Some(err),
            Self::NonUnicodeListenAddr => None,
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

        Ok(Self { listen_addr })
    }
}

#[cfg(test)]
mod tests {
    use std::{env, sync::Mutex};

    use super::{Config, ConfigError};

    static ENV_LOCK: Mutex<()> = Mutex::new(());
    const ENV_KEY: &str = "LAB_SERVER_ADDR";

    struct EnvVarGuard {
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(value: &str) -> Self {
            let previous = env::var_os(ENV_KEY);
            env::set_var(ENV_KEY, value);
            Self { previous }
        }

        fn unset() -> Self {
            let previous = env::var_os(ENV_KEY);
            env::remove_var(ENV_KEY);
            Self { previous }
        }

        #[cfg(unix)]
        fn set_os(value: std::ffi::OsString) -> Self {
            let previous = env::var_os(ENV_KEY);
            env::set_var(ENV_KEY, value);
            Self { previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => env::set_var(ENV_KEY, value),
                None => env::remove_var(ENV_KEY),
            }
        }
    }

    #[test]
    fn defaults_listen_address_when_env_is_unset() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::unset();

        let config = Config::from_env().unwrap();

        assert_eq!(config.listen_addr, "0.0.0.0:8080".parse().unwrap());
    }

    #[test]
    fn uses_listen_address_override_from_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::set("127.0.0.1:9090");

        let config = Config::from_env().unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:9090".parse().unwrap());
    }

    #[test]
    fn returns_error_for_invalid_listen_address_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::set("not-an-addr");

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::InvalidListenAddr(_)));
    }

    #[cfg(unix)]
    #[test]
    fn returns_error_for_non_unicode_env_var() {
        use std::os::unix::ffi::OsStringExt;

        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::set_os(std::ffi::OsString::from_vec(vec![0x66, 0x6f, 0x80]));

        let err = Config::from_env().unwrap_err();

        assert!(matches!(err, ConfigError::NonUnicodeListenAddr));
    }
}
