use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use web_sys::console;

/// A highly optimized library for managing NFTs (Non-Fungible Tokens) in Ethereum-compatible WebAssembly (WASM).
#[wasm_bindgen]
pub struct NFTManager {
    owner: String,
    nft_classes: HashMap<String, NFTClass>,
    tokens: HashMap<String, Token>,
    approvals: HashMap<String, HashSet<String>>, // Mapping for approval management
}

/// Structure for storing NFT class information
#[derive(Serialize, Deserialize, Clone)]
pub struct NFTClass {
    metadata: String,
    total_issuance: u32,
    owner: String,
}

/// Structure for storing individual NFT token information
#[derive(Serialize, Deserialize, Clone)]
pub struct Token {
    metadata: String,
    owner: String,
    approved: Option<String>, // Optional approved account for transfer
}

#[wasm_bindgen]
impl NFTManager {
    /// Initialize a new NFTManager
    #[wasm_bindgen(constructor)]
    pub fn new(owner: &str) -> NFTManager {
        console::log_1(&format!("NFT Manager initialized with owner: {}", owner).into());
        NFTManager {
            owner: owner.to_string(),
            nft_classes: HashMap::new(),
            tokens: HashMap::new(),
            approvals: HashMap::new(),
        }
    }

    /// Internal helper function to check if the current caller is the contract owner.
    fn is_owner(&self, caller: &str) -> Result<bool, String> {
        if caller == self.owner {
            Ok(true)
        } else {
            Err("Caller is not the contract owner".into())
        }
    }

    /// Internal helper function to check if the current caller is the owner of the class.
    fn is_owner_of_class(&self, class_id: &str, caller: &str) -> Result<bool, String> {
        if let Some(class) = self.nft_classes.get(class_id) {
            if class.owner == caller {
                Ok(true)
            } else {
                Err("Caller is not the class owner".into())
            }
        } else {
            Err("Class not found".into())
        }
    }

    /// Create a new NFT class. Only the contract owner can create classes.
    pub fn create_class(&mut self, class_id: &str, metadata: &str, caller: &str) -> Result<(), String> {
        if self.is_owner(caller)? {
            let class = NFTClass {
                metadata: metadata.to_string(),
                total_issuance: 0,
                owner: caller.to_string(),
            };

            self.nft_classes.insert(class_id.to_string(), class);
            console::log_1(&format!("Class created with ID: {}", class_id).into());
            Ok(())
        } else {
            Err("Caller is not authorized to create classes".into())
        }
    }

    /// Mint a new token within an NFT class. Only the owner of the class can mint.
    pub fn mint(&mut self, class_id: &str, token_id: &str, metadata: &str, caller: &str) -> Result<(), String> {
        if self.is_owner_of_class(class_id, caller)? {
            let token = Token {
                metadata: metadata.to_string(),
                owner: caller.to_string(),
                approved: None,
            };

            if let Some(class) = self.nft_classes.get_mut(class_id) {
                class.total_issuance += 1;
                self.tokens.insert(token_id.to_string(), token);
                console::log_1(&format!("Token minted with ID: {}", token_id).into());
                Ok(())
            } else {
                Err("Class not found".into())
            }
        } else {
            Err("Caller is not authorized to mint tokens".into())
        }
    }

    /// Transfer a token to another owner. Must be the token owner or approved operator.
    pub fn transfer(&mut self, token_id: &str, new_owner: &str, caller: &str) -> Result<(), String> {
        // Önce token sahibini öğrenelim
        let owner = self.get_owner(token_id)?;
    
        // Şimdi mutable borçlanmayı gerçekleştirebiliriz
        if let Some(token) = self.tokens.get_mut(token_id) {
            // Sahip doğrulaması
            if owner == caller {
                token.owner = new_owner.to_string();
                token.approved = None; // Clear approvals after transfer
                console::log_1(&format!("Token {} transferred to {}", token_id, new_owner).into());
                Ok(())
            } else {
                Err("Caller is not the token owner".into())
            }
        } else {
            Err("Token not found.".into())
        }
    }
    
    

    /// Approve another account to transfer the given token.
    pub fn approve(&mut self, token_id: &str, approved: &str, caller: &str) -> Result<(), String> {
        if let Some(token) = self.tokens.get_mut(token_id) {
            if token.owner != caller {
                console::log_1(&format!("Failed to approve token {}: Caller is not owner", token_id).into());
                return Err("Only the owner can approve a transfer.".into());
            }

            token.approved = Some(approved.to_string());
            self.approvals.entry(token_id.to_string()).or_default().insert(approved.to_string());
            console::log_1(&format!("Approval set for token {}: {}", token_id, approved).into());
            Ok(())
        } else {
            console::log_1(&format!("Token with ID: {} not found", token_id).into());
            Err("Token not found.".into())
        }
    }

    /// Burn a token (remove it from circulation). Only the token owner can burn the token.
    pub fn burn(&mut self, token_id: &str, caller: &str) -> Result<(), String> {
        if let Some(token) = self.tokens.get(token_id) {
            if token.owner != caller {
                console::log_1(&format!("Failed to burn token {}: Caller is not owner", token_id).into());
                return Err("Only the owner can burn the token.".into());
            }
        }

        if let Some(_token) = self.tokens.remove(token_id) {
            console::log_1(&format!("Token with ID: {} burned", token_id).into());
            Ok(())
        } else {
            console::log_1(&format!("Token with ID: {} not found", token_id).into());
            Err("Token not found.".into())
        }
    }

    /// Check the owner of a token.
    pub fn get_owner(&self, token_id: &str) -> Result<String, String> {
        if let Some(token) = self.tokens.get(token_id) {
            Ok(token.owner.clone())
        } else {
            console::log_1(&format!("Token with ID: {} not found", token_id).into());
            Err("Token not found.".into())
        }
    }

    /// Check if an address is approved for the given token.
    pub fn is_approved(&self, token_id: &str, operator: &str) -> bool {
        if let Some(approvals) = self.approvals.get(token_id) {
            approvals.contains(operator)
        } else {
            false
        }
    }

    /// Set a new owner for the NFTManager contract.
    pub fn transfer_ownership(&mut self, new_owner: &str, caller: &str) -> Result<(), String> {
        if self.is_owner(caller)? {
            self.owner = new_owner.to_string();
            console::log_1(&format!("Ownership transferred to {}", new_owner).into());
            Ok(())
        } else {
            Err("Caller is not authorized to transfer ownership".into())
        }
    }
}
