use std::cmp::min;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use std::{iter::Repeat, marker::PhantomData};
use std::error::Error;


use crypto::{merkle_tree::{fill_vec, HashTree}, poseidon::PoseidonHasher, HashFunction};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use crate::LedgerAuthority;

static mut UNUSED_COUNTER: u32 = 0;

#[derive(Debug,Deserialize,Serialize)]
struct BatchedLedgerAuthorityInner<H: HashFunction, const BRANCHING_FACTOR: usize>{    
    #[serde(bound(
        serialize = "HashTree<H,2>: Serialize",
        deserialize = "HashTree<H,2>: Deserialize<'de>"
    )
    )]
    pub tree: Vec<HashTree<H,BRANCHING_FACTOR>>,

    #[serde(with = "vectorize")]
    pub tree_roots: HashMap<BigInt,bool>,
    pub current_tree_index: usize,
    pub next_index: usize,
    pub tree_depth: u32,

    #[serde(bound(
        serialize = "HashTree<H,2>: Serialize",
        deserialize = "HashTree<H,2>: Deserialize<'de>"
    ))]
    pub trust_list: HashTree<H,2>,
    pub nullifier: Vec<BigInt>,
    leave_count: usize,

    update_tree_index: usize,
    update_next_index: usize,
    update_leaves: Vec<String>,
    duration_mint: Vec<Vec<u32>>,
    duration_transfer: Vec<Vec<u32>>,
    duration_burn: Vec<Vec<u32>>,
}

pub mod vectorize {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::iter::FromIterator;

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        let container: Vec<_> = target.into_iter().collect();
        serde::Serialize::serialize(&container, ser)
    }

    pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        let container: Vec<_> = serde::Deserialize::deserialize(des)?;
        Ok(T::from_iter(container.into_iter()))
    }
}

#[derive(Debug,Deserialize,Serialize)]
pub struct BatchedLedgerAuthority<H: HashFunction, const BRANCHING_FACTOR: usize>{    
    #[serde(bound(
        serialize = "BatchedLedgerAuthorityInner<H,BRANCHING_FACTOR>: Serialize",
        deserialize = "BatchedLedgerAuthorityInner<H,BRANCHING_FACTOR>: Deserialize<'de>"
    ))]
    inner: Mutex<BatchedLedgerAuthorityInner<H,BRANCHING_FACTOR>>,
    pub intervall_millis: usize
}


impl<H: HashFunction> LedgerAuthority<H,2> for BatchedLedgerAuthorityInner<H,2>{
    fn mint(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<(usize,usize),Box<dyn std::error::Error>> {

        let mut start = Instant::now();
        if proof.verify()? == false {
            Err("Invalid proof!")?
        }
        self.duration_mint[1].push(start.elapsed().as_nanos() as u32);

        
        let mut start = Instant::now();
        if !self.trust_list.get_root().eq(&proof.outputs[1]){
            Err("Invalid trust root!")?
        }
        self.duration_mint[2].push(start.elapsed().as_nanos() as u32);

        start = Instant::now();
        let index = (self.update_tree_index,self.update_next_index);
        self.increase_index();

        self.update_leaves.push(proof.outputs[0].to_string());
        self.duration_mint[3].push(start.elapsed().as_nanos() as u32);

        Ok(index)
    }

    fn transfer(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn std::error::Error>> {
        
        let mut start = Instant::now();
        if self.nullifier.contains(&proof.outputs[0]){
            Err("Nullified nft!")?
        }
        self.duration_transfer[0].push(start.elapsed().as_nanos() as u32);

        start = Instant::now();
        if proof.verify()? == false {
            Err("Invalid proof!")?
        }
        self.duration_transfer[1].push(start.elapsed().as_nanos() as u32);

        start = Instant::now();
        if !self.tree_roots[&proof.outputs[2]]{
            Err("Invalid merkle root!")?
        }
        self.duration_transfer[2].push(start.elapsed().as_nanos() as u32);


        start = Instant::now();
        self.nullifier.push(proof.outputs[0].clone());

        let index_a = (self.update_tree_index,self.update_next_index);
        self.increase_index();
        self.update_leaves.push(proof.outputs[3].to_string());

        let index_b = (self.update_tree_index,self.update_next_index);
        self.increase_index();
        self.update_leaves.push(proof.outputs[4].to_string());

        self.duration_transfer[3].push(start.elapsed().as_nanos() as u32);


        Ok((index_a,index_b))     
    }

    fn burn(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<(usize,usize),Box<dyn std::error::Error>> {
       
        let mut start = Instant::now();
        if self.nullifier.contains(&proof.outputs[0]){
            Err("Nullified nft!")?
        }
        self.duration_burn[0].push(start.elapsed().as_nanos() as u32);

        start = Instant::now();
        if proof.verify()? == false {
            Err("Invalid proof!")?
        }
        self.duration_burn[1].push(start.elapsed().as_nanos() as u32);

        start = Instant::now();
        if !self.tree_roots[&proof.outputs[2]]{
            Err("Invalid merkle root!")?
        }
        self.duration_burn[2].push(start.elapsed().as_nanos() as u32);

        
        start = Instant::now();
        self.update_leaves.push(proof.outputs[3].to_string());

        let index = (self.update_tree_index,self.update_next_index);
        self.increase_index();
        self.nullifier.push(proof.outputs[0].clone());
        self.duration_burn[3].push(start.elapsed().as_nanos() as u32);

        

        Ok(index)
    }

    fn get_tree(&self) -> Vec<HashTree<H,2>>{
        self.tree.clone()
    }
    
    fn get_trust_list(&self)-> HashTree<H,2>{
        self.trust_list.clone()
    }

    fn get_nullifier(&self) -> Vec<BigInt>{
        self.nullifier.clone()
    }
}


impl<H: HashFunction> BatchedLedgerAuthorityInner<H, 2> {
    
    pub fn new(tree_depth: u32, mut trust_list: Vec<String>)-> Self{
        

        fill_vec(&mut trust_list, 2);

        let leave_count = (2 as u32).pow(tree_depth) as usize;
        let leaves = vec!["0".to_owned(); leave_count];
        let initial_tree = HashTree::<H,2>::new(&leaves);
        let initial_root = initial_tree.get_root().clone();

        BatchedLedgerAuthorityInner{
            tree: vec![initial_tree],
            current_tree_index: 0,
            tree_roots: HashMap::from([(initial_root,true)]),
            next_index: 0,
            tree_depth,
            trust_list: HashTree::<H,2>::new(&trust_list),
            nullifier: Vec::new(),
            leave_count,
            update_tree_index: 0,
            update_next_index: 0,
            update_leaves: Vec::new(),
            duration_mint: vec![Vec::new();4]   ,
            duration_transfer: vec![Vec::new();4]    ,
            duration_burn: vec![Vec::new();4]   ,         
        }
    }
    pub fn update(&mut self){

        if self.update_leaves.is_empty(){
            return;
        }

        let first_batch = min(self.leave_count-self.next_index, self.update_leaves.len());
        self.tree[self.current_tree_index].update_batch(self.next_index, &self.update_leaves[0..first_batch].to_vec());
        self.tree_roots.insert(self.tree[self.current_tree_index].get_root().clone(), true);
        let mut current_update_index = first_batch;

        //generates new hashtrees and insertes the leaves
        for _ in 1..self.update_tree_index-self.current_tree_index{
            self.tree.push(
                HashTree::<H,2>::new(&self.update_leaves[current_update_index..current_update_index+self.leave_count].to_vec())
            );
            self.current_tree_index = self.current_tree_index+1;
            self.tree_roots.insert(self.tree[self.current_tree_index].get_root().clone(), true);
            current_update_index = current_update_index + self.leave_count;
        }
        
        //updates the leaves of the new hashtrees
        if current_update_index < self.update_leaves.len() {
            let mut append_vec = vec!["0".to_owned(); self.leave_count - (self.update_leaves.len() - current_update_index)];
            self.update_leaves.append(&mut append_vec);
            self.tree.push(
                HashTree::<H,2>::new(&self.update_leaves[current_update_index..current_update_index+self.leave_count].to_vec())
            );
            self.current_tree_index = self.current_tree_index+1;
            self.tree_roots.insert(self.tree[self.current_tree_index].get_root().clone(), true);
            current_update_index = current_update_index + self.leave_count;
        }

        self.next_index = self.update_next_index;
        self.update_leaves.clear();
        
    }

    

    fn increase_index(&mut self){

        self.update_next_index += 1;

        if self.update_next_index >= self.leave_count {
            self.update_tree_index +=1;
            self.update_next_index = 0;                       
        }

    }      
    
    pub fn transfer_bench(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn std::error::Error>> {
        
        if self.nullifier.contains(&proof.outputs[0]){
            unsafe { UNUSED_COUNTER = UNUSED_COUNTER +1 };
        }

        if proof.verify()? == false {
            Err("Invalid proof!")?
        }

        if !self.tree_roots[&proof.outputs[2]]{
            Err("Invalid merkle root!")?
        }

        self.nullifier.push(proof.outputs[0].clone());

        let index_a = (self.update_tree_index,self.update_next_index);
        self.increase_index();
        self.update_leaves.push(proof.outputs[3].to_string());

        let index_b = (self.update_tree_index,self.update_next_index);
        self.increase_index();
        self.update_leaves.push(proof.outputs[4].to_string());


        Ok((index_a,index_b))  
    }
    
    pub fn get_duration(&mut self) -> Result<Vec<Vec<u128>>,Box<dyn std::error::Error>> {

        let mut nullifier_duration_mint = 0 as u128;
        if self.duration_mint[0].len() > 0 {
            nullifier_duration_mint = self.duration_mint[0].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_mint[0].len() as u128);
        }
        
        let mut zkp_duration_mint = 0 as u128;
        if self.duration_mint[1].len() > 0 {
            zkp_duration_mint = self.duration_mint[1].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_mint[1].len() as u128);
        }

        let mut validating_duration_mint = 0 as u128;
        if self.duration_mint[2].len() > 0 {
            validating_duration_mint = self.duration_mint[2].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_mint[2].len() as u128);
        }

        let mut registering_duration_mint = 0 as u128;
        if self.duration_mint[3].len() > 0 {
            registering_duration_mint = self.duration_mint[3].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_mint[3].len() as u128);
        }

        
        let mint_durations = vec![nullifier_duration_mint,zkp_duration_mint,validating_duration_mint,registering_duration_mint];


        
        let mut nullifier_duration_transfer = 0 as u128;
        if self.duration_transfer[0].len() > 0 {
            nullifier_duration_transfer = self.duration_transfer[0].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_transfer[0].len() as u128);
        }
        
        let mut zkp_duration_transfer = 0 as u128;
        if self.duration_transfer[1].len() > 0 {
            zkp_duration_transfer = self.duration_transfer[1].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_transfer[1].len() as u128);
        }

        let mut validating_duration_transfer = 0 as u128;
        if self.duration_transfer[2].len() > 0 {
            validating_duration_transfer = self.duration_transfer[2].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_transfer[2].len() as u128);
        }

        let mut registering_duration_transfer = 0 as u128;
        if self.duration_transfer[3].len() > 0 {
            registering_duration_transfer = self.duration_transfer[3].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_transfer[3].len() as u128);
        }

        
        let transfer_durations = vec![nullifier_duration_transfer,zkp_duration_transfer,validating_duration_transfer,registering_duration_transfer];


        
        let mut nullifier_duration_burn = 0 as u128;
        if self.duration_burn[0].len() > 0 {
            nullifier_duration_burn = self.duration_burn[0].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_burn[0].len() as u128);
        }
        
        let mut zkp_duration_burn = 0 as u128;
        if self.duration_burn[1].len() > 0 {
            zkp_duration_burn = self.duration_burn[1].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_burn[1].len() as u128);
        }

        let mut validating_duration_burn = 0 as u128;
        if self.duration_burn[2].len() > 0 {
            validating_duration_burn = self.duration_burn[2].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_burn[2].len() as u128);
        }

        let mut registering_duration_burn = 0 as u128;
        if self.duration_burn[3].len() > 0 {
            registering_duration_burn = self.duration_burn[3].iter().fold(0u128, |sum, i| sum + (*i as u128))/(self.duration_burn[3].len() as u128);
        }

        
        let burn_durations = vec![nullifier_duration_burn,zkp_duration_burn,validating_duration_burn,registering_duration_burn];

        

        Ok(vec![mint_durations, transfer_durations,burn_durations])
    }

}


impl<H: HashFunction> LedgerAuthority<H,2> for BatchedLedgerAuthority<H,2>{
    fn mint(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<(usize,usize),Box<dyn Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.mint(&proof)
    }

    fn transfer(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.transfer(&proof)
    }

    fn burn(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<(usize,usize),Box<dyn Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.burn(&proof)
    }

    fn get_tree(&self) -> Vec<HashTree<H,2>> {
        let inner = self.inner.lock().unwrap();
        inner.get_tree()
    }

    fn get_trust_list(&self)-> HashTree<H,2> {
        let inner = self.inner.lock().unwrap();
        inner.get_trust_list()
    }

    fn get_nullifier(&self) -> Vec<BigInt> {
        let inner = self.inner.lock().unwrap();
        inner.get_nullifier()
    }
}

impl<H: HashFunction> BatchedLedgerAuthority<H, 2>{
    pub fn new(tree_depth: u32, mut trust_list: Vec<String>,intervall_millis: usize)-> Self{
        BatchedLedgerAuthority{
            inner: Mutex::new(BatchedLedgerAuthorityInner::<H,2>::new(tree_depth, trust_list)),
            intervall_millis
        }
    }

    
    pub fn init(&'static self){           
        
        thread::spawn(move || {
            loop{
            sleep(Duration::from_millis(self.intervall_millis as u64));
            let mut la = self.inner.lock().unwrap();
            la.update();
            }
        });
    }

    pub fn update(&mut self){
        let mut inner = self.inner.lock().unwrap();
        inner.update();
    }

    pub fn transfer_bench(&mut self, proof: &ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>)
    -> Result<((usize,usize),(usize,usize)),Box<dyn std::error::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.transfer_bench(proof)
        
    }

    pub fn get_duration(&mut self)
    -> Result<Vec<Vec<u128>>,Box<dyn std::error::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.get_duration()        
    }
}