use redis::{Client, Connection, RedisError};
use std::net::IpAddr;

const TOKENS_SET: &str = "tokens_of_interest";

pub struct LocalDb {
    client: Client,
}

impl LocalDb {

    /// Creates a new `LocalDb` Redis instance
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address of the Redis server
    /// * `port` - The port number of the Redis server
    ///
    /// # Returns
    ///
    /// * `Ok(LocalDb)` if the connection is successful.
    /// * `Err(RedisError)` if there is an error connecting to Redis.
    pub fn new(ip: IpAddr, port: u16) -> Result<Self, RedisError> {
        let url = format!("redis://{}:{}/", ip, port);
        let client = Client::open(url)?;
        Ok(LocalDb { client })
    }

    fn get_connection(&self) -> Result<Connection, RedisError> {
        self.client.get_connection()
    }

    /// Reads tokens of interest from db. 
    /// If db is uninitialized it populates provided defaults.
    ///
    /// # Arguments
    ///
    /// * `defaults` - Default tokens to use if not found in db.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - Tokens of interest from the db.
    /// * `Err(RedisError)` - Any db error.
    pub fn read_tokens_or_defaults(&self, defaults: &[&str]) -> Result<Vec<String>, RedisError> {
        let mut con = self.get_connection()?;
        let tokens: Vec<String> = redis::cmd("SMEMBERS")
            .arg(TOKENS_SET)
            .query(&mut con)?;

        if tokens.is_empty() {
            println!("No tokens of interest found in db, populating with defaults");
            for token in defaults {
                redis::cmd("SADD").arg(TOKENS_SET).arg(token).execute(&mut con);
            }
            Ok(defaults.iter().map(|token| token.to_string()).collect())
        } else {
            Ok(tokens)
        }
    }
} 