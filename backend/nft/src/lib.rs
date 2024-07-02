use std::error::Error;

use crypto::{poseidon::PoseidonHasher, merkle_tree::MerkleProof};
use ledger_authority::{DefaultLedgerAuthority, LedgerAuthority};
use num_bigint::BigInt;


pub mod shielded_fractional_nft;

///Trait for all NFTs
pub trait NFT: Sized {
    ///Mints a new NFT
    /// # Arguments
    /// * `source` - source/type ID
    /// * `value` - stored value in the [NFT]
    /// * `ledger_authority` - [LedgerAuthority] in which this [NFT] will be created
    /// * `owner` - hashed owner private key
    /// * `trust_list_proof` - containment proof of the private key in the [LedgerAuthority] trust list
    /// * `private_key` - private key corresponding to the trust_list_proof
    ///  
    /// # Return
    /// A minted [NFT]
    fn new(source: &BigInt, value: &BigInt, owner: &BigInt,
         ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>, trust_list_proof: &MerkleProof<PoseidonHasher,2>,private_key: &BigInt) 
         -> Result<Self,Box<dyn Error>> ;

    /// Transfers the [NFT] to another private key
    /// # Arguments
    /// * `ledger_authority` - [LedgerAuthority] in which this [NFT] is registered
    /// * `target` - hashed new private key
    /// * `private_key` - private key corresponding to hashed in the current [NFT]
    ///  
    /// # Return
    /// The updated [NFT]
    fn transfer(&self, ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>, target: &BigInt, private_key: &BigInt) -> Result<Self, Box<dyn Error>>;

    ///Burns the NFT
    /// # Arguments
    /// * `private_key` - private key corresponding to hashed in the current [NFT]
    /// * `ledger_authority` - [LedgerAuthority] in which this [NFT] will be created
    ///  
    /// # Return
    /// The burned [NFT]
    fn burn(&mut self,
        private_key: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>) -> Result<(), Box<dyn std::error::Error>>;

    ///Returns the current value of the NFT
    ///  
    /// # Return
    /// [BigInt] of the current value of the NFT
    fn get_value(&self) -> BigInt;
}

///Trait for all fractional NFTs
pub trait FractionalNFT: NFT {

    /// Transfers and splits the [NFT] to two private key
    /// # Arguments
    /// * `private_key` - private key corresponding to hashed in the current [NFT]
    /// * `fraction` - Pair (numerator, denominator) corresponding to the fraction of the current [NFT],
    ///                 that will be allocated to target_a
    /// * `target_a` - hashed private key for the first new [NFT]
    /// * `target_b` - hashed private key for the second new [NFT]
    /// * `ledger_authority` - [LedgerAuthority] in which this [NFT] is registered
    ///  
    /// # Return
    /// The two new [NFT]
    fn transfer_split(
        &self,
        private_key: &BigInt,
        fraction: [BigInt; 2],
        target_a: &BigInt,
        target_b: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>
    ) -> Result<(Self,Self), Box<dyn Error>>;
}

///Trait for all shielded NFTs
pub trait ShieldedNFT: NFT{

}
