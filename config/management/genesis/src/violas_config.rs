use anyhow::Result;
use diem_types::{
    chain_id::ChainId,
    network_address::{NetworkAddress, Protocol},
};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::str::FromStr;
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

pub fn get_chain_id_from_file() -> ChainId {
    if let Ok(genesis_config) = read_violas_configuration("genesis.yaml") {
        ChainId::new(genesis_config.chain_id)
    } else {
        ChainId::test()
    }
}

pub fn get_listen_address_from_file(index: usize) -> Option<(NetworkAddress, NetworkAddress)> {
    if let Ok(genesis_config) = read_violas_configuration("genesis.yaml") {
        // Update chain_id
        let chain_id = ChainId::new(genesis_config.chain_id);
        // update validators
        if index < genesis_config.validators.len() {
            let net_addr_str = genesis_config.validators[index].clone();
            //address format is "/ip4/10.0.0.16/tcp/80"
            if let Ok(addr) = NetworkAddress::from_str(net_addr_str.as_str()) {
                let protocols: Vec<Protocol> = addr.clone().into_iter().collect();

                if let Protocol::Tcp(port) = protocols[1] {
                    //update validator and fullnode address in genesis blob
                    let validator_network_address = addr.clone();
                    let fullnode_network_address =
                        NetworkAddress::from(protocols[0].clone()).push(Protocol::Tcp(port + 1)); // validator port + 1

                    println!(
                        "Chain Id : {}, validator address : {}, full node address : {} ",
                        chain_id, validator_network_address, fullnode_network_address
                    );

                    return Some((validator_network_address, fullnode_network_address));
                };
            };
        };
    };

    None
}
/// Reset listen address to 0.0.0.0:port
pub fn reset_listen_address(
    validator_address: &mut NetworkAddress,
    fullnode_address: &mut NetworkAddress,
) {
    //*validator_address.as_slice()[0] = Protocol::Ip4("0.0.0.0".parse().unwrap());
    //*fullnode_address.as_slice()[0] = Protocol::Ip4("0.0.0.0".parse().unwrap());

    let protocols: Vec<Protocol> = validator_address.clone().into_iter().collect();
    if let Protocol::Tcp(port) = protocols[1] {
        *validator_address = NetworkAddress::from(Protocol::Ip4("0.0.0.0".parse().unwrap()))
            .push(Protocol::Tcp(port));
        *fullnode_address = NetworkAddress::from(Protocol::Ip4("0.0.0.0".parse().unwrap()))
            .push(Protocol::Tcp(port + 1));
    }
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
