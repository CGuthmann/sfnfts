use std::error::Error;

use crypto::{poseidon::PoseidonHasher, merkle_tree::MerkleProof};
use ledger_authority::LedgerAuthority;
use num_bigint::BigInt;


pub mod shielded_fractional_nft;

///Trait for all NFTs
pub trait NFT: Sized {
    //Instatiates a new NFT.
    fn new(source: &BigInt, value: &BigInt, owner: &BigInt,
         ledger_authority: & mut LedgerAuthority<PoseidonHasher,2>, trust_list_proof: &MerkleProof<PoseidonHasher,2>) 
         -> Result<Self,Box<dyn Error>> ;

    fn transfer(&self, target: &BigInt) -> Result<Self, Box<dyn Error>>;

    fn burn(&mut self) -> Result<bool, Box<dyn std::error::Error>>;

    fn get_value(&self) -> BigInt;
}

///Trait for all fractional NFTs
pub trait FractionalNFT: NFT {
    fn transfer_split(
        &self,
        private_key: &BigInt,
        fraction: [BigInt; 2],
        target_a: &BigInt,
        target_b: &BigInt,
        ledger_authority: &mut LedgerAuthority<PoseidonHasher,2>
    ) -> Result<(Self,Self), Box<dyn Error>>;
}

///Trait for all shielded NFTs
pub trait ShieldedNFT: NFT{

}
