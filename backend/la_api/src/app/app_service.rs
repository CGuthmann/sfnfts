//! The service for the app controller

use std::{env::current_dir, fs::remove_file};

use db_service::{sqlite_service::SQLiteService, PersistentService, Table};
use num_bigint::BigInt;
use rocket::serde::json::Json;

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

pub async fn get_trust_list() -> Result<Vec<String>, &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => Ok(la.get_trust_list().leaves.clone()),
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}

pub async fn get_merkle_tree() -> Result<Vec<Vec<String>>, &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            let mut leaves: Vec<Vec<String>> = Vec::new();
            for tree in la.get_tree() {
                leaves.push(tree.leaves.clone());
            }
            Ok(leaves)
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
                Ok(res) => Ok(res),
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
                Ok(res) => Ok(res),
                Err(_) => Err("Minting failed."),
            }
        },
        Err(_) => Err("Ledger authority currently unavailable."),
    }
}

pub async fn post_burn(
    proof_json: Json<ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>>
) -> Result<bool, &'static str> {
    match ledger_authority::get_ledger_authority() {
        Ok(la) => {
            match la.burn(&proof_json.into_inner()) {
                Ok(res) => Ok(res),
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
        path::Path,
    };

    use crypto::{
        merkle_tree::{self, HashTree},
        poseidon::PoseidonHasher,
    };
    use db_service::{sqlite_service::SQLiteService, DatabaseError, PersistentService};
    use ledger_authority::LedgerAuthority;

    static mut LA: Option<LedgerAuthority<PoseidonHasher, 2>> = None;

    pub fn get_ledger_authority(
    ) -> Result<&'static mut LedgerAuthority<PoseidonHasher, 2>, Box<dyn Error>> {
        unsafe {
            match &mut LA {
                Some(la) => Ok(la),
                None => {
                    LA = Some(init_ledger_authority()?);
                    Ok(LA.as_mut().unwrap())
                }
            }
        }
    }

    fn init_ledger_authority() -> Result<LedgerAuthority<PoseidonHasher, 2>, Box<dyn Error>> {
        let db = db::get_db()?;

        let la_json_res = db.get_all("ledger_authority")?;

        let mut tree: Vec<HashTree<PoseidonHasher, 2>> = Vec::new();

        if la_json_res.is_empty() {
            let trust_list = lines_from_file("trust_list");

            let la = LedgerAuthority::<PoseidonHasher, 2>::new(3, trust_list);

            return Ok(la);
        }

        match la_json_res[0].get("json") {
            Some(res) => match res {
                Some(la_json) => {
                    let la: LedgerAuthority<PoseidonHasher, 2> = serde_json::from_str(&la_json)?;
                    return Ok(la);
                }
                None => (),
            },
            None => (),
        }

        let trust_list = lines_from_file("trust_list");

        let la = LedgerAuthority::<PoseidonHasher, 2>::new(3, trust_list);

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
            let service = SQLiteService::new("ledger_authority.db");
            let query = String::from(include_str!("create_database.sql"));
            service.create_db(query).unwrap();

            service
        }
    }
}
