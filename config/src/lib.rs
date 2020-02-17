#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: usize,
    pub redis_config: RedisConfig,
}

#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub port: usize,
    pub host: String,
    pub version: Option<String>,
}

impl Config {
    fn new(port: Option<usize>, host: Option<String>) -> Self {
        let port = match port {
            Some(p) => p,
            None => 6379,
        };
        let host = match host {
            Some(h) => h,
            None => "127.0.0.1".to_owned(),
        };
        Self {
            port: port,
            host: host,
            ..Self::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let redis_config = RedisConfig {
            port: 16379,
            version: None,
            host: "127.0.0.1".to_owned(),
        };
        Config {
            redis_config: redis_config,
            port: 6379,
            host: "127.0.0.1".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::new(None, None);
        assert_eq!(config.port, 6379);
        assert_eq!(config.redis_config.port, 16379);
        assert_eq!(config.redis_config.host, "127.0.0.1".to_owned());
    }
}
