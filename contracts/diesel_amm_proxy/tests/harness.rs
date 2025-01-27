// use fuels::{
//     accounts::wallet::WalletUnlocked,
//     prelude::*,
//     programs::contract::LoadConfiguration,
//     types::{ContractId, Identity},
// };


//     // Generate bindings for the proxy contract
//     abigen!(Contract(
//         name = "DieselAMMProxy",
//         abi = "out/debug/diesel_amm_proxy-abi.json"
//     ),
//     Contract(
//         name = "DieselAMMContract",
//         abi = "../diesel_amm_contract/out/debug/diesel_amm_contract-abi.json"
//     ));




// // Import State directly from generated bindings after abigen
// use standards::src5::State;

// pub struct TestContext {
//     pub deployer: WalletUnlocked,
//     pub proxy_instance: DieselAMMProxy<WalletUnlocked>,
//     pub implementation_id: ContractId,
// }

// impl TestContext {
//     pub async fn new() -> Self {
//         // Setup deployer wallet
//         let deployer = WalletUnlocked::new_random(None);
        
//         // Deploy implementation first
//         let implementation_id = Contract::load_from(
//             "../diesel_amm_contract/out/debug/diesel_amm_contract.bin",
//             LoadConfiguration::default()
//         )
//         .unwrap()
//         .deploy(&deployer, TxPolicies::default())
//         .await
//         .unwrap()
//         .into();

//         // Deploy proxy
//         let proxy_id = Contract::load_from(
//             "./out/debug/diesel_amm_proxy.bin",
//             LoadConfiguration::default()
//         )
//         .unwrap()
//         .deploy(&deployer, TxPolicies::default())
//         .await
//         .unwrap();

//         let proxy_instance = DieselAMMProxy::new(proxy_id.clone(), deployer.clone());

//         Self {
//             deployer,
//             proxy_instance,
//             implementation_id,
//         }
//     }

//     pub async fn setup_initialized_proxy(&self) -> DieselAMMProxy<WalletUnlocked> {
//         let call_params = CallParameters::default();
        
//         self.proxy_instance
//             .methods()
//             .initialize(
//                 Identity::Address(self.deployer.address().into()),
//                 self.implementation_id,
//             )
//             .call_params(call_params)
//             .unwrap()
//             .call()
//             .await
//             .unwrap();

//         self.proxy_instance.clone()
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[tokio::test]
//     async fn test_proxy_initialization() {
//         let context = TestContext::new().await;
//         let proxy = context.setup_initialized_proxy().await;

//         let target = proxy
//             .methods()
//             .proxy_target()
//             .call()
//             .await
//             .unwrap()
//             .value;

//         assert_eq!(target, Some(context.implementation_id));
//     }

//     #[tokio::test]
//     async fn test_proxy_owner() {
//         let context = TestContext::new().await;
//         let proxy = context.setup_initialized_proxy().await;

//         let owner = proxy
//             .methods()
//             .proxy_owner()
//             .call()
//             .await
//             .unwrap()
//             .value;

//         let expected_owner = Identity::Address(context.deployer.address().into());
//         assert_eq!(owner, State::Initialized(expected_owner));
//     }

//     #[tokio::test]
//     #[should_panic(expected = "AlreadyInitialized")]
//     async fn test_cannot_initialize_twice() {
//         let context = TestContext::new().await;
//         let proxy = context.setup_initialized_proxy().await;

//         // Try to initialize again
//         proxy
//             .methods()
//             .initialize(
//                 Identity::Address(context.deployer.address().into()),
//                 context.implementation_id,
//             )
//             .call()
//             .await
//             .unwrap();
//     }

//     #[tokio::test]
//     async fn test_proxy_upgrade() {
//         let context = TestContext::new().await;
//         let proxy = context.setup_initialized_proxy().await;

//         // Deploy new implementation
//         let new_implementation_id = Contract::load_from(
//             "../diesel_amm_contract/out/debug/diesel_amm_contract.bin",
//             LoadConfiguration::default()
//         )
//         .unwrap()
//         .deploy(&context.deployer, TxPolicies::default())
//         .await
//         .unwrap()
//         .into();

//         // Upgrade proxy
//         proxy
//             .methods()
//             .set_proxy_target(new_implementation_id)
//             .call()
//             .await
//             .unwrap();

//         let target = proxy
//             .methods()
//             .proxy_target()
//             .call()
//             .await
//             .unwrap()
//             .value;

//         assert_eq!(target, Some(new_implementation_id));
//     }

//     #[tokio::test]
//     #[should_panic(expected = "InvalidOwner")]
//     async fn test_non_owner_cannot_upgrade() {
//         let context = TestContext::new().await;
//         let proxy = context.setup_initialized_proxy().await;

//         // Create non-owner wallet
//         let non_owner = WalletUnlocked::new_random(None);
//         let non_owner_proxy = DieselAMMProxy::new(
//             proxy.contract_id().clone(),
//             non_owner,
//         );

//         // Try to upgrade (should fail)
//         non_owner_proxy
//             .methods()
//             .set_proxy_target(context.implementation_id)
//             .call()
//             .await
//             .unwrap();
//     }
// }
use fuels::{
    accounts::wallet::WalletUnlocked,
    prelude::*,
    programs::contract::Contract,
    types::{ContractId, Identity, Bits256},
};

// Import State from the standards
use fuels::programs::call_response::FuelCallResponse;

/////////////////////////////////////////////////////////////////////////////
// Load the contract ABIs
/////////////////////////////////////////////////////////////////////////////
abigen!(
    Contract(
        name = "DieselAMMContract",
        abi = "../diesel_amm_contract/out/debug/diesel_amm_contract-abi.json"
    ),
    Contract(
        name = "DieselAMMProxy",
        abi = "out/debug/diesel_amm_proxy-abi.json"
    )
);

/////////////////////////////////////////////////////////////////////////////
// Constants & Types
/////////////////////////////////////////////////////////////////////////////
const TARGET_SLOT: Bits256 = Bits256([
    0x7b, 0xb4, 0x58, 0xad, 0xc1, 0xd1, 0x18, 0x71, 
    0x33, 0x19, 0xa5, 0xba, 0xa0, 0x0a, 0x2d, 0x04,
    0x9d, 0xd6, 0x4d, 0x29, 0x16, 0x47, 0x7d, 0x26,
    0x88, 0xd7, 0x69, 0x70, 0xc8, 0x98, 0xcd, 0x55,
]);

const OWNER_SLOT: Bits256 = Bits256([
    0xbb, 0x79, 0x92, 0x7b, 0x15, 0xd9, 0x25, 0x9e,
    0xa3, 0x16, 0xf2, 0xec, 0xb2, 0x29, 0x7d, 0x6c,
    0xc8, 0x85, 0x18, 0x88, 0xa9, 0x82, 0x78, 0xc0,
    0xa2, 0xe0, 0x3e, 0x1a, 0x09, 0x1e, 0xa7, 0x54,
]);

/////////////////////////////////////////////////////////////////////////////
// Test Setup
/////////////////////////////////////////////////////////////////////////////
pub struct Contracts {
    pub proxy: DieselAMMProxy<WalletUnlocked>,
    pub implementation: DieselAMMContract<WalletUnlocked>,
}

pub struct Wallets {
    pub deployer: WalletUnlocked,
    pub other: WalletUnlocked,
}

pub struct TestContext {
    pub contracts: Contracts,
    pub wallets: Wallets,
}

impl TestContext {
    pub async fn new() -> Self {
        let deployer = WalletUnlocked::new_random(None);
        let other = WalletUnlocked::new_random(None);

        let contracts = Self::deploy_contracts(&deployer).await;

        Self {
            contracts,
            wallets: Wallets { deployer, other },
        }
    }

    async fn deploy_contracts(deployer: &WalletUnlocked) -> Contracts {
        // Deploy implementation
        let implementation_id = Contract::load_from(
            "../diesel_amm_contract/out/debug/diesel_amm_contract.bin",
            LoadConfiguration::default()
        )
        .unwrap()
        .deploy(deployer, TxPolicies::default())
        .await
        .unwrap();

        let implementation = DieselAMMContract::new(implementation_id.clone(), deployer.clone());

        // Deploy proxy
        let proxy_id = Contract::load_from(
            "./out/debug/diesel_amm_proxy.bin",
            LoadConfiguration::default()
        )
        .unwrap()
        .deploy(deployer, TxPolicies::default())
        .await
        .unwrap();

        let proxy = DieselAMMProxy::new(proxy_id.clone(), deployer.clone());

        Contracts {
            proxy,
            implementation,
        }
    }

    pub async fn initialize_proxy(&self) -> FuelCallResponse<()> {
        self.contracts.proxy
            .methods()
            .initialize_proxy()
            .call_params(CallParameters::default())
            .unwrap()
            .call()
            .await
            .unwrap()
    }
}

/////////////////////////////////////////////////////////////////////////////
// Test Cases
/////////////////////////////////////////////////////////////////////////////
mod success {
    use super::*;

    #[tokio::test]
    async fn initializes_with_initial_values() {
        let context = TestContext::new().await;
        context.initialize_proxy().await;

        let target = context.contracts.proxy
            .methods()
            .proxy_target()
            .call()
            .await
            .unwrap()
            .value;

        let owner = context.contracts.proxy
            .methods()
            .proxy_owner()
            .call()
            .await
            .unwrap()
            .value;

        assert_eq!(target, None);
        assert!(matches!(owner, State::Initialized(_)));
    }

    #[tokio::test]
    async fn owner_can_set_target() {
        let context = TestContext::new().await;
        context.initialize_proxy().await;

        // Set target
        context.contracts.proxy
            .methods()
            .set_proxy_target(context.contracts.implementation.contract_id())
            .call()
            .await
            .unwrap();

        // Verify target
        let target = context.contracts.proxy
            .methods()
            .proxy_target()
            .call()
            .await
            .unwrap()
            .value;

        assert_eq!(target, Some(context.contracts.implementation.contract_id()));
    }

    #[tokio::test]
    async fn owner_can_transfer_ownership() {
        let context = TestContext::new().await;
        context.initialize_proxy().await;

        // Transfer ownership
        let new_owner = Identity::Address(context.wallets.other.address().into());
        context.contracts.proxy
            .methods()
            .set_proxy_owner(State::Initialized(new_owner.clone()))
            .call()
            .await
            .unwrap();

        // Verify new owner
        let owner = context.contracts.proxy
            .methods()
            .proxy_owner()
            .call()
            .await
            .unwrap()
            .value;

        assert_eq!(owner, State::Initialized(new_owner));
    }
}

mod revert {
    use super::*;

    #[tokio::test]
    #[should_panic(expected = "CannotReinitialized")]
    async fn cannot_initialize_twice() {
        let context = TestContext::new().await;
        
        // First initialization
        context.initialize_proxy().await;
        
        // Second initialization should fail
        context.initialize_proxy().await;
    }

    #[tokio::test]
    #[should_panic(expected = "NotOwner")]
    async fn non_owner_cannot_set_target() {
        let context = TestContext::new().await;
        context.initialize_proxy().await;

        // Try to set target with non-owner
        let non_owner_proxy = context.contracts.proxy
            .with_account(context.wallets.other.clone())
            .unwrap();

        non_owner_proxy
            .methods()
            .set_proxy_target(context.contracts.implementation.contract_id())
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "NotOwner")]
    async fn non_owner_cannot_transfer_ownership() {
        let context = TestContext::new().await;
        context.initialize_proxy().await;

        // Try to transfer ownership with non-owner
        let non_owner_proxy = context.contracts.proxy
            .with_account(context.wallets.other.clone())
            .unwrap();

        non_owner_proxy
            .methods()
            .set_proxy_owner(State::Initialized(Identity::Address(context.wallets.other.address().into())))
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "TargetNotSet")]
    async fn cannot_use_fallback_without_target() {
        let context = TestContext::new().await;
        context.initialize_proxy().await;

        // Try to call a non-existent function (will use fallback)
        context.contracts.proxy
            .methods()
            .non_existent_function()
            .call()
            .await
            .unwrap();
    }
}

mod storage {
    use super::*;

    #[tokio::test]
    async fn correct_storage_slots() {
        let context = TestContext::new().await;
        
        // Get storage slots from contract
        let target_slot = context.contracts.proxy
            .methods()
            .get_target_slot()
            .call()
            .await
            .unwrap()
            .value;

        let owner_slot = context.contracts.proxy
            .methods()
            .get_owner_slot()
            .call()
            .await
            .unwrap()
            .value;

        // Verify slots match constants
        assert_eq!(target_slot, TARGET_SLOT);
        assert_eq!(owner_slot, OWNER_SLOT);
    }
}

mod proxy_functionality {
    use super::*;

    #[tokio::test]
    async fn can_delegate_calls() {
        let context = TestContext::new().await;
        context.initialize_proxy().await;

        // Set implementation as target
        context.contracts.proxy
            .methods()
            .set_proxy_target(context.contracts.implementation.contract_id())
            .call()
            .await
            .unwrap();

        // Now try to call implementation function through proxy
        // Note: Replace with actual implementation function
        let result = context.contracts.proxy
            .methods()
            .implementation_function()
            .call()
            .await
            .unwrap();

        // Verify result
        assert!(result.value);
    }

    #[tokio::test]
    async fn maintains_proxy_storage_context() {
        let context = TestContext::new().await;
        context.initialize_proxy().await;

        // Set implementation as target
        let proxy = &context.contracts.proxy;
        proxy
            .methods()
            .set_proxy_target(context.contracts.implementation.contract_id())
            .call()
            .await
            .unwrap();

        // Verify storage stays in proxy context
        let owner_before = proxy
            .methods()
            .proxy_owner()
            .call()
            .await
            .unwrap()
            .value;

        // Make delegated call that might modify storage
        proxy
            .methods()
            .implementation_function()
            .call()
            .await
            .unwrap();

        // Verify storage wasn't affected
        let owner_after = proxy
            .methods()
            .proxy_owner()
            .call()
            .await
            .unwrap()
            .value;

        assert_eq!(owner_before, owner_after);
    }
}

mod state_transitions {
    use super::*;

    #[tokio::test]
    async fn complete_ownership_lifecycle() {
        let context = TestContext::new().await;
        
        // 1. Initialize
        context.initialize_proxy().await;

        // 2. Verify initial owner
        let initial_owner = context.contracts.proxy
            .methods()
            .proxy_owner()
            .call()
            .await
            .unwrap()
            .value;
        assert!(matches!(initial_owner, State::Initialized(_)));

        // 3. Transfer ownership
        let new_owner = Identity::Address(context.wallets.other.address().into());
        context.contracts.proxy
            .methods()
            .set_proxy_owner(State::Initialized(new_owner.clone()))
            .call()
            .await
            .unwrap();

        // 4. Verify new owner
        let current_owner = context.contracts.proxy
            .methods()
            .proxy_owner()
            .call()
            .await
            .unwrap()
            .value;
        assert_eq!(current_owner, State::Initialized(new_owner));

        // 5. Revoke ownership
        let proxy_with_new_owner = context.contracts.proxy
            .with_account(context.wallets.other.clone())
            .unwrap();
        proxy_with_new_owner
            .methods()
            .set_proxy_owner(State::Revoked)
            .call()
            .await
            .unwrap();

        // 6. Verify revoked state
        let final_state = context.contracts.proxy
            .methods()
            .proxy_owner()
            .call()
            .await
            .unwrap()
            .value;
        assert_eq!(final_state, State::Revoked);
    }

    #[tokio::test]
    async fn complete_target_lifecycle() {
        let context = TestContext::new().await;
        
        // 1. Initialize
        context.initialize_proxy().await;

        // 2. Verify initial target
        let initial_target = context.contracts.proxy
            .methods()
            .proxy_target()
            .call()
            .await
            .unwrap()
            .value;
        assert_eq!(initial_target, None);

        // 3. Set first implementation
        context.contracts.proxy
            .methods()
            .set_proxy_target(context.contracts.implementation.contract_id())
            .call()
            .await
            .unwrap();

        // 4. Verify first implementation
        let first_target = context.contracts.proxy
            .methods()
            .proxy_target()
            .call()
            .await
            .unwrap()
            .value;
        assert_eq!(first_target, Some(context.contracts.implementation.contract_id()));

        // 5. Deploy and set new implementation
        let new_implementation_id = Contract::load_from(
            "../diesel_amm_contract/out/debug/diesel_amm_contract.bin",
            LoadConfiguration::default()
        )
        .unwrap()
        .deploy(&context.wallets.deployer, TxPolicies::default())
        .await
        .unwrap();

        context.contracts.proxy
            .methods()
            .set_proxy_target(new_implementation_id.into())
            .call()
            .await
            .unwrap();

        // 6. Verify new implementation
        let final_target = context.contracts.proxy
            .methods()
            .proxy_target()
            .call()
            .await
            .unwrap()
            .value;
        assert_eq!(final_target, Some(ContractId::from(new_implementation_id)));
    }
}

