use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_address")]
    pub grpc_address: SocketAddr,
    #[serde(deserialize_with = "deserialize_address")]
    pub rest_address: SocketAddr,
    pub memory_storage_size: usize,
    pub message_channel_size: usize,
    pub peers: Vec<PeerConfig>,
}

#[derive(Deserialize)]
pub struct PeerConfig {
    pub peer_address: String,
    pub polling_interval: u64,
}

fn deserialize_address<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    struct AddressVisitor;

    impl<'de> Visitor<'de> for AddressVisitor {
        type Value = SocketAddr;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing json data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse().map_err(E::custom)
        }
    }

    deserializer.deserialize_str(AddressVisitor)
}
