use std::error::Error;

use num_bigint::BigInt;
use crypto::{merkle_tree::{HashTree,fill_vec}, HashFunction};
use ark_circom_service::{ArkCircomFullProof,Bn254};
use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize,Serialize)]
pub struct LedgerAuthority<H: HashFunction, const BRANCHING_FACTOR: usize>{

    #[serde(bound(
        serialize = "HashTree<H,2>: Serialize",
        deserialize = "HashTree<H,2>: Deserialize<'de>"
    ))]
    pub tree: Vec<HashTree<H,BRANCHING_FACTOR>>,

    pub current_tree_index: usize,
    pub next_index: usize,
    pub tree_depth: u32,

    #[serde(bound(
        serialize = "HashTree<H,2>: Serialize",
        deserialize = "HashTree<H,2>: Deserialize<'de>"
    ))]
    pub trust_list: HashTree<H,2>,
    pub nullifier: Vec<BigInt>
}

impl<H:HashFunction, const BRANCHING_FACTOR: usize> LedgerAuthority<H,BRANCHING_FACTOR>  {

    ///Initializes a new ledger authority
    pub fn new(tree_depth: u32, mut trust_list: Vec<String>)
    -> Self{
        
        let leaves = vec!["0".to_owned(); BRANCHING_FACTOR.pow(tree_depth )];

        fill_vec(&mut trust_list, 2);

        return LedgerAuthority{
            tree: vec![HashTree::<H,BRANCHING_FACTOR>::new(&leaves)],
            current_tree_index: 0,
            next_index: 0,
            tree_depth,
            trust_list: HashTree::<H,2>::new(&trust_list),
            nullifier: Vec::new()
        };
    }

    ///Mints a new nft and returns its index in the merkle tree
    /// 
    /// # Arguments
    /// 
    /// * 'proof' - A ZKP proofing the validity of the new NFT (nft_mint)
    pub fn mint(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<(usize,usize),Box<dyn Error>>{
        if proof.verify()? == false {
            Err("Invalid proof!")?
        }
        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[0].to_string());

        self.next_index += 1;
        Ok((self.current_tree_index,self.next_index -1))
    }

    ///Burns a nft
    /// 
    /// # Arguments
    /// 
    /// * 'proof' - A ZKP proofing the validity of the NFT burn (nft_burn)
    pub fn burn(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<bool,Box<dyn Error>>{

        if self.nullifier.contains(&proof.outputs[1]){
            Err("Nullified nft!")?
        }

        if proof.verify()? == false {
            Err("Invalid proof!")?
        }
        self.tree[proof.outputs[0].to_string().parse::<usize>()?].update(self.next_index,proof.outputs[1].to_string());

        Ok(true)
    }

    ///Transfers and splits an nft to two parties and returns the merkle tree indices
    /// 
    /// # Arguments
    /// 
    /// * 'proof' - A ZKP proofing the validity of the new NFT (nft_transfer)
    pub fn transfer(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn Error>>{

        if self.nullifier.contains(&proof.outputs[0]){
            Err("Nullified nft!")?
        }

        if proof.verify()? == false {
            Err("Invalid proof!")?
        }

        if !self.tree[proof.outputs[1].to_string().parse::<usize>()?].get_root().eq(&proof.outputs[2]) {
            Err("Invalid merkle root!")?
        }

        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[3].to_string());
        self.next_index += 1;

        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[4].to_string());
        self.next_index += 1;

        self.nullifier.push(proof.outputs[0].clone());

        Ok(((self.current_tree_index,self.next_index -2),(self.current_tree_index,self.next_index -1)))       
        
    }



    pub fn get_tree(&self) -> &Vec<HashTree<H,BRANCHING_FACTOR>>{
        &self.tree
    }
    
    pub fn get_trust_list(&self)-> &HashTree<H,2>{
        &self.trust_list
    }

    pub fn get_nullifier(&self) -> &Vec<BigInt>{
        &self.nullifier
    }

}



#[cfg(test)]
mod test{
    use super::*;
    use crypto::poseidon::PoseidonHasher;
    use num_traits::Num;


    
    fn ledger_authority_check(){

        let private_key = 
        BigInt::from_str_radix("0001020304050607080900010203040506070809000102030405060708090001",16).unwrap();

        let ledger_authority = 
            LedgerAuthority::<PoseidonHasher,2>::new(3, vec![private_key.to_string()]);
        
    }

    #[test]
    fn serialize_ledger_authority(){

        let private_key = 
        BigInt::from_str_radix("0001020304050607080900010203040506070809000102030405060708090001",16).unwrap();

        let ledger_authority = 
            LedgerAuthority::<PoseidonHasher,2>::new(3, vec![private_key.to_string()]);

        let json = serde_json::to_string(&ledger_authority);
        dbg!(json);
    }
}