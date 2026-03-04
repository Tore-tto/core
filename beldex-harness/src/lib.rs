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

//! Local Beldex Harness (No Docker)
//!
//! Connects to locally running:
//! - beldexd
//! - beldex-wallet-rpc
//!
//! Provides mining, funding and wallet helpers.

use anyhow::{anyhow, bail, Result};
use beldex_rpc::{
    beldexd,
    wallet::{self, GetAddress, Refreshed, Transfer},
};
use std::time::Duration;
use tokio::time;

const BLOCK_TIME_SECS: u64 = 1;
const WAIT_WALLET_SYNC_MILLIS: u64 = 1000;

pub const BELDEXD_RPC_PORT: u16 = 29091;
pub const WALLET_RPC_PORT: u16 = 19091;

#[derive(Clone, Debug)]
pub struct Beldex {
    beldexd: Beldexd,
    wallets: Vec<BeldexWalletRpc>,
}

impl Beldex {
    /// Connect to locally running daemon + wallets
    pub async fn new(wallet_names: Vec<String>) -> Result<Self> {
        let beldexd = Beldexd::new(BELDEXD_RPC_PORT);

        let mut wallets = vec![];

        for name in wallet_names {
            wallets.push(BeldexWalletRpc::new(&name, WALLET_RPC_PORT));
        }

        Ok(Self { beldexd, wallets })
    }

    pub fn beldexd(&self) -> &Beldexd {
        &self.beldexd
    }

    pub fn wallet(&self, name: &str) -> Result<&BeldexWalletRpc> {
        self.wallets
            .iter()
            .find(|w| w.name == name)
            .ok_or_else(|| anyhow!("Wallet not found"))
    }

}

#[derive(Clone, Debug)]
pub struct Beldexd {
    rpc_port: u16,
}

impl Beldexd {
    pub fn new(rpc_port: u16) -> Self {
        Self { rpc_port }
    }

    pub fn client(&self) -> beldexd::Client {
        beldexd::Client::localhost(self.rpc_port)
    }

    pub async fn get_height(&self) -> Result<u32> {
        Ok(self.client().get_block_count().await?)
    }
}

#[derive(Clone, Debug)]
pub struct BeldexWalletRpc {
    rpc_port: u16,
    name: String,
}

impl BeldexWalletRpc {
    pub fn new(name: &str, rpc_port: u16) -> Self {
        Self {
            rpc_port,
            name: name.to_string(),
        }
    }

    pub fn client(&self) -> wallet::Client {
        wallet::Client::localhost(self.rpc_port)
    }

    pub async fn wait_for_wallet_height(&self, height: u32) -> Result<()> {
        let mut retry = 0;

        while self.client().block_height().await?.height < height {
            if retry >= 30 {
                bail!("Wallet did not sync after 30 retries");
            }

            time::sleep(Duration::from_millis(WAIT_WALLET_SYNC_MILLIS)).await;
            retry += 1;
        }

        Ok(())
    }

    pub async fn transfer(&self, address: &str, amount: u64) -> Result<Transfer> {
        self.client().transfer(0, amount, address).await
    }

    pub async fn address(&self) -> Result<GetAddress> {
        self.client().get_address(0).await
    }

    pub async fn balance(&self) -> Result<u64> {
        self.client().refresh().await?;
        self.client().get_balance(0).await
    }

    pub async fn refresh(&self) -> Result<Refreshed> {
        self.client().refresh().await
    }
}

