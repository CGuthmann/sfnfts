use std::{error::Error, io::Write, ops::Mul, str::FromStr};

use crate::{shielded_fractional_nft::witness_generation::{child_parent::proof_nft_child_parent, transfer_child::proof_nft_transfer_child}, FractionalNFT, ShieldedNFT, NFT};

use ark_circom_service::{ArkCircomFullProof, Bn254};
use crypto::{merkle_tree::MerkleProof, poseidon::PoseidonHasher, HashFunction};
use num_bigint::{BigInt, RandBigInt};
use num_traits::FromPrimitive;
use rand::thread_rng;

use serde::{Deserialize, Serialize};

use self::witness_generation::{mint::proof_nft_mint, transfer::proof_nft_transfer, burn::proof_nft_burn};

use ledger_authority::{remote_ledger_authority::RemoteLedgerAuthority, LedgerAuthority};

///Implementation of a shielded fractional NFT
#[derive(Debug, Serialize, Deserialize)]
pub struct ShieldedFractionalNFT {
    pub source_hash: BigInt,
    pub public_key_owner: BigInt,
    pub total_quantity: BigInt,
    pub fraction: [BigInt; 2],
    pub consumed: bool,
    pub is_child: bool,
    pub salt: BigInt,
    pub tree_index: (usize, usize)
}

impl NFT for ShieldedFractionalNFT {
    ///Mints a new ShieldedFractionalNFT
    fn new(
        source: &BigInt,
        value: &BigInt,
        owner: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,
        trust_list_proof: &MerkleProof<PoseidonHasher, 2>,
        private_key: &BigInt,
        is_child: bool
    ) -> Result<Self, Box<dyn Error>> {
        let mut nft = ShieldedFractionalNFT {
            source_hash: source.clone(),
            total_quantity: value.clone(),
            public_key_owner: owner.clone(),
            fraction: [
                BigInt::from_str("1").unwrap(),
                BigInt::from_str("1").unwrap(),
            ],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
            is_child
        };


        std::io::stdout().flush().unwrap();
        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');
        nft.append_flat_json(&mut json, &"".to_owned())?;
        json.push(',');

        
        json.push_str(&format!("\"private_key\":\"{:?}\",", private_key));

        json.push_str(&format!("\"pathTrust\":"));
        append_json_vector(&mut json, &trust_list_proof.path);

        json.push_str(&format!(",\"lemmaTrust\":"));
        append_json_vector(&mut json, &trust_list_proof.lemma);

        json.push('}');


        let proof = proof_nft_mint(&json, None, None)?;


        std::io::stdout().flush().unwrap();
        let index = ledger_authority.mint(&proof)?;

        nft.tree_index = index; //updates the merkle tree index stored in the nft

        Ok(nft)
    }

    

    fn transfer(&self, ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>, target: &BigInt, private_key: &BigInt) -> Result<Self, Box<dyn std::error::Error>> {
        Err("Unsupported operation.")?
    }

    fn burn(
        &mut self,
        private_key: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        

        std::io::stdout().flush().unwrap();
        let merkle_proof =
            ledger_authority.get_tree()[self.tree_index.0].generate_proof(self.tree_index.1)?;

        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');
        self.append_flat_json(&mut json, &"".to_owned())?;

        json.push_str(&format!(",\"private_key\":\"{:?}\",", private_key));

        json.push_str(&format!("\"tree_index\":\"{:?}\",", self.tree_index.0));
        json.push_str(&format!("\"id\":\"{:?}\"", self.tree_index.1));

        json.push_str(&format!(",\"pathNFT\":"));
        append_json_vector(&mut json, &merkle_proof.path);

        json.push_str(&format!(",\"lemmaNFT\":"));
        append_json_vector(&mut json, &merkle_proof.lemma);

        json.push('}');

    

        let proof = proof_nft_burn(&json, None, None)?;



        std::io::stdout().flush().unwrap();
        let result = ledger_authority.burn(&proof)?;
        


        self.consumed = true;
        self.tree_index = result;
        
        
        Ok(())
    }


    fn get_value(&self) -> BigInt {
        (self.total_quantity.clone() * self.fraction[0].clone()) / self.fraction[1].clone()
    }

    fn get_id(&self) -> BigInt {
        let hasher = PoseidonHasher::new();

        hasher.hash_big_int(&[self.source_hash.clone(),self.salt.clone()])
    }
}

impl FractionalNFT for ShieldedFractionalNFT {
    fn transfer_split(
        &self,
        private_key: &BigInt,
        fraction: [BigInt; 2],
        target_a: &BigInt,
        target_b: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,
    ) -> Result<(Self, Self), Box<dyn std::error::Error>> {


        std::io::stdout().flush().unwrap();
        
        let total_nom = fraction[1].clone().mul(self.fraction[0].clone());
        let nom = fraction[0].clone().mul(self.fraction[0].clone());
        let denom = self.fraction[1].clone().mul(fraction[1].clone());


        //Instantiates the new objects

        let mut nft_a = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_a.clone(),
            fraction: [nom.clone(), denom.clone()],
            consumed: false,
            salt: self.salt.clone(),
            tree_index: (0, 0),
            is_child: false
        };
        let mut nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_b.clone(),
            fraction: [total_nom - nom, denom],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
            is_child: false
        };


        std::io::stdout().flush().unwrap();
        let merkle_proof =
            ledger_authority.get_tree()[self.tree_index.0].generate_proof(self.tree_index.1)?;

        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');

        json.push_str(&format!("\"private_key\":\"{:?}\",", private_key));

        json.push_str(&format!(
            "\"fraction\":[\"{:?}\",\"{:?}\"],",
            fraction[0], fraction[1]
        ));

        self.append_flat_json(&mut json, &"base_".to_owned())?;
        json.push(',');

        json.push_str(&format!("\"base_tree_index\":\"{:?}\",", self.tree_index.0));
        json.push_str(&format!("\"base_id\":\"{:?}\",", self.tree_index.1));
        json.push_str(&format!("\"base_path\":"));
        append_json_vector(&mut json, &merkle_proof.path);
        json.push(',');
        json.push_str(&format!("\"base_lemma\":"));
        append_json_vector(&mut json, &merkle_proof.lemma);
        json.push(',');

        nft_a.append_flat_json(&mut json, &"a_".to_owned())?;
        json.push(',');
        json.push_str(&format!("\"a_divider\":\"{:?}\",", 1));

        nft_b.append_flat_json(&mut json, &"b_".to_owned())?;
        json.push(',');
        json.push_str(&format!("\"b_divider\":\"{:?}\"", 1));

        json.push('}');

        let proof = proof_nft_transfer(&json, None, None)?;



        std::io::stdout().flush().unwrap();
        let (index_a, index_b) = ledger_authority.transfer_split(&proof)?;


        nft_a.tree_index = index_a;
        nft_b.tree_index = index_b;

        return Ok((nft_a, nft_b));
    }
}

impl ShieldedNFT for ShieldedFractionalNFT {}

impl ShieldedFractionalNFT {
    //Formates the nft to a flat json object (1 layer, without opening/closing braces)
    fn append_flat_json(&self, json: &mut String, prefix: &String) -> Result<(), Box<dyn Error>> {
        json.push_str(&format!(
            "\"{}source_hash\":\"{:?}\",",
            prefix, self.source_hash
        ));

        json.push_str(&format!(
            "\"{}public_key_owner\":\"{:?}\",",
            prefix, self.public_key_owner
        ));

        json.push_str(&format!(
            "\"{}total_quantity\":\"{:?}\",",
            prefix, self.total_quantity
        ));

        json.push_str(&format!(
            "\"{}fraction\":[\"{:?}\",\"{:?}\"],",
            prefix, self.fraction[0], self.fraction[1]
        ));

        json.push_str(&format!(
            "\"{}consumed\":\"{:?}\",",
            prefix,
            if self.consumed { 1 } else { 0 }
        ));
        
        json.push_str(&format!(
            "\"{}is_child\":\"{:?}\",",
            prefix,
            if self.is_child { 1 } else { 0 }
        ));

        json.push_str(&format!("\"{}salt\":\"{:?}\"", prefix, self.salt));

        Ok(())
    }


  
    ///Mints a new NFT.
    /// Creates the trust_list_proof remote at the ledger_authority.
    /// This reveals the trust_list index to the [RemoteLedgerAuthority]
    /// # Arguments
    /// * `source` - source/type ID
    /// * `value` - stored value in the [NFT]
    /// * `ledger_authority` - [RemoteLedgerAuthority] in which this [NFT] will be created
    /// * `owner` - hashed owner private key
    /// * `trust_list_index` - index of the private key in the [RemoteLedgerAuthority] trust list
    /// * `private_key` - private key corresponding to the trust_list_proof
    ///  
    /// # Return
    /// A minted [NFT]
    pub fn new_remote(
        source: &BigInt,
        value: &BigInt,
        owner: &BigInt,
        ledger_authority: &mut RemoteLedgerAuthority<PoseidonHasher,2>,
        trust_list_index: usize,
        private_key: &BigInt
    ) -> Result<Self, Box<dyn Error>> {
        let mut nft = ShieldedFractionalNFT {
            source_hash: source.clone(),
            total_quantity: value.clone(),
            public_key_owner: owner.clone(),
            fraction: [
                BigInt::from_str("1").unwrap(),
                BigInt::from_str("1").unwrap(),
            ],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
            is_child: true
        };

        let trust_list_proof = ledger_authority.get_trust_proof(trust_list_index)?;

        std::io::stdout().flush().unwrap();
        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');
        nft.append_flat_json(&mut json, &"".to_owned())?;
        json.push(',');

        
        json.push_str(&format!("\"private_key\":\"{:?}\",", private_key));

        json.push_str(&format!("\"pathTrust\":"));
        append_json_vector(&mut json, &trust_list_proof.path);

        json.push_str(&format!(",\"lemmaTrust\":"));
        append_json_vector(&mut json, &trust_list_proof.lemma);

        json.push('}');


        let proof = proof_nft_mint(&json, None, None)?;


        std::io::stdout().flush().unwrap();
        let index = ledger_authority.mint(&proof)?;

        nft.tree_index = index; //updates the merkle tree index stored in the nft

        Ok(nft)
    }
    
    
    ///Burns the NFT
    /// Creates the NFT merkle proof remote at the ledger_authority. 
    /// This reveals the index to the [RemoteLedgerAuthority].
    /// # Arguments
    /// * `private_key` - private key corresponding to hashed in the current [NFT]
    /// * `ledger_authority` - [RemoteLedgerAuthority] in which this [NFT] will be created
    ///  
    /// # Return
    /// The burned [NFT]
    pub fn burn_remote(
        &mut self,
        private_key: &BigInt,
        ledger_authority: &mut RemoteLedgerAuthority<PoseidonHasher,2>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        

        std::io::stdout().flush().unwrap();
        let merkle_proof =
            ledger_authority.get_merkle_proof(self.tree_index)?;

        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');
        self.append_flat_json(&mut json, &"".to_owned())?;

        json.push_str(&format!(",\"private_key\":\"{:?}\",", private_key));

        json.push_str(&format!("\"tree_index\":\"{:?}\",", self.tree_index.0));
        json.push_str(&format!("\"id\":\"{:?}\"", self.tree_index.1));

        json.push_str(&format!(",\"pathNFT\":"));
        append_json_vector(&mut json, &merkle_proof.path);

        json.push_str(&format!(",\"lemmaNFT\":"));
        append_json_vector(&mut json, &merkle_proof.lemma);

        json.push('}');

    

        let proof = proof_nft_burn(&json, None, None)?;



        std::io::stdout().flush().unwrap();
        let result = ledger_authority.burn(&proof)?;
        


        self.consumed = true;
        self.tree_index = result;
        
        
        Ok(())
    }


    /// Transfers the [NFT] to another private key
    /// Creates the NFT merkle proof remote at the ledger_authority. 
    /// This reveals the index to the [RemoteLedgerAuthority].
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
    pub fn transfer_split_remote(
        &self,
        private_key: &BigInt,
        fraction: [BigInt; 2],
        target_a: &BigInt,
        target_b: &BigInt,
        ledger_authority: &mut RemoteLedgerAuthority<PoseidonHasher,2>,
    ) -> Result<(Self, Self), Box<dyn std::error::Error>> {


        std::io::stdout().flush().unwrap();
        
        let total_nom = fraction[1].clone().mul(self.fraction[0].clone());
        let nom = fraction[0].clone().mul(self.fraction[0].clone());
        let denom = self.fraction[1].clone().mul(fraction[1].clone());


        //Instantiates the new objects

        let mut nft_a = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_a.clone(),
            fraction: [nom.clone(), denom.clone()],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
            is_child: false
        };
        let mut nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_b.clone(),
            fraction: [total_nom - nom, denom],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
            is_child: false
        };


        std::io::stdout().flush().unwrap();
        let merkle_proof =
            ledger_authority.get_merkle_proof(self.tree_index)?;

        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');

        json.push_str(&format!("\"private_key\":\"{:?}\",", private_key));

        json.push_str(&format!(
            "\"fraction\":[\"{:?}\",\"{:?}\"],",
            fraction[0], fraction[1]
        ));

        self.append_flat_json(&mut json, &"base_".to_owned())?;
        json.push(',');

        json.push_str(&format!("\"base_tree_index\":\"{:?}\",", self.tree_index.0));
        json.push_str(&format!("\"base_id\":\"{:?}\",", self.tree_index.1));
        json.push_str(&format!("\"base_path\":"));
        append_json_vector(&mut json, &merkle_proof.path);
        json.push(',');
        json.push_str(&format!("\"base_lemma\":"));
        append_json_vector(&mut json, &merkle_proof.lemma);
        json.push(',');

        nft_a.append_flat_json(&mut json, &"a_".to_owned())?;
        json.push(',');
        json.push_str(&format!("\"a_divider\":\"{:?}\",", 1));

        nft_b.append_flat_json(&mut json, &"b_".to_owned())?;
        json.push(',');
        json.push_str(&format!("\"b_divider\":\"{:?}\"", 1));

        json.push('}');

        let proof = proof_nft_transfer(&json, None, None)?;



        std::io::stdout().flush().unwrap();
        let (index_a, index_b) = ledger_authority.transfer_split(&proof)?;


        nft_a.tree_index = index_a;
        nft_b.tree_index = index_b;

        return Ok((nft_a, nft_b));
    }


    pub fn transfer_to_nft(
        &self,
        private_key: &BigInt,
        target: &ShieldedFractionalNFT,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,)
        -> Result<Self, Box<dyn std::error::Error>>{
            
             //Instantiates the new objects

        let mut nft_a = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target.get_id(),
            fraction: [BigInt::from_u8(1).unwrap(), BigInt::from_u8(1).unwrap()],
            consumed: self.consumed,
            salt: self.salt.clone(),
            tree_index: (0, 0),
            is_child: true
        };
        let nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target.get_id(),
            fraction: [BigInt::from_u8(0).unwrap(), BigInt::from_u8(1).unwrap()],
            consumed: self.consumed,
            salt: get_rand_int(),
            tree_index: (0, 0),
            is_child: false
        };


        std::io::stdout().flush().unwrap();
        let merkle_proof =
            ledger_authority.get_tree()[self.tree_index.0].generate_proof(self.tree_index.1)?;

        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');

        json.push_str(&format!("\"private_key\":{:?},", private_key.to_string()));

        json.push_str(&format!(
            "\"fraction\":[\"1\",\"1\"],"
        ));

        self.append_flat_json(&mut json, &"base_".to_owned())?;
        json.push(',');

        json.push_str(&format!("\"base_tree_index\":\"{:?}\",", self.tree_index.0));
        json.push_str(&format!("\"base_id\":\"{:?}\",", self.tree_index.1));
        json.push_str(&format!("\"base_path\":"));
        append_json_vector(&mut json, &merkle_proof.path);
        json.push(',');
        json.push_str(&format!("\"base_lemma\":"));
        append_json_vector(&mut json, &merkle_proof.lemma);
        json.push(',');

        nft_a.append_flat_json(&mut json, &"a_".to_owned())?;
        json.push(',');
        json.push_str(&format!("\"a_divider\":\"{:?}\",", 1));

        nft_b.append_flat_json(&mut json, &"b_".to_owned())?;
        json.push(',');
        json.push_str(&format!("\"b_divider\":\"{:?}\"", 1));

        json.push('}');


        let proof = proof_nft_transfer(&json, None, None)?;

        std::io::stdout().flush().unwrap();
        let (index_a, _) = ledger_authority.transfer_split(&proof)?;
        nft_a.tree_index = index_a;
        Ok(nft_a)
    }

    pub fn transfer_from_nft(
        &self,
        private_key_root: &BigInt,
        target: &BigInt,
        target_is_nft: bool,
        path_nfts: &Vec<&ShieldedFractionalNFT>,    //path[0] is root, last is parent of self
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,)
        -> Result<Self, Box<dyn std::error::Error>>{
            
             //Instantiates the new objects

        let mut nft_a = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target.clone(),
            fraction: self.fraction.clone(),
            consumed: self.consumed,
            salt: self.salt.clone(),
            tree_index: (0, 0),
            is_child: target_is_nft
        };


        std::io::stdout().flush().unwrap();
        let merkle_proof =
            ledger_authority.get_tree()[self.tree_index.0].generate_proof(self.tree_index.1)?;

        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');

        json.push_str(&format!("\"private_key\":\"{}\",", private_key_root.to_string()));

        path_nfts[0].append_flat_json(&mut json, &"root_".to_owned())?;
        json.push(',');
        
        self.append_flat_json(&mut json, &"child_".to_owned())?;
        json.push(',');

        json.push_str(&format!("\"child_tree_index\":\"{:?}\",", self.tree_index.0));
        json.push_str(&format!("\"child_id\":\"{:?}\",", self.tree_index.1));
        json.push_str(&format!("\"child_path\":"));
        append_json_vector(&mut json, &merkle_proof.path);
        json.push(',');
        json.push_str(&format!("\"child_lemma\":"));
        append_json_vector(&mut json, &merkle_proof.lemma);
        json.push(',');

        json.push_str(&format!("\"new_public_key_owner\":\"{:?}\",", target.clone()));
        json.push_str(&format!(
            "\"{}is_child\":\"{:?}\"",
            "new_",
            if self.is_child { 1 } else { 0 }
        ));

        json.push('}');


        let proof_transfer_child = proof_nft_transfer_child(&json, None, None)?;

        let mut proofs_child_parent: Vec<ArkCircomFullProof<Bn254>> = vec![];
        
        for i in 0..path_nfts.len()-1 {

            let merkle_proof_child =
                ledger_authority.get_tree()[path_nfts[i+1].tree_index.0].generate_proof(path_nfts[i+1].tree_index.1)?;
                
            let merkle_proof_parent =
                ledger_authority.get_tree()[path_nfts[i].tree_index.0].generate_proof(path_nfts[i].tree_index.1)?;


            let mut json = String::new();
            json.push('{');
            path_nfts[i].append_flat_json(&mut json, &"parent_".to_owned())?;
            json.push(',');
            json.push_str(&format!("\"parent_id\":\"{:?}\",", path_nfts[i].tree_index.1));
            json.push_str(&format!("\"parent_index\":\"{:?}\",", path_nfts[i].tree_index.0));
            json.push_str(&format!("\"parent_path\":"));
            append_json_vector(&mut json, &merkle_proof_parent.path);
            json.push(',');
            json.push_str(&format!("\"parent_lemma\":"));
            append_json_vector(&mut json, &merkle_proof_parent.lemma);
            json.push(',');


            
            path_nfts[i+1].append_flat_json(&mut json, &"child_".to_owned())?;            
            json.push(',');
            json.push_str(&format!("\"child_id\":\"{:?}\",", path_nfts[i+1].tree_index.1));
            json.push_str(&format!("\"child_index\":\"{:?}\",", path_nfts[i+1].tree_index.0));
            json.push_str(&format!("\"child_path\":"));
            append_json_vector(&mut json, &merkle_proof_child.path);
            json.push(',');
            json.push_str(&format!("\"child_lemma\":"));
            append_json_vector(&mut json, &merkle_proof_child.lemma);

            json.push('}');

            let proof_child_parent = proof_nft_child_parent(&json, None, None)?;
            proofs_child_parent.push(proof_child_parent);
        }

        let index_a = ledger_authority.transfer_from_nft(&proof_transfer_child,&proofs_child_parent)?;
        nft_a.tree_index = index_a;
        Ok(nft_a)
    }
}

///Returns a pseudo random BigInt
fn get_rand_int() -> BigInt {
    thread_rng().gen_bigint_range(
        &BigInt::from(0),
        &BigInt::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617",
        )
        .unwrap(),
    )
}

///Function appending a vector in json format to a string.
fn append_json_vector<T: ToString>(json: &mut String, vec: &[T]) {
    json.push('[');

    for element in vec {
        json.push('"');
        json.push_str(&element.to_string());
        json.push('"');
        json.push(',');
    }
    if vec.len() > 0 {
        json.pop();
    }
    json.push(']');
}


#[cfg(not(feature = "rapid_witnesscalc"))]
mod witness_generation {

    const CIRCUITS_PATH: &str = "/usr/lib/sfnft-libs";

    pub mod mint {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_circuit, ArkCircomFullProof, Bn254};
        
        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_mint(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_mint.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };
    
            let proof = create_proof_from_circuit(
                &format!("{}/{}.wasm",CIRCUITS_PATH,"nft_mint"),
                &format!("{}/{}.r1cs",CIRCUITS_PATH,"nft_mint"),
                json,
                &zkey
            )?;
    
            Ok(proof)
        }
    }

    pub mod burn {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_circuit, ArkCircomFullProof, Bn254};

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_burn(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_burn.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };
    
            let proof = create_proof_from_circuit(
                &format!("{}/{}.wasm",CIRCUITS_PATH,"nft_burn"),
                &format!("{}/{}.r1cs",CIRCUITS_PATH,"nft_burn"),
                json,
                &zkey
            )?;
    
            Ok(proof)
        }
    }

    pub mod transfer {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_circuit, ArkCircomFullProof, Bn254};

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_transfer(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_transfer.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };
            let proof = create_proof_from_circuit(
                &format!("{}/{}.wasm",CIRCUITS_PATH,"nft_transfer"),
                &format!("{}/{}.r1cs",CIRCUITS_PATH,"nft_transfer"),
                json,
                &zkey
            )?;
    
            Ok(proof)
        }
    }

    pub mod transfer_child {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_circuit, ArkCircomFullProof, Bn254};

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_transfer_child(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_transfer_child.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };
    
            let proof = create_proof_from_circuit(
                &format!("{}/{}.wasm",CIRCUITS_PATH,"nft_transfer_child"),
                &format!("{}/{}.r1cs",CIRCUITS_PATH,"nft_transfer_child"),
                json,
                &zkey
            )?;
    
            Ok(proof)
        }
    }

    pub mod child_parent {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_circuit, ArkCircomFullProof, Bn254};

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_child_parent(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_child_parent.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };
    
            let proof = create_proof_from_circuit(
                &format!("{}/{}.wasm",CIRCUITS_PATH,"nft_child_parent"),
                &format!("{}/{}.r1cs",CIRCUITS_PATH,"nft_child_parent"),
                json,
                &zkey
            )?;
    
            Ok(proof)
        }
    }
    ///Attempts to load the specified ressoruce at the default location for the system.
    fn get_ressource(name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match std::fs::read(format!("{}/{}", CIRCUITS_PATH, name)) {
            Ok(res) => Ok(res),
            Err(err) => Err(err)?,
        }
    }
}


#[cfg(feature = "rapid_witnesscalc")]
mod witness_generation {

    const CIRCUITS_PATH: &str = "/usr/lib/sfnft-libs";

    pub mod mint {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_witness, ArkCircomFullProof, Bn254};
        use witness_utils::{create::WITNESS_SIZE_GUESS, read::read_wtns};

        //Link to external witness generation provided by witnesscalc
        #[link(name = "nft_mint", kind = "static")]
        extern "C" {
            fn witnesscalc_nft_mint(
                circuit_buffer: *const u8,
                circuit_size: u64,
                json_buffer: *const u8,
                json_size: u64,
                wtns_buffer: *mut u8,
                wtns_size: *mut u64,
                error_buffer: *mut u8,
                error_msg_maxsize: u64,
            ) -> i32;
        }

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_mint(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {

            let circuit = match circuit {
                Some(res) => res.clone(),
                None => match get_ressource("nft_mint.dat") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let mut wtns: Vec<u8> = Vec::with_capacity(WITNESS_SIZE_GUESS as usize);
            let mut witness_size: u64 = WITNESS_SIZE_GUESS;

            let mut error: Vec<u8> = Vec::with_capacity(200);

            let mut result;
            
            unsafe {
                result = witnesscalc_nft_mint(
                    circuit.as_ptr(),
                    circuit.len() as u64,
                    json.as_ptr(),
                    json.len() as u64,
                    wtns.as_mut_ptr(),
                    &mut witness_size,
                    error.as_mut_ptr(),
                    200,
                );
            }
            
            if result == 1 {
                Err(result.to_string())?
            } else if result == 2 {
                wtns = Vec::with_capacity(witness_size as usize);
                unsafe {
                    result = witnesscalc_nft_mint(
                        circuit.as_ptr(),
                        circuit.len() as u64,
                        json.as_ptr(),
                        json.len() as u64,
                        wtns.as_mut_ptr(),
                        &mut witness_size,
                        error.as_mut_ptr(),
                        200,
                    );
                };

                if result != 0 {
                    Err(result.to_string())?
                }
            }
            unsafe {
                wtns.set_len(witness_size as usize);
            }
            let witness = read_wtns(&wtns);

            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_mint.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let proof = create_proof_from_witness(&witness, &zkey)?;

            Ok(proof)
        }
    }

    pub mod burn {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_witness, ArkCircomFullProof, Bn254};
        use witness_utils::{create::WITNESS_SIZE_GUESS, read::read_wtns};

        //Link to external witness generation provided by witnesscalc
        #[link(name = "nft_burn", kind = "static")]
        extern "C" {
            fn witnesscalc_nft_burn(
                circuit_buffer: *const u8,
                circuit_size: u64,
                json_buffer: *const u8,
                json_size: u64,
                wtns_buffer: *mut u8,
                wtns_size: *mut u64,
                error_buffer: *mut u8,
                error_msg_maxsize: u64,
            ) -> i32;
        }

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_burn(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let circuit = match circuit {
                Some(res) => res.clone(),
                None => match get_ressource("nft_burn.dat") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let mut wtns: Vec<u8> = Vec::with_capacity(WITNESS_SIZE_GUESS as usize);
            let mut witness_size: u64 = WITNESS_SIZE_GUESS;

            let mut error: Vec<u8> = Vec::with_capacity(200);

            let mut result;
            unsafe {
                result = witnesscalc_nft_burn(
                    circuit.as_ptr(),
                    circuit.len() as u64,
                    json.as_ptr(),
                    json.len() as u64,
                    wtns.as_mut_ptr(),
                    &mut witness_size,
                    error.as_mut_ptr(),
                    200,
                );
            }

            if result == 1 {
                Err(result.to_string())?
            } else if result == 2 {
                wtns = Vec::with_capacity(witness_size as usize);
                unsafe {
                    result = witnesscalc_nft_burn(
                        circuit.as_ptr(),
                        circuit.len() as u64,
                        json.as_ptr(),
                        json.len() as u64,
                        wtns.as_mut_ptr(),
                        &mut witness_size,
                        error.as_mut_ptr(),
                        200,
                    );
                };

                if result != 0 {
                    Err(result.to_string())?
                }
            }
            unsafe {
                wtns.set_len(witness_size as usize);
            }
            let witness = read_wtns(&wtns);

            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_burn.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let proof = create_proof_from_witness(&witness, &zkey)?;

            Ok(proof)
        }
    }

    pub mod transfer {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_witness, ArkCircomFullProof, Bn254};
        use witness_utils::{create::WITNESS_SIZE_GUESS, read::read_wtns};

        //Link to external witness generation provided by witnesscalc
        #[link(name = "nft_transfer", kind = "static")]
        extern "C" {
            fn witnesscalc_nft_transfer(
                circuit_buffer: *const u8,
                circuit_size: u64,
                json_buffer: *const u8,
                json_size: u64,
                wtns_buffer: *mut u8,
                wtns_size: *mut u64,
                error_buffer: *mut u8,
                error_msg_maxsize: u64,
            ) -> i32;
        }

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_transfer(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let circuit = match circuit {
                Some(res) => res.clone(),
                None => match get_ressource("nft_transfer.dat") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let mut wtns: Vec<u8> = Vec::with_capacity(WITNESS_SIZE_GUESS as usize);
            let mut witness_size: u64 = WITNESS_SIZE_GUESS;

            let mut error: Vec<u8> = Vec::with_capacity(200);

            let mut result;
            unsafe {
                result = witnesscalc_nft_transfer(
                    circuit.as_ptr(),
                    circuit.len() as u64,
                    json.as_ptr(),
                    json.len() as u64,
                    wtns.as_mut_ptr(),
                    &mut witness_size,
                    error.as_mut_ptr(),
                    200,
                );
            }

            if result == 1 {
                Err(result.to_string())?
            } else if result == 2 {
                wtns = Vec::with_capacity(witness_size as usize);
                unsafe {
                    result = witnesscalc_nft_transfer(
                        circuit.as_ptr(),
                        circuit.len() as u64,
                        json.as_ptr(),
                        json.len() as u64,
                        wtns.as_mut_ptr(),
                        &mut witness_size,
                        error.as_mut_ptr(),
                        200,
                    );
                };

                if result != 0 {
                    Err(result.to_string())?
                }
            }
            unsafe {
                wtns.set_len(witness_size as usize);
            }
            let witness = read_wtns(&wtns);

            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_transfer.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let proof = create_proof_from_witness(&witness, &zkey)?;

            Ok(proof)
        }
    }

     pub mod transfer_child {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_witness, ArkCircomFullProof, Bn254};
        use witness_utils::{create::WITNESS_SIZE_GUESS, read::read_wtns};

        //Link to external witness generation provided by witnesscalc
        #[link(name = "nft_transfer_child", kind = "static")]
        extern "C" {
            fn witnesscalc_nft_transfer_child(
                circuit_buffer: *const u8,
                circuit_size: u64,
                json_buffer: *const u8,
                json_size: u64,
                wtns_buffer: *mut u8,
                wtns_size: *mut u64,
                error_buffer: *mut u8,
                error_msg_maxsize: u64,
            ) -> i32;
        }

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_transfer_child(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let circuit = match circuit {
                Some(res) => res.clone(),
                None => match get_ressource("nft_transfer_child.dat") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let mut wtns: Vec<u8> = Vec::with_capacity(WITNESS_SIZE_GUESS as usize);
            let mut witness_size: u64 = WITNESS_SIZE_GUESS;

            let mut error: Vec<u8> = Vec::with_capacity(200);

            let mut result;
            unsafe {
                result = witnesscalc_nft_transfer_child(
                    circuit.as_ptr(),
                    circuit.len() as u64,
                    json.as_ptr(),
                    json.len() as u64,
                    wtns.as_mut_ptr(),
                    &mut witness_size,
                    error.as_mut_ptr(),
                    200,
                );
            }

            if result == 1 {
                Err(result.to_string())?
            } else if result == 2 {
                wtns = Vec::with_capacity(witness_size as usize);
                unsafe {
                    result = witnesscalc_nft_transfer_child(
                        circuit.as_ptr(),
                        circuit.len() as u64,
                        json.as_ptr(),
                        json.len() as u64,
                        wtns.as_mut_ptr(),
                        &mut witness_size,
                        error.as_mut_ptr(),
                        200,
                    );
                };

                if result != 0 {
                    Err(result.to_string())?
                }
            }
            unsafe {
                wtns.set_len(witness_size as usize);
            }
            let witness = read_wtns(&wtns);

            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("nft_transfer_child.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let proof = create_proof_from_witness(&witness, &zkey)?;

            Ok(proof)
        }
    }

    pub mod child_parent {
        use super::*;

        use std::error::Error;

        use ark_circom_service::{create_proof_from_witness, ArkCircomFullProof, Bn254};
        use witness_utils::{create::WITNESS_SIZE_GUESS, read::read_wtns};

        //Link to external witness generation provided by witnesscalc
        #[link(name = "nft_child_parent", kind = "static")]
        extern "C" {
            fn witnesscalc_nft_child_parent(
                circuit_buffer: *const u8,
                circuit_size: u64,
                json_buffer: *const u8,
                json_size: u64,
                wtns_buffer: *mut u8,
                wtns_size: *mut u64,
                error_buffer: *mut u8,
                error_msg_maxsize: u64,
            ) -> i32;
        }

        ///Generates a witness for the attribute presentation circuit.
        ///
        /// # Arguments
        /// * 'circuit' -  The circuit definition provided by circom in the .dat file.
        ///                 If not provided, the function attempts to read it from
        ///                  the default directory for the system.
        ///
        /// * 'zkey' - The proving key for the circuit.
        ///            If not provided, the function attempts to read it from
        ///               the default directory for the system.
        ///
        pub fn proof_nft_child_parent(
            json: &str,
            circuit: Option<&Vec<u8>>,
            zkey: Option<&Vec<u8>>,
        ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
            let circuit = match circuit {
                Some(res) => res.clone(),
                None => match get_ressource("nft_child_parent.dat") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let mut wtns: Vec<u8> = Vec::with_capacity(WITNESS_SIZE_GUESS as usize);
            let mut witness_size: u64 = WITNESS_SIZE_GUESS;

            let mut error: Vec<u8> = Vec::with_capacity(200);

            let mut result;
            unsafe {
                result = witnesscalc_nft_child_parent(
                    circuit.as_ptr(),
                    circuit.len() as u64,
                    json.as_ptr(),
                    json.len() as u64,
                    wtns.as_mut_ptr(),
                    &mut witness_size,
                    error.as_mut_ptr(),
                    200,
                );
            }

            if result == 1 {
                Err(result.to_string())?
            } else if result == 2 {
                wtns = Vec::with_capacity(witness_size as usize);
                unsafe {
                    result = witnesscalc_nft_child_parent(
                        circuit.as_ptr(),
                        circuit.len() as u64,
                        json.as_ptr(),
                        json.len() as u64,
                        wtns.as_mut_ptr(),
                        &mut witness_size,
                        error.as_mut_ptr(),
                        200,
                    );
                };

                if result != 0 {
                    Err(result.to_string())?
                }
            }
            unsafe {
                wtns.set_len(witness_size as usize);
            }
            let witness = read_wtns(&wtns);

            let zkey = match zkey {
                Some(res) => res.clone(),
                None => match get_ressource("child_parent.zkey") {
                    Ok(res) => res,
                    Err(er) => Err(er)?,
                },
            };

            let proof = create_proof_from_witness(&witness, &zkey)?;

            Ok(proof)
        }
    }

    

    ///Attempts to load the specified ressoruce at the default location for the system.
    fn get_ressource(name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match std::fs::read(format!("{}/{}", CIRCUITS_PATH, name)) {
            Ok(res) => Ok(res),
            Err(err) => Err(err)?,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{io::{stdin, stdout, Read, Write}, str::FromStr, thread::{self, sleep, JoinHandle, Thread}, time::{Duration, Instant}};

    use crate::{shielded_fractional_nft::ShieldedFractionalNFT, *};
    use num_bigint::BigInt;
    use num_traits::Num;
    use ledger_authority::{batched_ledger_authority::BatchedLedgerAuthority, remote_ledger_authority::RemoteLedgerAuthority};

    static mut ledger_authority_batched: Option<BatchedLedgerAuthority<PoseidonHasher, 2>> = None;

    
    const TREE_DEPTH: u32 = 10;
    const TRUST_DEPTH: u32 = 10;

    #[test]
    fn test_nft_local() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        println!("Ledger authority init start");

        let mut trust_list =vec![
            trusted_pub.to_string(),
            "2".to_owned(),
            "3".to_owned(),
            "4".to_owned(),
            "5".to_owned(),
        ];

        for _ in trust_list.len()..((2 as u32).pow(TRUST_DEPTH) as usize) {
            trust_list.push("0".to_owned());
        }

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            TREE_DEPTH,
            trust_list
        );
        println!("Ledger authority init finished");

        //
        // minting
        // 
        println!("Testing minting");

        //default case
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();


        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&trusted_priv, false)
                ;
        let nft = nft_wrap.expect("mint default case");



        //wrong trust list
        let proof = ledger_authority.get_trust_list().generate_proof(1).unwrap();


        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap(), false)
                ;
        
        nft_wrap.expect_err("mint wrong trust list");
        
        //
        // transfer_split
        // 
        println!("Testing transfer");
        
        //wrong pk
        nft
            .transfer_split(
                &owner_b_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .expect_err("transfer wrong pk");

        //default case
        let (mut nft_a,mut nft_b) = nft
            .transfer_split(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .expect("transfer default case");
        
            
        //double spending
        nft
        .transfer_split(
            &owner_a_priv,
            [BigInt::from(1), BigInt::from(2)],
            &owner_a_pub,
            &owner_b_pub,
            &mut ledger_authority,
        )
        .expect_err("transfer double");

        //
        // burn
        // 
        println!("Testing burn");

        //wrong pk
        nft_b.burn(&owner_a_priv,&mut ledger_authority).expect_err("burn wrong pk");

        //default case
        nft_b.burn(&owner_b_priv,&mut ledger_authority).expect("burn default case");

       
        //wrong pk
        nft_b.burn(&owner_a_priv,&mut ledger_authority).expect_err("burn double spending");


        //
        //get_value
        //
        assert_eq!(nft_a.get_value(),BigInt::from_str("60").unwrap());
    }

    
    #[test]
    fn test_nft_child() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        println!("Ledger authority init start");

        let mut trust_list =vec![
            trusted_pub.to_string(),
            "2".to_owned(),
            "3".to_owned(),
            "4".to_owned(),
            "5".to_owned(),
        ];

        for _ in trust_list.len()..((2 as u32).pow(TRUST_DEPTH) as usize) {
            trust_list.push("0".to_owned());
        }

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            TREE_DEPTH,
            trust_list
        );
        println!("Ledger authority init finished");

        //
        // minting
        // 
        println!("Testing minting");

        //default case
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();


        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&trusted_priv, false)
                ;
        let nft = nft_wrap.expect("mint default case");

        let nft_wrap2 =
            ShieldedFractionalNFT::new(&BigInt::from_str("334").unwrap(), &value, &owner_a_pub, &mut ledger_authority, &proof,&trusted_priv, false)
                ;
        let nft_2 = nft_wrap2.expect("mint default case");

         let nft_wrap3 =
            ShieldedFractionalNFT::new(&BigInt::from_str("335").unwrap(), &value, &owner_a_pub, &mut ledger_authority, &proof,&trusted_priv, false)
                ;
        let nft_3 = nft_wrap3.expect("mint default case");

        println!("Testing transfer to nft");

        let nft_child_wrap2 = nft_2.transfer_to_nft(&owner_a_priv, &nft, &mut ledger_authority);
        let nft_child = nft_child_wrap2.expect("transfer to nft");

        let nft_child_wrap3 = nft_3.transfer_to_nft(&owner_a_priv, &nft_child, &mut ledger_authority);
        let nft_child_2 = nft_child_wrap3.expect("transfer to nft");
        

        println!("Testing transfer from nft");
        let nft_wrap3: Result<ShieldedFractionalNFT, Box<dyn Error>> = nft_child_2.transfer_from_nft(&owner_a_priv, &owner_a_pub, false, &vec![&nft,&nft_child], &mut ledger_authority);
        let nft3 = nft_wrap3.expect("Transfer from nft");
    }



    //#[test]
    fn test_nft_remote() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();

        let trusted_priv = BigInt::from_str("1").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        let mut ledger_authority = RemoteLedgerAuthority::new("http://localhost:8000".to_string());

        println!("Ledger authority init finished");

        //
        // minting
        // 
        println!("Testing minting");

        //default case
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();


        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&trusted_priv, false)
                ;
        let nft = nft_wrap.expect("mint default case");


        //wrong trust list
        let proof = ledger_authority.get_trust_list().generate_proof(1).unwrap();


        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap(), false)
                ;
        
        nft_wrap.expect_err("mint wrong trust list");
        

        
        //default remote case
        let nft_wrap2 =
            ShieldedFractionalNFT::new_remote(&source, &value, &owner_a_pub, &mut ledger_authority, 0,&trusted_priv)
                ;
        let nft2 = nft_wrap2.expect("mint default remote case");

        //wrong trust list remote
        let nft_wrap2 =
        ShieldedFractionalNFT::new_remote(&source, &value, &owner_a_pub, &mut ledger_authority, 1,&trusted_priv);
        
        nft_wrap2.expect_err("mint wrong trust list remote");


        sleep(Duration::from_secs(15)); //waiting for tree update

        //
        // transfer_split
        // 
        println!("Testing transfer");
        

        //wrong pk
        nft
            .transfer_split(
                &owner_b_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .expect_err("transfer wrong pk");

        //default case
        let (mut nft_a,mut nft_b) = nft
            .transfer_split(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .expect("transfer default case");
        
            
        //double spending
        nft
        .transfer_split(
            &owner_a_priv,
            [BigInt::from(1), BigInt::from(2)],
            &owner_a_pub,
            &owner_b_pub,
            &mut ledger_authority,
        )
        .expect_err("transfer double");

    
        
        //wrong pk remote
        nft2
            .transfer_split_remote(
                &owner_b_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .expect_err("transfer wrong pk remotey");

        //remote case
        let (mut nft2_a,mut nft2_b) = nft2
            .transfer_split_remote(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .expect("transfer remote case");
        
            
        //double spending remote
        nft2
        .transfer_split_remote(
            &owner_a_priv,
            [BigInt::from(1), BigInt::from(2)],
            &owner_a_pub,
            &owner_b_pub,
            &mut ledger_authority,
        )
        .expect_err("transfer double remote");


        sleep(Duration::from_secs(15)); //waiting for tree update
        //
        // burn
        // 
        println!("Testing burn");

        //wrong pk
        nft_b.burn(&owner_a_priv,&mut ledger_authority).expect_err("burn wrong pk");

        //default case
        nft_b.burn(&owner_b_priv,&mut ledger_authority).expect("burn default case");

       
        //double burning
        nft_b.burn(&owner_b_priv,&mut ledger_authority).expect_err("burn double spending remote");

        

        //wrong pk remote
        nft2_b.burn_remote(&owner_a_priv,&mut ledger_authority).expect_err("burn wrong pk");

        //default case remote
        nft2_b.burn_remote(&owner_b_priv,&mut ledger_authority).expect("burn default case");

       
        //double burning remote
        nft2_b.burn_remote(&owner_b_priv,&mut ledger_authority).expect_err("burn double spending remote");

    }



}
