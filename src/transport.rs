use anyhow::{Error, Result};
use deadpool_lapin::{Config, Pool};

pub fn create_conn_pool() -> Result<Pool, Error> {
    let cfg = Config::from_env("AMQP")?;
    Ok(cfg.create_pool())
}
