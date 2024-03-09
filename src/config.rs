use once_cell::sync::Lazy;


fn get_env(env: &'static str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get the {} env variable", env))
}


pub struct Config {
    pub broadcast_id: u32,
    pub guild_id: u64,
    pub bot_token: String,
    pub bot_id: u128,
}


impl Config {
    pub fn load() -> Config {
        Config {
            broadcast_id: get_env("BROADCAST_ID").parse().unwrap(),
            guild_id: get_env("GUILD_ID").parse().unwrap(),
            bot_token: get_env("BOT_TOKEN"),
            bot_id: get_env("BOT_ID").parse().unwrap(),
        }
    }
}


pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);
