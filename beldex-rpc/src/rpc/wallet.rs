use crate::rpc::{Request, Response};
use anyhow::{bail, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// JSON RPC client for monero-wallet-rpc.
#[derive(Debug, Clone)]
pub struct Client {
    pub inner: reqwest::Client,
    pub url: Url,
}

impl Client {
    /// Constructs a monero-wallet-rpc client with localhost endpoint.
    pub fn localhost(port: u16) -> Self {
        let url = format!("http://127.0.0.1:{}/json_rpc", port);
        let url = Url::parse(&url).expect("url is well formed");

        Client::new(url)
    }

    /// Constructs a monero-wallet-rpc client with `url` endpoint.
    pub fn new(url: Url) -> Self {
        Self {
            inner: reqwest::Client::new(),
            url,
        }
    }

    /// Get addresses for account by index.
    pub async fn get_address(&self, account_index: u32) -> Result<GetAddress> {
        let params = GetAddressParams { account_index };
        let request = Request::new("get_address", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("get address RPC response: {}", response);

        let r = serde_json::from_str::<Response<GetAddress>>(&response)?;
        r.into_result()
    }

    /// Gets the balance of account by index.
    pub async fn get_balance(&self, index: u32) -> Result<u64> {
        let params = GetBalanceParams {
            account_index: index,
        };
        let request = Request::new("get_balance", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!(
            "get balance of account index {} RPC response: {}",
            index, response
        );

        let r = serde_json::from_str::<Response<GetBalance>>(&response)?;

        let balance = r.into_result()?.balance;

        Ok(balance)
    }

    pub async fn create_account(&self, label: &str) -> Result<CreateAccount> {
        let params = LabelParams {
            label: label.to_owned(),
        };
        let request = Request::new("create_account", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("create account RPC response: {}", response);

        let r = serde_json::from_str::<Response<CreateAccount>>(&response)?;
        r.into_result()
    }

    /// Get accounts, filtered by tag ("" for no filtering).
    pub async fn get_accounts(&self, tag: &str) -> Result<GetAccounts> {
        let params = TagParams {
            tag: tag.to_owned(),
        };
        let request = Request::new("get_accounts", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("get accounts RPC response: {}", response);

        let r = serde_json::from_str::<Response<GetAccounts>>(&response)?;

        r.into_result()
    }

    /// Opens a wallet using `filename`.
    pub async fn open_wallet(&self, filename: &str) -> Result<()> {
        let params = OpenWalletParams {
            filename: filename.to_owned(),
        };
        let request = Request::new("open_wallet", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("open wallet RPC response: {}", response);

        // TODO: Proper error handling once switching to https://github.com/thomaseizinger/rust-jsonrpc-client/
        //  Currently blocked by https://github.com/thomaseizinger/rust-jsonrpc-client/issues/20
        if response.contains("error") {
            bail!("Failed to open wallet")
        }

        Ok(())
    }

    /// Close the currently opened wallet, after trying to save it.
    pub async fn close_wallet(&self) -> Result<()> {
        let request = Request::new("close_wallet", serde_json::json!({}));

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("close wallet RPC response: {}", response);

        if response.contains("error") {
            bail!("Failed to close wallet")
        }

        Ok(())
    }

    /// Creates a wallet using `filename`.
    pub async fn create_wallet(&self, filename: &str) -> Result<()> {
        let params = CreateWalletParams {
            filename: filename.to_owned(),
            language: "English".to_owned(),
        };
        let request = Request::new("create_wallet", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("create wallet RPC response: {}", response);

        if response.contains("error") {
            bail!("Failed to create wallet")
        }

        Ok(())
    }

    /// Transfers `amount` moneroj from `account_index` to `address`.
    pub async fn transfer(
        &self,
        account_index: u32,
        amount: u64,
        address: &str,
    ) -> Result<Transfer> {
        let dest = vec![Destination {
            amount,
            address: address.to_owned(),
        }];
        self.multi_transfer(account_index, dest).await
    }

    /// Transfers moneroj from `account_index` to `destinations`.
    pub async fn multi_transfer(
        &self,
        account_index: u32,
        destinations: Vec<Destination>,
    ) -> Result<Transfer> {
        let params = TransferParams {
            account_index,
            destinations,
            get_tx_key: true,
        };
        let request = Request::new("transfer", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("transfer RPC response: {}", response);

        let r = serde_json::from_str::<Response<Transfer>>(&response)?;
        r.into_result()
    }

    /// Get wallet block height, this might be behind monerod height.
    pub async fn block_height(&self) -> Result<BlockHeight> {
        let request = Request::new("get_height", serde_json::json!({}));

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("wallet height RPC response: {}", response);

        let r = serde_json::from_str::<Response<BlockHeight>>(&response)?;
        r.into_result()
    }

    /// Check a transaction in the blockchain with its secret key.
    pub async fn check_tx_key(
        &self,
        tx_id: &str,
        tx_key: &str,
        address: &str,
    ) -> Result<CheckTxKey> {
        let params = CheckTxKeyParams {
            txid: tx_id.to_owned(),
            tx_key: tx_key.to_owned(),
            address: address.to_owned(),
        };
        let request = Request::new("check_tx_key", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("transfer RPC response: {}", response);

        let r = serde_json::from_str::<Response<CheckTxKey>>(&response)?;
        r.into_result()
    }

    pub async fn generate_from_keys(
        &self,
        address: &str,
        spend_key: &str,
        view_key: &str,
        restore_height: u32,
    ) -> Result<GenerateFromKeys> {
        let params = GenerateFromKeysParams {
            restore_height,
            filename: view_key.into(),
            address: address.into(),
            spendkey: spend_key.into(),
            viewkey: view_key.into(),
            password: "".into(),
            autosave_current: true,
        };
        let request = Request::new("generate_from_keys", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("generate_from_keys RPC response: {}", response);

        let r = serde_json::from_str::<Response<GenerateFromKeys>>(&response)?;
        r.into_result()
    }

    pub async fn refresh(&self) -> Result<Refreshed> {
        let request = Request::new("refresh", serde_json::json!({}));

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("refresh RPC response: {}", response);

        let r = serde_json::from_str::<Response<Refreshed>>(&response)?;
        r.into_result()
    }

    /// Transfers the complete balance of the account to `address`.
    pub async fn sweep_all(&self, address: &str) -> Result<SweepAll> {
        let params = SweepAllParams {
            address: address.into(),
        };
        let request = Request::new("sweep_all", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("sweep_all RPC response: {}", response);

        let r = serde_json::from_str::<Response<SweepAll>>(&response)?;
        r.into_result()
    }

    pub async fn get_version(&self) -> Result<Version> {
        let request = Request::new("get_version", serde_json::json!({}));
        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;
        debug!("get_version RPC response: {}", response);

        let r = serde_json::from_str::<Response<Version>>(&response)?;
        r.into_result()
    }

    pub async fn get_transfer_by_txid(&self, txid: &str) -> Result<GetTransferByTxid> {
        let params = GetTransferByTxidParams {
            txid: txid.to_owned(),
        };
        let request = Request::new("get_transfer_by_txid", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("get_transfer_by_txid RPC response: {}", response);

        let r = serde_json::from_str::<Response<GetTransferByTxid>>(&response)?;
        r.into_result()
    }
}

#[derive(Serialize, Debug, Clone)]
struct GetAddressParams {
    account_index: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetAddress {
    pub address: String,
}

#[derive(Serialize, Debug, Clone)]
struct GetBalanceParams {
    account_index: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetBalance {
    pub balance: u64,
    pub blocks_to_unlock: u32,
    pub multisig_import_needed: bool,
    pub time_to_unlock: u32,
    pub unlocked_balance: u64,
}

#[derive(Serialize, Debug, Clone)]
struct LabelParams {
    label: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateAccount {
    pub account_index: u32,
    pub address: String,
}

#[derive(Serialize, Debug, Clone)]
struct TagParams {
    tag: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetAccounts {
    pub subaddress_accounts: Vec<SubAddressAccount>,
    pub total_balance: u64,
    pub total_unlocked_balance: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SubAddressAccount {
    pub account_index: u32,
    pub balance: u32,
    pub base_address: String,
    pub label: String,
    pub tag: String,
    pub unlocked_balance: u64,
}

#[derive(Serialize, Debug, Clone)]
struct OpenWalletParams {
    filename: String,
}

#[derive(Serialize, Debug, Clone)]
struct CreateWalletParams {
    filename: String,
    language: String,
}

#[derive(Serialize, Debug, Clone)]
struct TransferParams {
    // Transfer from this account.
    account_index: u32,
    // Destinations to receive XMR:
    destinations: Vec<Destination>,
    // Return the transaction key after sending.
    get_tx_key: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct Destination {
    amount: u64,
    address: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Transfer {
    pub amount: u64,
    pub fee: u64,
    pub multisig_txset: String,
    pub tx_blob: String,
    pub tx_hash: String,
    pub tx_key: String,
    pub tx_metadata: String,
    pub unsigned_txset: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct BlockHeight {
    pub height: u32,
}

#[derive(Serialize, Debug, Clone)]
struct CheckTxKeyParams {
    txid: String,
    tx_key: String,
    address: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckTxKey {
    pub confirmations: u64,
    pub received: u64,
    pub in_pool: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct GenerateFromKeysParams {
    pub restore_height: u32,
    pub filename: String,
    pub address: String,
    pub spendkey: String,
    pub viewkey: String,
    pub password: String,
    pub autosave_current: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GenerateFromKeys {
    pub address: String,
    pub info: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Refreshed {
    pub blocks_fetched: u32,
    pub received_money: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SweepAllParams {
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SweepAll {
    #[serde(default)]
    pub amount_list: Vec<u64>,
    #[serde(default)]
    pub fee_list: Vec<u64>,
    #[serde(default)]
    pub multisig_txset: String,
    pub tx_hash_list: Vec<String>,
    #[serde(default)]
    pub unsigned_txset: String,
    #[serde(default)]
    pub weight_list: Vec<u32>,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Version {
    version: u32,
}

#[derive(Serialize, Debug, Clone)]
struct GetTransferByTxidParams {
    txid: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetTransferByTxid {
    pub transfer: TransferEntry,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransferEntry {
    #[serde(default)]
    pub amount: u64,
    #[serde(default)]
    pub confirmations: u64,
    #[serde(default)]
    pub fee: u64,
    #[serde(default)]
    pub txid: String,
    #[serde(rename = "type", default)]
    pub type_: String,
}
