use std::{error::Error, io::Write, ops::Mul, str::FromStr};

use crate::{FractionalNFT, ShieldedNFT, NFT};

use ark_circom_service::{ArkCircomFullProof, Bn254};
use crypto::{merkle_tree::MerkleProof, poseidon::PoseidonHasher};
use num_bigint::{BigInt, RandBigInt};
use rand::thread_rng;

use serde::{Deserialize, Serialize};

use self::witness_generation::{mint::proof_nft_mint, transfer::proof_nft_transfer, burn::proof_nft_burn};

use ledger_authority::{remote_ledger_authority::RemoteLedgerAuthority, LedgerAuthority};

///Implementation of a shielded fractional NFT
#[derive(Debug, Serialize, Deserialize)]
pub struct ShieldedFractionalNFT {
    source_hash: BigInt,
    public_key_owner: BigInt,
    total_quantity: BigInt,
    fraction: [BigInt; 2],
    consumed: bool,
    salt: BigInt,
    tree_index: (usize, usize),
}

impl NFT for ShieldedFractionalNFT {
    ///Mints a new ShieldedFractionalNFT
    fn new(
        source: &BigInt,
        value: &BigInt,
        owner: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,
        trust_list_proof: &MerkleProof<PoseidonHasher, 2>,
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

    

    fn transfer(&self, ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>, target: &BigInt) -> Result<Self, Box<dyn std::error::Error>> {
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
        self.total_quantity.clone()
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
            salt: get_rand_int(),
            tree_index: (0, 0),
        };
        let mut nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_b.clone(),
            fraction: [total_nom - nom, denom],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
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
        let (index_a, index_b) = ledger_authority.transfer(&proof)?;


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

        json.push_str(&format!("\"{}salt\":\"{:?}\"", prefix, self.salt));

        Ok(())
    }

     ///Mints a new ShieldedFractionalNFT
     pub fn new_bench(
        source: &BigInt,
        value: &BigInt,
        owner: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,
        trust_list_proof: &MerkleProof<PoseidonHasher, 2>,
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
        //let index = ledger_authority.mint(&proof)?;

        //nft.tree_index = index; //updates the merkle tree index stored in the nft

        Ok(nft)
    }
    
    

    ///Mints a new ShieldedFractionalNFT
    pub fn new_bench_ledger(
        source: &BigInt,
        value: &BigInt,
        owner: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,
        trust_list_proof: &MerkleProof<PoseidonHasher, 2>,
        private_key: &BigInt
    ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn Error>> {
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
        //let index = ledger_authority.mint(&proof)?;

        //nft.tree_index = index; //updates the merkle tree index stored in the nft

        Ok(proof)
    }

    

    pub fn transfer_split_bench(
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
            salt: get_rand_int(),
            tree_index: (0, 0),
        };
        let mut nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_b.clone(),
            fraction: [total_nom - nom, denom],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
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



        //let (index_a, index_b) = ledger_authority.transfer(&proof)?;


        //nft_a.tree_index = index_a;
        //nft_b.tree_index = index_b;

        return Ok((nft_a, nft_b));
    }

    pub fn transfer_split_bench_ledger(
        &self,
        private_key: &BigInt,
        fraction: [BigInt; 2],
        target_a: &BigInt,
        target_b: &BigInt,
        ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>,
    ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn std::error::Error>> {


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
        };
        let mut nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_b.clone(),
            fraction: [total_nom - nom, denom],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
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



        //let (index_a, index_b) = ledger_authority.transfer(&proof)?;


        //nft_a.tree_index = index_a;
        //nft_b.tree_index = index_b;

        return Ok(proof);
    }

    
    pub fn transfer_split_bench_ledger_remote(
        &self,
        private_key: &BigInt,
        fraction: [BigInt; 2],
        target_a: &BigInt,
        target_b: &BigInt,
        ledger_authority: &mut RemoteLedgerAuthority<PoseidonHasher,2>,
    ) -> Result<ArkCircomFullProof<Bn254>, Box<dyn std::error::Error>> {


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
        };
        let mut nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_b.clone(),
            fraction: [total_nom - nom, denom],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
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



        //let (index_a, index_b) = ledger_authority.transfer(&proof)?;


        //nft_a.tree_index = index_a;
        //nft_b.tree_index = index_b;

        return Ok(proof);
    }

    
    
    pub fn burn_bench(
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
        //let result = ledger_authority.burn(&proof)?;
    
        
        
        Ok(())
    }





  ///Mints a new ShieldedFractionalNFT
    pub fn new_remote(
        source: &BigInt,
        value: &BigInt,
        owner: &BigInt,
        ledger_authority: &mut RemoteLedgerAuthority<PoseidonHasher,2>,
        trust_list_proof: &MerkleProof<PoseidonHasher, 2>,
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

    

    fn transfer(&self, ledger_authority: &mut dyn LedgerAuthority<PoseidonHasher,2>, target: &BigInt) -> Result<Self, Box<dyn std::error::Error>> {
        Err("Unsupported operation.")?
    }

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
        };
        let mut nft_b = ShieldedFractionalNFT {
            source_hash: self.source_hash.clone(),
            total_quantity: self.total_quantity.clone(),
            public_key_owner: target_b.clone(),
            fraction: [total_nom - nom, denom],
            consumed: false,
            salt: get_rand_int(),
            tree_index: (0, 0),
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
        let (index_a, index_b) = ledger_authority.transfer(&proof)?;


        nft_a.tree_index = index_a;
        nft_b.tree_index = index_b;

        return Ok((nft_a, nft_b));
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
mod witness_generation {

    const CIRCUITS_PATH: &str = "lib";

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

    
    const TREE_DEPTH: u32 = 22;
    const TRUST_DEPTH: u32 = 20;

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

        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();

        println!("Membership proof done!");

        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;
        let nft = nft_wrap.unwrap();
        println!("Nft minted");
        let (mut nft_a,mut nft_b) = nft
            .transfer_split(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .unwrap();
        

            match nft_b.burn(&owner_b_priv,&mut ledger_authority){
                Ok(_) => (),
                Err(err) => panic!("{}", err.to_string()),
            }



    }

    #[test]
    fn test_nft_remote() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();

        let trusted_priv = BigInt::from_str("1").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        let mut ledger_authority = RemoteLedgerAuthority::new("http://192.168.178.87:8000".to_string());

        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();



        let nft = match
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&trusted_priv)
                {
                    Ok(res) => res,
                    Err(err) => {

                        panic!("Error")
                    },
                };
                
        sleep(Duration::from_millis(12000));

        let (mut nft_a,mut nft_b) = nft
            .transfer_split(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .unwrap();
        
        sleep(Duration::from_millis(12000));

        match nft_b.burn(&owner_b_priv,&mut ledger_authority){
            Ok(_) => (),
            Err(err) => panic!("{}", err.to_string()),
        }
        



    }


    #[test]
    fn test_nft_batched_manuell() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3
        dbg!("LA init ");
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
        unsafe { ledger_authority_batched = Some(BatchedLedgerAuthority::new(
            TREE_DEPTH,
            trust_list,
            10000
        )) };

        
        match unsafe { ledger_authority_batched.as_mut() } {
            Some(ledger_authority) => {
                
        
        dbg!("Gen trust lsit proof");
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();


        dbg!("Gen new NFT");
        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft = nft_wrap.unwrap();
        dbg!("Root {}", ledger_authority.get_tree()[0].get_root());
        
        ledger_authority.update();
        
        dbg!("Root {}", ledger_authority.get_tree()[0].get_root());
        let (mut nft_a,mut nft_b) = nft
            .transfer_split(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                ledger_authority,
            )
            .unwrap();

        //println!("\n\n\n\n{}\n\n", ledger_authority.get_tree()[0].to_string());
    
        ledger_authority.update();

        
        //println!("\n\n{}\n\n\n\n", ledger_authority.get_tree()[0].to_string());

        match nft_b.burn(&owner_b_priv,ledger_authority){
            Ok(_) => (),
            Err(err) => panic!("{}", err.to_string()),
        }


            },
            None => todo!(),
        }

    }

    #[test]
    fn test_nft_batched_intervall() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3
        dbg!("LA init "); 
        
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
        unsafe { ledger_authority_batched = Some(BatchedLedgerAuthority::new(
            TREE_DEPTH,
            trust_list,
            10000
        )) };

        unsafe { ledger_authority_batched.as_mut().unwrap().init() };
        
        match unsafe { ledger_authority_batched.as_mut() } {
            Some(ledger_authority) => {
                

        dbg!("Gen trust lsit proof");
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();


        dbg!("Gen new NFT");
        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft = nft_wrap.unwrap();
        dbg!("Root {}", ledger_authority.get_tree()[0].get_root());
        
        sleep(Duration::from_millis(12000));
        
        dbg!("Root {}", ledger_authority.get_tree()[0].get_root());
        let (mut nft_a,mut nft_b) = nft
            .transfer_split(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                ledger_authority,
            )
            .unwrap();

        println!("\n\n\n\n{}\n\n", ledger_authority.get_tree()[0].to_string());
    
        sleep(Duration::from_millis(12000));
        
        println!("\n\n{}\n\n\n\n", ledger_authority.get_tree()[0].to_string());

        match nft_b.burn(&owner_b_priv,ledger_authority){
            Ok(_) => (),
            Err(err) => panic!("{}", err.to_string()),
        }


            },
            None => todo!(),
        }

    }


    #[test]
    fn example_nft_backend(){

        println!("Step 0: Initializing public/private key pairs for participants");
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();

        let trusted_priv = BigInt::from_str("1").unwrap();
        let _trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();
        println!("Bundesumweltamt:");
        println!("\tprivate key:\t {}", trusted_priv);
        println!("\tpublic key:\t {}", _trusted_pub);
        println!("");

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2
        println!("PV owner Alice:");
        println!("\tprivate key:\t {}", owner_a_priv);
        println!("\tpublic key:\t {}", owner_a_pub);
        println!("");

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3
        println!("Chocolate manufacturer Bob:");
        println!("\tprivate key:\t {}", owner_b_priv);
        println!("\tpublic key:\t {}", owner_b_pub);
        println!("");
        pause();

        println!("Step 1a: Initializing remote ledeger authority\n");
        let mut ledger_authority = RemoteLedgerAuthority::new("http://192.168.178.87:8000".to_string());
        pause();

        println!("Step 1b: Initializing connection to the remote ledger authority:");
        println!("done!\n");
        pause();


        println!("Step 2: Bundesumweltamt mints a new NFT for Alice, proofing the generation of 120 kWh green energy:\n");

        print!("Generating membership proof of the Bundesumweltamt in the trust list...\t\t");
        std::io::stdout().flush().unwrap();
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();
        println!("done!");
        println!("Merkle proof: {}\n", serde_json::to_string(&proof).unwrap());
        
        let nft = match
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&trusted_priv)
                {
                    Ok(res) => res,
                    Err(err) => {

                        panic!("Error")
                    },
                };

        println!("Ledger authority accepted the minting zkp, NFT is now valid.");
        println!("Alice's NFT:");
        println!("{:?}\n\n", nft);
        pause();

        
        println!("Step 3: Alice sells 60 kWh of green energy to Bob");

        let (mut nft_a,mut nft_b) = nft
            .transfer_split(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .unwrap();
        println!("Ledger authority accepted the transfer zkp, the two resulting NFTs are now valid.");
        println!("Alice's new NFT:");
        println!("{:?}\n\n", nft_a);

        println!("Bob's NFT:");
        println!("{:?}\n\n", nft_b);
        pause();         

        println!("Step 4: Bob uses the energy to manifacture green chocolate:");
        match nft_b.burn(&owner_b_priv,&mut ledger_authority){
            Ok(_) => (),
            Err(err) => panic!("{}", err.to_string()),
        }
        println!("Bob's burned NFT:");
        println!("{:?}\n\n", nft_b);

        


    }

    fn pause(){
        let mut stdout = stdout();
        stdout.write(b"Press Enter to continue...\n").unwrap();
        stdout.flush().unwrap();
        stdin().read(&mut [0]).unwrap();
    }

    
    #[test]
    pub fn nft_throughput(){
        println!("Bench init");
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        let mut ledger_authority = RemoteLedgerAuthority::new("http://192.168.178.87:8000".to_string());

        
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();

        let nft_proof_wrap =
            ShieldedFractionalNFT::new_bench_ledger(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft_proof = nft_proof_wrap.unwrap();
        let n = 100;
        let m = 10;

        let start = Instant::now();
        for i in (1..n){
            let nft = ledger_authority.mint(&nft_proof).unwrap();
        }
        let ellapsed = start.elapsed();

        println!("Minting {} NFTS took {:.2?}ms",n,ellapsed.as_millis());

        let nft_wrap =
        ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
            ;

        let nft = nft_wrap.unwrap();

        sleep(Duration::from_millis(12000));

        let proof = nft
        .transfer_split_bench_ledger_remote(
            &owner_a_priv,
            [BigInt::from(1), BigInt::from(2)],
            &owner_a_pub,
            &owner_b_pub,
            &mut ledger_authority,
        )
        .unwrap();

        let proof2 = nft
        .transfer_split_bench_ledger_remote(
            &owner_a_priv,
            [BigInt::from(1), BigInt::from(2)],
            &owner_a_pub,
            &owner_b_pub,
            &mut ledger_authority,
        )
        .unwrap();



        let mut handler: Vec<JoinHandle<()>> = Vec::new();

        let start_total = Instant::now();

        for _ in 0..m {

            let proof2: ArkCircomFullProof<Bn254> = serde_json::from_str(&serde_json::to_string(&proof).unwrap()).unwrap();
        
            handler.push(thread::spawn(move ||{
                let mut ledger_authority: RemoteLedgerAuthority<PoseidonHasher, 2> = RemoteLedgerAuthority::new("http://192.168.178.87:8000".to_string());
                
                for i in (1..n){
                    let nft = 
                    ledger_authority.transfer_bench(&proof2).unwrap();
                }
            }));

        }

        for h in handler{
            h.join();
        }
        let ellapsed_total = start_total.elapsed();
        println!("Transfering total {} NFTS took {:.2?}ms",n*m,ellapsed_total.as_millis());


    }

    #[test]
    pub fn nft_bench_update(){

        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

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

        let mut ledger_authority: BatchedLedgerAuthority<PoseidonHasher, 2> = BatchedLedgerAuthority::new(
            TREE_DEPTH,
            trust_list,
            10000
        );

        
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();

        let nft_proof_wrap =
            ShieldedFractionalNFT::new_bench_ledger(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft_proof = nft_proof_wrap.unwrap();
        let n = 1000;

        for i in (1..n){
            let nft = ledger_authority.mint(&nft_proof).unwrap();
        }
        
        let start = Instant::now();
        ledger_authority.update();
        let ellapsed = start.elapsed();

        println!("Updating {} NFTS took {:.2?}ms",n,ellapsed.as_millis());
    }




    
    #[test]
    fn bench_nft_remote() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();

        let trusted_priv = BigInt::from_str("1").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        let mut ledger_authority = RemoteLedgerAuthority::new("http://192.168.49.63:8000".to_string());

        let proof = ledger_authority.get_trust_proof(0).unwrap();

        let n = 100;

        let mut nft: Vec<ShieldedFractionalNFT> = Vec::new();

        for _ in 0..n {
            nft.push(match
                ShieldedFractionalNFT::new_remote(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&trusted_priv)
                    {
                        Ok(res) => res,
                        Err(err) => {

                            panic!("Error")
                        },
                    });
        }
                
        sleep(Duration::from_millis(12000));

        
        let mut nft_a: Vec<ShieldedFractionalNFT> = Vec::new();
        for i in 0..n {
            let (mut nft_b,nft_c) = nft[i]
            .transfer_split_remote(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .unwrap();

            nft_a.push(nft_b);
        }
        
        
        sleep(Duration::from_millis(12000));

        for i in 0..n {
            
            match nft_a[i].burn_remote(&owner_a_priv,&mut ledger_authority){
                Ok(_) => (),
                Err(err) => panic!("{}", err.to_string()),
            }
        }
        



    }
}
