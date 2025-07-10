use redis::{Client, Connection, RedisError};
use std::net::IpAddr;

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

    pub fn ping(&self) -> Result<String, RedisError> {
        let mut con = self.get_connection()?;
        let pong: String = redis::cmd("PING").query(&mut con)?;
        Ok(pong)
    }
} 