//! The service for the app controller

use std::{env::current_dir, fs::remove_file};

use crypto::{merkle_tree::HashTree, poseidon::PoseidonHasher};
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
            match la.transfer(&proof_json.into_inner()) {
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

pub async fn post_transfer_bench(
    proof_json: Json<ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>>
) -> Result<((usize,usize),(usize,usize)), &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            match la.transfer_bench(&proof_json.into_inner()) {
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
    use ledger_authority::{DefaultLedgerAuthority, LedgerAuthority, batched_ledger_authority::BatchedLedgerAuthority};
    use num_bigint::BigInt;

    
    const TREE_DEPTH: u32 = 3;

    static mut LA: Option<BatchedLedgerAuthority<PoseidonHasher, 2>> = None;

    pub fn get_ledger_authority(
    ) -> Result<&'static mut DefaultLedgerAuthority<PoseidonHasher, 2>, Box<dyn Error>> {
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
            let trust_list = lines_from_file("trust_list");

            let la = BatchedLedgerAuthority::<PoseidonHasher, 2>::new(TREE_DEPTH, trust_list, 10000);

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

        let trust_list = lines_from_file("trust_list");

        let la = BatchedLedgerAuthority::<PoseidonHasher, 2>::new(TREE_DEPTH, trust_list, 10000);

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
