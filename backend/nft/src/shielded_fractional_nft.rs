use std::{error::Error, ops::Mul, str::FromStr};

use crate::{FractionalNFT, ShieldedNFT, NFT};

use crypto::{merkle_tree::MerkleProof, poseidon::PoseidonHasher};
use num_bigint::{BigInt, RandBigInt};
use rand::thread_rng;

use serde::{Deserialize, Serialize};

use self::witness_generation::{mint::proof_nft_mint, transfer::proof_nft_transfer, burn::proof_nft_burn};

use ledger_authority::LedgerAuthority;

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
        ledger_authority: &mut LedgerAuthority<PoseidonHasher, 2>,
        trust_list_proof: &MerkleProof<PoseidonHasher, 2>,
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

        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');
        nft.append_flat_json(&mut json, &"".to_owned())?;
        json.push(',');

        json.push_str(&format!("\"pathTrust\":"));
        append_json_vector(&mut json, &trust_list_proof.path);

        json.push_str(&format!(",\"lemmaTrust\":"));
        append_json_vector(&mut json, &trust_list_proof.lemma);

        json.push('}');

        let proof = proof_nft_mint(&json, None, None)?;

        dbg!(serde_json::to_string(&proof));
        let index = ledger_authority.mint(&proof)?;

        nft.tree_index = index; //updates the merkle tree index stored in the nft

        Ok(nft)
    }

    fn transfer(&self, target: &BigInt) -> Result<Self, Box<dyn std::error::Error>> {
        Err("Unsupported operation.")?
    }

    fn burn(
        &mut self,
        ledger_authority: &mut LedgerAuthority<PoseidonHasher, 2>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let merkle_proof =
            ledger_authority.get_tree()[self.tree_index.0].generate_proof(self.tree_index.1)?;

        //prepares the input arguments for the ZKP
        let mut json = String::new();
        json.push('{');
        self.append_flat_json(&mut json, &"".to_owned())?;

        json.push_str(&format!(",\"pathNFT\":"));
        append_json_vector(&mut json, &merkle_proof.path);

        json.push_str(&format!(",\"lemmaNFT\":"));
        append_json_vector(&mut json, &merkle_proof.lemma);

        json.push('}');
        let proof = proof_nft_burn(&json, None, None)?;

        let result = ledger_authority.burn(&proof);

        self.consumed = true;

        result
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
        ledger_authority: &mut LedgerAuthority<PoseidonHasher, 2>,
    ) -> Result<(Self, Self), Box<dyn std::error::Error>> {
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

    use std::str::FromStr;

    use crate::{shielded_fractional_nft::ShieldedFractionalNFT, *};
    use num_bigint::BigInt;
    use num_traits::Num;

    #[test]
    fn test_nft() {
        let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        let owner = BigInt::from_str_radix(
            "0001020304050607080900010203040506070809000102030405060708090001",
            16,
        )
        .unwrap();
        let owner_b = BigInt::from_str_radix(
            "0001020304050607080900010203040506070809000102030405060708090002",
            16,
        )
        .unwrap();

        let mut ledger_authority: LedgerAuthority<PoseidonHasher, 2> = LedgerAuthority::new(
            3,
            vec![
                owner.to_string(),
                "2".to_owned(),
                "3".to_owned(),
                "4".to_owned(),
                "5".to_owned(),
            ],
        );

        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();

        dbg!(&proof.path);

        let nft =
            ShieldedFractionalNFT::new(&source, &value, &owner, &mut ledger_authority, &proof)
                .unwrap();

        let (mut nft_a,mut nft_b) = nft
            .transfer_split(
                &owner,
                [BigInt::from(1), BigInt::from(2)],
                &owner,
                &owner_b,
                &mut ledger_authority,
            )
            .unwrap();

        dbg!(nft_a.burn(&mut ledger_authority));
        dbg!(nft_a);
        dbg!(nft_b);

    }
}
