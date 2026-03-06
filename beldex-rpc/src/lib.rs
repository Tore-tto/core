#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::dbg_macro
)]
#![forbid(unsafe_code)]

mod rpc;

pub use self::rpc::*;
use std::str::FromStr;
use anyhow::{bail, Context, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeldexNetwork {
    Mainnet,
    Testnet,
    Stagenet,
}

#[derive(Debug, Clone)]
pub struct BeldexAddress {
    pub network: BeldexNetwork,
    pub raw_bytes: Vec<u8>,
    pub original: String,
}
impl BeldexAddress {
    pub fn to_monero_address(&self) -> Result<monero::Address> {
        monero::Address::from_str(&self.original)
            .map_err(|e| anyhow::anyhow!("Failed to convert beldex address to monero address: {}", e))
    }
}

impl FromStr for BeldexAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Beldex uses Monero-style block base58
        let decoded = decode_beldex_base58(s)
            .map_err(|e| anyhow::anyhow!("Failed to base58-decode beldex address: {}", e))?;

        if decoded.len() < 69 {
            bail!("Decoded address too short: {} bytes", decoded.len());
        }

        let prefix = decoded[0];

        let network = match prefix {
            0xd1 => BeldexNetwork::Mainnet,   
            53   => BeldexNetwork::Testnet,
            24   => BeldexNetwork::Stagenet,
            _    => bail!("Unknown Beldex address prefix: {} (0x{:02x})", prefix, prefix),
        };

        Ok(BeldexAddress {
            network,
            raw_bytes: decoded,
            original: s.to_string(),
        })
    }
}

fn decode_beldex_base58(s: &str) -> Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    const FULL_ENCODED_BLOCK_SIZE: usize = 11;
    const FULL_BLOCK_SIZE: usize = 8;
    // encoded block size -> decoded byte size
    const BLOCK_SIZES: [usize; 12] = [0, 0, 1, 2, 0, 3, 4, 5, 0, 6, 7, 8];

    let s = s.as_bytes();
    let mut result = Vec::new();

    let full_blocks = s.len() / FULL_ENCODED_BLOCK_SIZE;
    let last_block_len = s.len() % FULL_ENCODED_BLOCK_SIZE;

    let decode_block = |block: &[u8], out_size: usize| -> Result<Vec<u8>> {
        let mut n: u128 = 0;
        for &c in block {
            let digit = ALPHABET.iter().position(|&x| x == c)
                .ok_or_else(|| anyhow::anyhow!("Invalid base58 character: {}", c as char))?;
            n = n * 58 + digit as u128;
        }
        Ok(n.to_be_bytes()[16 - out_size..].to_vec())
    };

    for i in 0..full_blocks {
        let block = &s[i * FULL_ENCODED_BLOCK_SIZE..(i + 1) * FULL_ENCODED_BLOCK_SIZE];
        result.extend(decode_block(block, FULL_BLOCK_SIZE)?);
    }

    if last_block_len > 0 {
        let block = &s[full_blocks * FULL_ENCODED_BLOCK_SIZE..];
        let out_size = BLOCK_SIZES[last_block_len];
        if out_size == 0 {
            bail!("Invalid encoded block size: {}", last_block_len);
        }
        result.extend(decode_block(block, out_size)?);
    }

    Ok(result)
}

pub fn parse_beldex_address(s: &str) -> Result<BeldexAddress> {
    s.parse::<BeldexAddress>()
        .with_context(|| format!(
            "Failed to parse {} as a beldex address, please make sure it is a valid address", s
        ))
}
impl BeldexNetwork {
    pub fn into_monero_network(self) -> ::monero::Network {
        match self {
            BeldexNetwork::Mainnet  => ::monero::Network::Mainnet,
            BeldexNetwork::Testnet  => ::monero::Network::Testnet,
            BeldexNetwork::Stagenet => ::monero::Network::Stagenet,
        }
    }
}
