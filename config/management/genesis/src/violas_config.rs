use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{string::String, vec::Vec};

//#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: u8,
    pub validators: Vec<String>,
}

impl Config {
    #[cfg(test)]
    pub fn new(chain_id: u8, validators: Vec<String>) -> Self {
        Self {
            chain_id,
            validators,
        }
    }
}

pub fn read_violas_configuration(config_file_name: &str) -> Result<Config> {
    let f = std::fs::File::open(config_file_name)?;
    let c: Config = serde_yaml::from_reader(f)?;
    Ok(c)
}

#[cfg(test)]
pub fn test_violas_config() -> Result<()> {
    let yaml = "./test.yaml";

    let f = std::fs::File::create(yaml)?;
    let a = Config::new(
        4,
        vec![
            "/ip4/127.0.0.1/tcp/8000".to_string(),
            "/ip4/127.0.0.1/tcp/8010".to_string(),
            "/ip4/127.0.0.1/tcp/8020".to_string(),
            "/ip4/127.0.0.1/tcp/8030".to_string(),
        ],
    );

    serde_yaml::to_writer(f, &a)?;
    //println!("{}", s);

    let f = std::fs::File::open(yaml)?;
    let c: Config = serde_yaml::from_reader(f)?;

    assert_eq!(a == c, "");

    Ok(())
}
