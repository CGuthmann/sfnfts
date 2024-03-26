use std::error::Error;

use num_bigint::BigInt;
use crypto::{merkle_tree::{HashTree,fill_vec}, HashFunction};
use ark_circom_service::{ArkCircomFullProof,Bn254};
use serde::{Deserialize, Serialize};

pub mod remote_ledger_authority;

pub trait LedgerAuthority<H: HashFunction, const BRANCHING_FACTOR: usize> {
    fn mint(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<(usize,usize),Box<dyn Error>>;

    fn transfer(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn Error>>;

    fn burn(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<(usize,usize),Box<dyn Error>>;

    fn get_tree(&self) -> Vec<HashTree<H,BRANCHING_FACTOR>>;  
    fn get_trust_list(&self)-> HashTree<H,2>;
    fn get_nullifier(&self) -> Vec<BigInt>;
}

#[derive(Debug,Deserialize,Serialize)]
pub struct DefaultLedgerAuthority<H: HashFunction, const BRANCHING_FACTOR: usize>{

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
    pub nullifier: Vec<BigInt>,
    leave_count: usize
}

impl<H:HashFunction, const BRANCHING_FACTOR: usize> DefaultLedgerAuthority<H,BRANCHING_FACTOR>  {
    ///Initializes a new ledger authority
    pub fn new(tree_depth: u32, mut trust_list: Vec<String>)
    -> Self{
        

        fill_vec(&mut trust_list, 2);

        let leave_count = BRANCHING_FACTOR.pow(tree_depth);
        let leaves = vec!["0".to_owned(); leave_count];

        return DefaultLedgerAuthority{
            tree: vec![HashTree::<H,BRANCHING_FACTOR>::new(&leaves)],
            current_tree_index: 0,
            next_index: 0,
            tree_depth,
            trust_list: HashTree::<H,2>::new(&trust_list),
            nullifier: Vec::new(),
            leave_count
        };
    }

    fn increase_index(&mut self){

        self.next_index += 1;

        if self.next_index >= self.leave_count {
            self.tree.push(HashTree::<H,BRANCHING_FACTOR>::new(&vec!["0".to_owned(); self.leave_count]));

            self.current_tree_index +=1;
            self.next_index = 0;
        }

    }
}

impl<H:HashFunction, const BRANCHING_FACTOR: usize> LedgerAuthority<H,BRANCHING_FACTOR> for DefaultLedgerAuthority<H,BRANCHING_FACTOR>  {

    ///Mints a new nft and returns its index in the merkle tree
    /// 
    /// # Arguments
    /// 
    /// * 'proof' - A ZKP proofing the validity of the new NFT (nft_mint)
    fn mint(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<(usize,usize),Box<dyn Error>>{
        if proof.verify()? == false {
            Err("Invalid proof!")?
        }
        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[0].to_string());
        let index = (self.current_tree_index,self.next_index);

        self.increase_index();

        Ok(index)
    }

    ///Burns a nft
    /// 
    /// # Arguments
    /// 
    /// * 'proof' - A ZKP proofing the validity of the NFT burn (nft_burn)
    fn burn(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<(usize,usize), Box<dyn Error>>{

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
        let index = (self.current_tree_index,self.next_index);

        
        self.nullifier.push(proof.outputs[0].clone());

        self.increase_index();
        Ok(index)
    }

    ///Transfers and splits an nft to two parties and returns the merkle tree indices
    /// 
    /// # Arguments
    /// 
    /// * 'proof' - A ZKP proofing the validity of the new NFT (nft_transfer)
    fn transfer(&mut self, proof: &ArkCircomFullProof<Bn254>)
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
        let index_a = (self.current_tree_index,self.next_index);
        self.increase_index();

        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[4].to_string());
        let index_b = (self.current_tree_index,self.next_index);
        self.increase_index();

        self.nullifier.push(proof.outputs[0].clone());

        Ok((index_a,index_b))       
        
    }



    fn get_tree(&self) -> Vec<HashTree<H,BRANCHING_FACTOR>>{
        self.tree.clone()
    }
    
    fn get_trust_list(&self)-> HashTree<H,2>{
        self.trust_list.clone()
    }

    fn get_nullifier(&self) -> Vec<BigInt>{
        self.nullifier.clone()
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
            DefaultLedgerAuthority::<PoseidonHasher,2>::new(3, vec![private_key.to_string()]);
        
    }

    #[test]
    fn serialize_ledger_authority(){

        let private_key = 
        BigInt::from_str_radix("0001020304050607080900010203040506070809000102030405060708090001",16).unwrap();

        let ledger_authority = 
            DefaultLedgerAuthority::<PoseidonHasher,2>::new(3, vec![private_key.to_string()]);

        let json = serde_json::to_string(&ledger_authority);
    }
}