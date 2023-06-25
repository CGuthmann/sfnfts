use std::{ops::Mul, str::FromStr};

use crate::{FractionalNFT, ShieldedNFT, NFT};

use num_bigint::{BigInt, RandBigInt};
use rand::thread_rng;

#[derive(Debug)]
pub struct ShieldedFractionalNFT {
    source_hash: BigInt,
    public_key_owner: BigInt,
    total_quantity: BigInt,
    fraction: [BigInt; 2],
    cosumed: bool,
    salt: BigInt
}

impl NFT for ShieldedFractionalNFT {
    fn new(source: &BigInt, value: &BigInt, owner: &BigInt) -> Self {
        ShieldedFractionalNFT {
            source_hash: source.clone(),
            total_quantity: value.clone(),
            public_key_owner: owner.clone(),
            fraction: [
                BigInt::from_str("1").unwrap(),
                BigInt::from_str("1").unwrap(),
            ],
            cosumed: false,
            salt: get_rand_int()
        }
    }

    fn transact(&self, target: &BigInt) -> Result<Self, Box<dyn std::error::Error>> {
        let minted = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target.clone(),
            fraction: self.fraction.clone(),
            cosumed: false,
            salt: get_rand_int()
        };
        Ok(minted)
    }

    fn burn(&self) {
        todo!()
    }

    fn get_value(&self) -> BigInt {
        self.total_quantity.clone()
    }
}

impl FractionalNFT for ShieldedFractionalNFT {
    fn transact_split(
        &self,
        fraction: [BigInt; 2],
        target_a: &BigInt,
        target_b: &BigInt,
    ) -> Result<[Self; 2], Box<dyn std::error::Error>> {

        let nom = fraction[0].clone().mul(self.fraction[0].clone());
        let denom = self.fraction[1].clone().mul(fraction[1].clone());

        let nft_a = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_a.clone(),
            fraction: [nom.clone(), denom.clone()],
            cosumed: false,
            salt: get_rand_int()
        };
        let nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_b.clone(),
            fraction: [denom.clone() - nom, denom],
            cosumed: false,
            salt: get_rand_int()
        };

        return Ok([nft_a, nft_b]);
    }
}

impl ShieldedNFT for ShieldedFractionalNFT {}

fn get_rand_int() -> BigInt{
    thread_rng().gen_bigint_range(&BigInt::from(0),
    &BigInt::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495617").unwrap())
}


#[cfg(test)]
mod test {
    use std::str::FromStr;

    use num_bigint::{BigInt};

    use crate::{shielded_fractional_nft::ShieldedFractionalNFT, *};

    #[test]
    fn test1() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("123").unwrap();
        let owner = BigInt::from_str("46").unwrap();

        let nft = ShieldedFractionalNFT::new(&source, &value, &owner);

        dbg!(nft);
    }
}
