mod env;
mod local_db;
use local_db::LocalDb;

fn main() {
    let env_config = env::load_from_env(|key| std::env::var(key));
    
    let local_db = LocalDb::new(env_config.ip, env_config.port).expect("Failed to connect to db");

    let pong = local_db.ping().expect("Failed to ping LocalDb");
    println!("LocalDb ping successful: {}", pong);
}
