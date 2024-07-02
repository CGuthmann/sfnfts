use std::{iter::Repeat, marker::PhantomData};

use crypto::{merkle_tree::{fill_vec, HashTree, MerkleProof}, poseidon::PoseidonHasher, HashFunction};
use num_bigint::BigInt;

use crate::LedgerAuthority;

pub struct RemoteLedgerAuthority<H: HashFunction, const BRANCHING_FACTOR: usize>{
    url: String,
    pub tree: Vec<HashTree<H,BRANCHING_FACTOR>>,
    pub current_tree_index: usize,
    pub next_index: usize,
    pub tree_depth: u32,
    pub trust_list: HashTree<H,2>,
    pub nullifier: Vec<BigInt>
}

impl<H: HashFunction, const BRANCHING_FACTOR: usize> RemoteLedgerAuthority<H,BRANCHING_FACTOR>{
    pub fn new(url: String)-> Self{
        RemoteLedgerAuthority{ 
            url, 
            tree: Vec::<HashTree<H,BRANCHING_FACTOR>>::new(),
            current_tree_index: 0,
            next_index: 0, 
            tree_depth: 0, 
            trust_list: HashTree::new(&vec!["".to_owned()]), 
            nullifier: vec![]}
    }
}

impl<H: HashFunction, const BRANCHING_FACTOR: usize> LedgerAuthority<H,BRANCHING_FACTOR> for RemoteLedgerAuthority<H,BRANCHING_FACTOR>{
    

    fn mint(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<(usize,usize),Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client.post(self.url.to_owned() + "/mint").json(proof).send()?;
        Ok(resp.json()?)
    }

    fn transfer(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client.post(self.url.to_owned() + "/transfer").json(proof).send()?;
        Ok(resp.json()?)
    }

    fn get_tree(&self) -> Vec<crypto::merkle_tree::HashTree<H,BRANCHING_FACTOR>> {
        let resp = reqwest::blocking::get(self.url.to_owned() + "/merkle_tree").unwrap();
        resp.json().unwrap()
    }

    fn get_trust_list(&self)-> crypto::merkle_tree::HashTree<H,2> {
        let resp = reqwest::blocking::get(self.url.to_owned() + "/trust_list").unwrap();
        resp.json().unwrap()
    }

    fn get_nullifier(&self) -> Vec<num_bigint::BigInt> {
        let resp = reqwest::blocking::get(self.url.to_owned() + "/merkle_tree").unwrap();
        resp.json().unwrap()
    }

    fn burn(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<(usize,usize),Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client.post(self.url.to_owned() + "/burn").json(proof).send()?;
        
        Ok(resp.json()?)
    }


}


impl<H: HashFunction, const BRANCHING_FACTOR: usize> RemoteLedgerAuthority<H,BRANCHING_FACTOR>{
    
    pub fn get_merkle_proof(&mut self, index: (usize,usize))
    -> Result<MerkleProof<PoseidonHasher,2>,Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(self.url.to_owned() + "/get_merkle_proof").json(&index).send()?;
        Ok(resp.json()?)
    }
    
    pub fn get_trust_proof(&mut self, index: usize)
    -> Result<MerkleProof<PoseidonHasher,2>,Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(self.url.to_owned() + "/get_trust_proof").json(&index).send()?;
        Ok(resp.json()?)
    }
}