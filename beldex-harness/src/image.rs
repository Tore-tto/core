pub const BELDEXD_RPC_PORT: u16 = 18081; // or your local port
pub const WALLET_RPC_PORT: u16 = 18083;

#[derive(Clone)]
pub struct BeldexConfig {
    pub daemon_url: String,
    pub wallet_url: String,
}

impl Default for BeldexConfig {
    fn default() -> Self {
        Self {
            daemon_url: format!("http://127.0.0.1:{}", BELDEXD_RPC_PORT),
            wallet_url: format!("http://127.0.0.1:{}", WALLET_RPC_PORT),
        }
    }
}