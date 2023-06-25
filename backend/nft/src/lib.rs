use std::error::Error;

use num_bigint::BigInt;


pub mod shielded_fractional_nft;

pub trait NFT: Sized {
    //Instatiates a new NFT.
    fn new(source: &BigInt, value: &BigInt, owner: &BigInt) -> Self;

    fn transact(&self, target: &BigInt) -> Result<Self, Box<dyn Error>>;

    fn burn(&self);

    fn get_value(&self) -> BigInt;
}

pub trait FractionalNFT: NFT {
    fn transact_split(
        &self,
        fraction: [BigInt; 2],
        target_a: &BigInt,
        target_b: &BigInt,
    ) -> Result<[Self;2], Box<dyn Error>>;
}

pub trait ShieldedNFT: NFT{

}
