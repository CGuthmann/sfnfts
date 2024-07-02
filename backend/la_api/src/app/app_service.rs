//! The service for the app controller

use std::{env::current_dir, fs::remove_file};

use crypto::{merkle_tree::{HashTree, MerkleProof}, poseidon::PoseidonHasher};
use db_service::{sqlite_service::SQLiteService, PersistentService, Table};
use ::ledger_authority::LedgerAuthority;
use num_bigint::BigInt;
use rocket::serde::json::Json;

use self::ledger_authority::save_ledger_authority;
//TODO: unsure about error handling
//  - forward errors from deep backend until they reach the api?
//      or return a error response as soon as possible?
//  - define api errors?

pub async fn get_nullifier() -> Result<Vec<BigInt>, &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => Ok(la.get_nullifier().clone()),
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}

pub async fn get_trust_list() -> Result<HashTree<PoseidonHasher,2>, &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => Ok(la.get_trust_list()),
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}

pub async fn get_merkle_tree() -> Result<Vec<HashTree<PoseidonHasher,2>>, &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            Ok(la.get_tree())
        }
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}

pub async fn post_mint(
    proof_json: Json<ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>>,
) -> Result<(usize, usize), &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            match la.mint(&proof_json.into_inner()) {
                Ok(res) => {
                    match save_ledger_authority() {
                        Ok(_) =>  Ok(res),
                        Err(_) => Err("Minting failed."),
                    }
                   
                },
                Err(_) => Err("Minting failed."),
            }
        },
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}

pub async fn post_transfer(
    proof_json: Json<ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>>
) -> Result<((usize,usize),(usize,usize)), &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            match la.transfer_split(&proof_json.into_inner()) {
                Ok(res) => {
                    match save_ledger_authority() {
                        Ok(_) =>  Ok(res),
                        Err(_) => Err("Transfer failed."),
                    }
                },
                Err(_) => Err("Transfer failed."),
            }
        },
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}


pub async fn post_burn(
    proof_json: Json<ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>>
) -> Result<(usize,usize), &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            match la.burn(&proof_json.into_inner()) {
                Ok(res) => {
                    match save_ledger_authority() {
                        Ok(_) =>  Ok(res),
                        Err(_) => Err("Burning failed."),
                    }
                },
                Err(_) => Err("Burning failed."),
            }
        },
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}


pub async fn get_merkle_proof(
    index: Json<(usize,usize)>
) -> Result<MerkleProof<PoseidonHasher, 2>, &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            let index = index.into_inner();
            match la.get_tree()[index.0].generate_proof(index.1){
                Ok(pf) => Ok(pf),
                Err(_) => Err("Merkle Proof generation failed"),
            }
        },
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}

pub async fn get_trust_proof(
    index: Json<usize>
) -> Result<MerkleProof<PoseidonHasher, 2>, &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            let index = index.into_inner();
            match la.get_trust_list().generate_proof(index){
                Ok(pf) => Ok(pf),
                Err(_) => Err("Merkle Proof generation failed"),
            }
        },
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}

mod ledger_authority {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
        path::Path, str::FromStr,
    };

    use crypto::{
        merkle_tree::{self, HashTree},
        poseidon::PoseidonHasher,
    };
    use db_service::{sqlite_service::SQLiteService, DatabaseError, PersistentService};
    use ledger_authority::{LedgerAuthority, batched_ledger_authority::BatchedLedgerAuthority};
    use num_bigint::BigInt;

    
    const TREE_DEPTH: u32 = 10;
    const TRUST_DEPTH: u32 = 10;
    const UPDATE_INTERVALL_MS: usize = 10000;

    static mut LA: Option<BatchedLedgerAuthority<PoseidonHasher, 2>> = None;

    pub fn get_ledger_authority(
    ) -> Result<&'static mut BatchedLedgerAuthority<PoseidonHasher, 2>, Box<dyn Error>> {
        unsafe {
            match &mut LA {
                Some(la) => Ok(la),
                None => {
                    LA = Some(init_ledger_authority()?);
                    LA.as_mut().unwrap().init();
                    Ok(LA.as_mut().unwrap())
                }
            }
        }
    }

    pub fn save_ledger_authority()
    -> Result<(), Box<dyn Error>>{
        unsafe { 
            match &LA  {
                Some(la) => {
                    let json = serde_json::to_string(la)?;
                    let db = db::get_db()?;
                    db.update("ledger_authority", &BigInt::from_str("0")?, vec!["json".to_string()], vec![json])?;
                    Ok(())
                },
                None => Ok(()),
            }
        }
    }

    fn init_ledger_authority() -> Result<BatchedLedgerAuthority<PoseidonHasher, 2>, Box<dyn Error>> {
        let db = db::get_db()?;

        let la_json_res = db.get_all("ledger_authority")?;

        let mut tree: Vec<HashTree<PoseidonHasher, 2>> = Vec::new();

        if la_json_res.is_empty() {
            let mut trust_list = lines_from_file("trust_list");
            
            let element = trust_list[0].clone();
            for _ in trust_list.len()..((2 as u32).pow(TRUST_DEPTH) as usize) {
                trust_list.push(element.clone());
            }

            let la = BatchedLedgerAuthority::<PoseidonHasher, 2>::new(TREE_DEPTH, trust_list, UPDATE_INTERVALL_MS);

            db.save("ledger_authority", vec!["ID".to_string(), "json".to_string()], vec!["0".to_string(), serde_json::to_string(&la)?])?;

            return Ok(la);
        }

        match la_json_res[0].get("json") {
            Some(res) => match res {
                Some(la_json) => {
                    let la: BatchedLedgerAuthority<PoseidonHasher, 2> = serde_json::from_str(&la_json)?;
                    return Ok(la);
                }
                None => (),
            },
            None => (),
        }

        let mut trust_list = lines_from_file("trust_list");
        
        let element = trust_list[0].clone();
        for _ in trust_list.len()..((2 as u32).pow(TRUST_DEPTH) as usize) {
            trust_list.push(element.clone());
        }

        let la = BatchedLedgerAuthority::<PoseidonHasher, 2>::new(TREE_DEPTH, trust_list, UPDATE_INTERVALL_MS);

        Ok(la)
    }

    fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
        let file = File::open(filename).expect("no such file");
        let buf = BufReader::new(file);
        buf.lines()
            .map(|l| l.expect("Could not parse line"))
            .collect()
    }

    mod db {
        use std::error::Error;

        use db_service::{sqlite_service::SQLiteService, PersistentService};

        static mut DB: Option<SQLiteService> = None;
        pub fn get_db() -> Result<&'static SQLiteService, Box<dyn Error>> {
            unsafe {
                match &DB {
                    Some(db) => Ok(&db),
                    None => {
                        DB = Some(init_db());
                        Ok(DB.as_ref().unwrap())
                    }
                }
            }
        }

        fn init_db() -> SQLiteService {
            let service = SQLiteService::new("ledger_authority.sqlite");
            let query = String::from(include_str!("create_database.sql"));
            service.create_db(query).unwrap();

            service
        }
    }
}
