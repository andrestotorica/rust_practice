
use std::net::IpAddr;
use std::str::FromStr;

const ERR_REDIS_DB_IP: &str = "REDIS_DB_IP is missing or invalid";
const ERR_REDIS_DB_PORT: &str = "REDIS_DB_PORT is missing or invalid";

pub struct EnvConfig {
    pub ip: IpAddr,
    pub port: u16,
}

pub fn load_from_env<F>(env_var_fn: F) -> EnvConfig
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    let ip = env_var_fn("REDIS_DB_IP").ok()
        .and_then(|s| IpAddr::from_str(&s).ok())
        .expect(ERR_REDIS_DB_IP);
    let port = env_var_fn("REDIS_DB_PORT").ok()
        .and_then(|s| s.parse::<u16>().ok())
        .expect(ERR_REDIS_DB_PORT);
    EnvConfig { ip, port }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::VarError;

    struct TestEnvVars {
        ip: Option<String>,
        port: Option<String>,
    }

    impl TestEnvVars {
        fn good() -> Self {
            Self {
                ip: Some("127.0.0.1".to_string()),
                port: Some("6379".to_string()),
            }
        }
        fn as_env_var_fn(&self) -> impl Fn(&str) -> Result<String, VarError> + '_ {
            |key| match key {
                "REDIS_DB_IP" => self.ip.clone().ok_or(VarError::NotPresent),
                "REDIS_DB_PORT" => self.port.clone().ok_or(VarError::NotPresent),
                _ => Err(VarError::NotPresent),
            }
        }
    }

    #[test]
    fn test_load_from_env_success() {
        let env = TestEnvVars::good();
        let config = load_from_env(env.as_env_var_fn());
        assert_eq!(config.ip, IpAddr::from_str("127.0.0.1").unwrap());
        assert_eq!(config.port, 6379u16);
    }

    #[test]
    #[should_panic(expected = "REDIS_DB_IP is missing or invalid")]
    fn test_load_from_env_missing_ip() {
        let mut env = TestEnvVars::good();
        env.ip = None;
        let _ = load_from_env(env.as_env_var_fn());
    }

    #[test]
    #[should_panic(expected = "REDIS_DB_PORT is missing or invalid")]
    fn test_load_from_env_missing_port() {
        let mut env = TestEnvVars::good();
        env.port = None;
        let _ = load_from_env(env.as_env_var_fn());
    }

    #[test]
    #[should_panic(expected = "REDIS_DB_IP is missing or invalid")]
    fn test_load_from_env_invalid_ip() {
        let mut env = TestEnvVars::good();
        env.ip = Some("not_an_ip".to_string());
        let _ = load_from_env(env.as_env_var_fn());
    }

    #[test]
    #[should_panic(expected = "REDIS_DB_PORT is missing or invalid")]
    fn test_load_from_env_invalid_port() {
        let mut env = TestEnvVars::good();
        env.port = Some("not_a_port".to_string());
        let _ = load_from_env(env.as_env_var_fn());
    }
}
