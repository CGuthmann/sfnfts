use std::error::Error;

use num_bigint::BigInt;
use crypto::{merkle_tree::{HashTree,fill_vec}, HashFunction};
use ark_circom_service::{ArkCircomFullProof,Bn254};
use serde::{Deserialize, Serialize};

pub mod remote_ledger_authority;
pub mod batched_ledger_authority;

pub trait LedgerAuthority<H: HashFunction, const BRANCHING_FACTOR: usize> {

    
    ///Mints the new NFT encapsuled in the proof
    /// # Arguments
    /// * `proof` - nft_mint ZKP, proofing the validity of the transfer
    ///  
    /// # Return
    /// The index of the minted [NFT] (tree_index, nft_index)
    fn mint(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<(usize,usize),Box<dyn Error>>;

    /// Transfers and splits the [NFT] encapsuled in the proof to two private key
    /// # Arguments
    /// * `proof` - nft_transfer ZKP, proofing the validity of the transfer
    ///  
    /// # Return
    /// The indices of the new [NFT] (tree_index, nft_index)
    fn transfer_split(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn Error>>;

    
    /// Transfers  the [NFT], which owner is another NFT
    /// # Arguments
    /// * `proof_transfer_child` - nft_transfer_child ZKP, proofing the validity of the transfer
    /// * `proofs_child_parent` - vec of nft_child_parent ZKP, proofing the validity of child to root mapping
    ///  
    /// # Return
    /// The indices of the new [NFT] (tree_index, nft_index)
    fn transfer_from_nft(&mut self, proof_transfer_child: &ArkCircomFullProof<Bn254>, proofs_child_parent: &Vec<ArkCircomFullProof<Bn254>>)
    -> Result<(usize,usize),Box<dyn Error>>;

    ///Burns the [NFT] encapsuled in the proof
    /// # Arguments
    /// * `proof` - nft_burn ZKP, proofing the validity of the burn
    ///  
    /// # Return
    /// The index of the burned [NFT] (tree_index, nft_index)
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
    pub tree_roots: Vec<Vec<BigInt>>,
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
    
    /// Initializes a new [DefaultLedgerAuthority]
    /// # Arguments
    /// * `tree_depth` - depth of the [HashTree] storing the [NFT]
    /// * `trust_list` - list of the hashed private key of the trusted partys
    ///  
    /// # Return
    /// A new [DefaultLedgerAuthority]
    pub fn new(tree_depth: u32, mut trust_list: Vec<String>)
    -> Self{
        

        fill_vec(&mut trust_list, 2);

        let leave_count = BRANCHING_FACTOR.pow(tree_depth);
        let leaves = vec!["0".to_owned(); leave_count];
        
        let initial_tree = HashTree::<H,BRANCHING_FACTOR>::new(&leaves);
        let initial_root = initial_tree.get_root().clone();
        return DefaultLedgerAuthority{
            tree: vec![initial_tree],
            current_tree_index: 0,
            tree_roots: vec![vec![initial_root]],
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
            
            self.tree_roots.push(vec![self.tree[self.current_tree_index].get_root().clone()]);
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
        if !self.trust_list.get_root().eq(&proof.outputs[1]){
            Err("Invalid trust root!")?
        }
        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[0].to_string());
        self.tree_roots[self.current_tree_index].push(self.tree[self.current_tree_index].get_root().clone());
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

        let proof_root = &proof.outputs[2];

        if !self.tree_roots[proof.outputs[1].to_string().parse::<usize>()?].contains(proof_root) {
            Err("Invalid merkle root!")?
        }

        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[3].to_string());
        self.tree_roots[self.current_tree_index].push(self.tree[self.current_tree_index].get_root().clone());
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
    fn transfer_split(&mut self, proof: &ArkCircomFullProof<Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn Error>>{

        if self.nullifier.contains(&proof.outputs[0]){
            Err("Nullified nft!")?
        }

        if proof.verify()? == false {
            Err("Invalid proof!")?
        }

        let proof_root = &proof.outputs[2];

        if !self.tree_roots[proof.outputs[1].to_string().parse::<usize>()?].contains(proof_root) {
            Err("Invalid merkle root!")?
        }

        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[3].to_string());
        self.tree_roots[self.current_tree_index].push(self.tree[self.current_tree_index].get_root().clone());
        let index_a = (self.current_tree_index,self.next_index);
        self.increase_index();

        self.tree[self.current_tree_index].update(self.next_index,proof.outputs[4].to_string());
        self.tree_roots[self.current_tree_index].push(self.tree[self.current_tree_index].get_root().clone());
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
    
    fn transfer_from_nft(&mut self, proof_transfer_child: &ArkCircomFullProof<Bn254>, proofs_child_parent: &Vec<ArkCircomFullProof<Bn254>>)
    -> Result<(usize,usize),Box<dyn Error>> {
        
        if self.nullifier.contains(&proof_transfer_child.outputs[0]){
            Err("Nullified nft!")?
        }

        if proof_transfer_child.verify()? == false {
            Err("Invalid proof!")?
        }
        
        let proof_root_child = &proof_transfer_child.outputs[2];

        if !self.tree_roots[proof_transfer_child.outputs[1].to_string().parse::<usize>()?].contains(proof_root_child) {
            Err("Invalid merkle root!")?
        }

        for i in 0..proofs_child_parent.len(){

            if self.nullifier.contains(&proofs_child_parent[i].outputs[0]){
                        Err("Nullified nft!")?
                }
            
            if self.nullifier.contains(&proofs_child_parent[i].outputs[1]){
                        Err("Nullified nft!")?
                }
                

            if proofs_child_parent[i].verify()? == false {
                Err("Invalid proof!")?
            }

            let proof_root_parent = &proofs_child_parent[i].outputs[2];

            if !self.tree_roots[proofs_child_parent[i].outputs[7].to_string().parse::<usize>()?].contains(proof_root_parent) {
                Err("Invalid merkle root!")?
            }
            let proof_root_child = &proofs_child_parent[i].outputs[3];

            if !self.tree_roots[proofs_child_parent[i].outputs[6].to_string().parse::<usize>()?].contains(proof_root_child) {
                Err("Invalid merkle root!")?
            }

        }

        let mut current_id = proof_transfer_child.outputs[5].clone();
        
        for i in 0..proofs_child_parent.len(){
            println!("Round {i}");
            if proofs_child_parent[i].outputs[5] != current_id {
                Err("Invalid proof chain!")?
            }
            current_id = proofs_child_parent[i].outputs[4].clone();
        }

        if current_id != proof_transfer_child.outputs[3]{
                Err("Invalid proof chain!")?
        }
        
        self.tree[self.current_tree_index].update(self.next_index,proof_transfer_child.outputs[4].to_string());
        self.tree_roots[self.current_tree_index].push(self.tree[self.current_tree_index].get_root().clone());
        let index = (self.current_tree_index,self.next_index);
        self.increase_index();

        self.nullifier.push(proof_transfer_child.outputs[0].clone());

        Ok(index)

    }

    
}


#[cfg(test)]
mod test{
    use super::*;
    use crypto::poseidon::PoseidonHasher;
    use num_traits::Num;



    #[test]
    fn serialize_ledger_authority(){

        let private_key = 
        BigInt::from_str_radix("0001020304050607080900010203040506070809000102030405060708090001",16).unwrap();

        let ledger_authority = 
            DefaultLedgerAuthority::<PoseidonHasher,2>::new(10, vec![private_key.to_string()]);

        let json = serde_json::to_string(&ledger_authority);
    }
}