use redis::{Commands, Client};

fn main() {
    // Connect to Redis server at localhost:6379
    let client = Client::open("redis://127.0.0.1/").expect("Invalid Redis URL");
    let mut con = client.get_connection().expect("Failed to connect to Redis");

    // Simple PING command to test connection
    let pong: String = redis::cmd("PING").query(&mut con).expect("Failed to execute PING");
    println!("Redis PING response: {}", pong);
}
