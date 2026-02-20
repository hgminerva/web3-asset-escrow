#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// pallet_assets runtime calls
pub mod assets;

/// Errors
pub mod errors;

#[ink::contract]
mod escrow {

    use ink::prelude::vec::Vec;

    use crate::errors::{Error, RuntimeError, ContractError};
    use crate::assets::{AssetsCall, RuntimeCall};

    /// Success Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
        /// Escrow setup successful
        EscrowSetupSuccess,
        /// Escrow close successful
        EscrowCloseSuccess,
        /// Escrow open successful
        EscrowOpenSuccess,
        /// Escrow account added
        EscrowAccountAdded,
        /// Escrow account released
        EscrowAccountReleased,
    }      

    /// Escrow status
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum EscrowStatus {
        EmitSuccess(Success),
        EmitError(Error),
    }       

    /// Escrow event
    #[ink(event)]
    pub struct EscrowEvent {
        #[ink(topic)]
        operator: AccountId,
        status: EscrowStatus,
    }  

    /// Escrow Account
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Account {
        /// Unique reference from the dApp
        pub reference: u16,
        /// Account address
        pub account: AccountId,
        /// Free balance
        pub balance: u128,
        /// Recipient address
        pub recipient: AccountId,
        /// Status (0-Frozen, 1-Liquid)
        pub status: u8,
    }  

    /// Escrow storage
    #[ink(storage)]
    pub struct Escrow {
        /// Escrow asset
        pub asset_id: u128,
        /// Escrow owner
        pub owner: AccountId,
        /// Escrow manager
        pub manager: AccountId,
        /// Maximum accounts the escrow can handle
        pub maximum_accounts: u16,
        /// Escrow accounts
        pub accounts: Vec<Account>,
        /// Status (0-Open, 1-Close)
        pub status: u8,
    }


    impl Escrow {

        /// Create new escrow service
        #[ink(constructor)]
        pub fn new(asset_id: u128, 
            maximum_accounts: u16) -> Self {

            let caller: ink::primitives::AccountId = Self::env().caller();

            Self { 
                asset_id: asset_id, 
                owner: caller,
                manager: caller,
                maximum_accounts: maximum_accounts,
                accounts: Vec::new(),
                status: 0u8,
            }
        }

        /// Default setup
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0u128, 0u16)
        }

        /// Setup escrow
        #[ink(message)]
        pub fn setup(&mut self,
            asset_id: u128,
            manager: AccountId,
            maximum_accounts: u16) -> Result<(), Error> {
            
            // Setup can only be done by the owner
            let caller = self.env().caller();
            if self.env().caller() != self.owner {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // The setup will delete all existing accounts - Very Important!
            self.asset_id = asset_id;
            self.manager = manager;
            self.maximum_accounts = maximum_accounts;
            self.accounts =  Vec::new();
            self.status = 0;

            self.env().emit_event(EscrowEvent {
                operator: caller,
                status: EscrowStatus::EmitSuccess(Success::EscrowSetupSuccess),
            });

            Ok(())
        }

        /// Get the escrow information
        #[ink(message)]
        pub fn get(&self) -> (u128, AccountId, AccountId, u16, u8) {
            (
                self.asset_id,
                self.owner,
                self.manager,
                self.maximum_accounts,
                self.status,
            )
        }

        /// Close the escrow service
        #[ink(message)]
        pub fn close(&mut self) -> Result<(), Error> {

            // Closing the can only be done by the manager
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // This will close the Escrow
            self.status = 1;

            self.env().emit_event(EscrowEvent {
                operator: caller,
                status: EscrowStatus::EmitSuccess(Success::EscrowCloseSuccess),
            });

            Ok(())
        }

        /// Open the escrow service
        #[ink(message)]
        pub fn open(&mut self) -> Result<(), Error> {

            // Opening the can only be done by the manager
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // This will open the Escrow
            self.status = 0;

            self.env().emit_event(EscrowEvent {
                operator: caller,
                status: EscrowStatus::EmitSuccess(Success::EscrowOpenSuccess),
            });

            Ok(())
        }   
        
        /// Add escrow account, done only by the manager once the transfer of the asset
        /// us verified through the tx-hash
        #[ink(message)]
        pub fn add(&mut self,
            reference: u16,
            account: AccountId,
            amount: u128,
            recipient: AccountId) -> Result<(), Error> {

            // Adding escrow account can only be done by the manager once the transfer of the 
            // asset is verified through the tx-hash.
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the escrow is open
            if self.status != 0 {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::EscrowIsClose),
                });
                return Ok(());
            }

            // Check if there is a duplicate escrow account
            for a in self.accounts.iter_mut() {
                if a.account == account {
                    self.env().emit_event(EscrowEvent {
                        operator: caller,
                        status: EscrowStatus::EmitError(Error::EscrowAccountDuplicate),
                    });
                    return Ok(());
                }
            }

            // Add the escrow account
            if self.accounts.len() as u16 >= self.maximum_accounts {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::EscrowAccountMax),
                });
                return Ok(());
            }

            let new_account = Account {
                reference,
                account,
                balance: amount,
                recipient,
                status: 1, // 1 = Liquid
            };
            
            self.accounts.push(new_account);

            self.env().emit_event(EscrowEvent {
                operator: caller,
                status: EscrowStatus::EmitSuccess(Success::EscrowAccountAdded),
            });

            Ok(())
        }

        /// Released the escrow account balance to the recipient
        #[ink(message)]
        pub fn release(&mut self) -> Result<(), ContractError> {

            // Release an escrow account by the caller
            let caller = self.env().caller();

            // Check if the escrow is open
            if self.status != 0 {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::EscrowIsClose),
                });
                return Ok(());
            }

            // Locate the account of the caller and delete it from the escrow 
            for i in 0..self.accounts.len() {

                if self.accounts[i].account == caller {
                    // Transfer funds - Todo
                    self.env()
                        .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                            id: self.asset_id,
                            target: self.accounts[i].recipient.into(),
                            amount: self.accounts[i].balance,
                        }))
                        .map_err(|_| RuntimeError::CallRuntimeFailed)?;                    

                    // Remove escrow account (gas efficient)
                    self.accounts.swap_remove(i);


                    self.env().emit_event(EscrowEvent {
                        operator: caller,
                        status: EscrowStatus::EmitSuccess(Success::EscrowAccountReleased),
                    });

                    return Ok(());
                }
            }            

            self.env().emit_event(EscrowEvent {
                operator: caller,
                status: EscrowStatus::EmitError(Error::EscrowAccountNotFound),
            });

            Ok(())
        }

        /// Override, this will release the escrow account to some recipient
        #[ink(message)]
        pub fn force_release(&mut self,
            account: AccountId,
            recipient: AccountId) -> Result<(), ContractError> {

            // Override the release of the escrow account can only be done by 
            // the manager.
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the escrow is open
            if self.status != 0 {
                self.env().emit_event(EscrowEvent {
                    operator: caller,
                    status: EscrowStatus::EmitError(Error::EscrowIsClose),
                });
                return Ok(());
            }
            
            // Locate the account of the caller and delete it from the escrow 
            for i in 0..self.accounts.len() {

                if self.accounts[i].account == account {
                    // Transfer funds - Todo (Recipient must be manually provided)
                    self.env()
                        .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                            id: self.asset_id,
                            target: recipient.into(),
                            amount: self.accounts[i].balance,
                        }))
                        .map_err(|_| RuntimeError::CallRuntimeFailed)?;  

                    // Remove escrow account (gas efficient)
                    self.accounts.swap_remove(i);

                    self.env().emit_event(EscrowEvent {
                        operator: caller,
                        status: EscrowStatus::EmitSuccess(Success::EscrowAccountReleased),
                    });

                    return Ok(());
                }
            }            

            self.env().emit_event(EscrowEvent {
                operator: caller,
                status: EscrowStatus::EmitError(Error::EscrowAccountNotFound),
            });

            Ok(())            
        }

    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let Escrow = Escrow::default();
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = EscrowRef::default();

            // When
            let contract_account_id = client
                .instantiate("escrow", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<EscrowRef>(contract_account_id.clone())
                .call(|escrow| escrow.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = EscrowRef::new(false);
            let contract_account_id = client
                .instantiate("escrow", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<EscrowRef>(contract_account_id.clone())
                .call(|escrow| escrow.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<EscrowRef>(contract_account_id.clone())
                .call(|escrow| escrow.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<EscrowRef>(contract_account_id.clone())
                .call(|escrow| escrow.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
