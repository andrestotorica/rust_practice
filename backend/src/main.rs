mod env;
mod local_db;
use local_db::LocalDb;

const DEFAULT_TOKENS: [&str; 2] = ["UNI", "ZRX"];

fn main() {
    let env_config = env::load_from_env(|key| std::env::var(key));
    
    let local_db = LocalDb::new(env_config.ip, env_config.port).expect("Failed to connect to db");

    let tokens = local_db.read_tokens_or_defaults(&DEFAULT_TOKENS).expect("Failed to read tokens");
    println!("Tokens: {:?}", tokens);
}
