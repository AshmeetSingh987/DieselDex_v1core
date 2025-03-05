pub const ABI_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../contracts");

pub const AMM_CONTRACT_BINARY_PATH: &str =
    "../contracts/diesel_amm_contract/out/debug/diesel_amm_contract.bin";
pub const HOOK_CONTRACT_BINARY_PATH: &str =
    "../contracts/diesel_validation_hook/out/debug/diesel_validation_hook.bin";
pub const MOCK_TOKEN_CONTRACT_BINARY_PATH: &str =
    "../contracts/mocks/mock_token/out/debug/mock_token.bin";
pub const PROXY_CONTRACT_BINARY_PATH: &str =
    "../contracts/diesel_amm_proxy/out/debug/diesel_amm_proxy.bin";