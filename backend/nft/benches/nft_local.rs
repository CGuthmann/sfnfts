use std::{str::FromStr, thread::sleep, time::Duration};

use criterion::{
    criterion_group,
    criterion_main,
    Criterion, black_box
};

use crypto::poseidon::PoseidonHasher;
use ledger_authority::{batched_ledger_authority::BatchedLedgerAuthority, remote_ledger_authority::RemoteLedgerAuthority, DefaultLedgerAuthority, LedgerAuthority};
use nft::{shielded_fractional_nft::ShieldedFractionalNFT, FractionalNFT, ShieldedNFT, NFT};
use num_bigint::BigInt;

const TREE_DEPTH: u32 = 22;
const TRUST_DEPTH: u32 = 20;

//benchmarks the nft minting

fn nft_mint(c: &mut Criterion){
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

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            TREE_DEPTH,
            trust_list
        );
        
    c.bench_function("NFT mint local full", 
        |b| b.iter(||{
            

        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();



        let nft_wrap =
            ShieldedFractionalNFT::new_bench(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft = nft_wrap.unwrap();
            
        })
    );
}

fn nft_transfer(c: &mut Criterion){
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

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            TREE_DEPTH,
            trust_list
        );
        
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();



        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft = nft_wrap.unwrap();
        
    c.bench_function("NFT transfer local full", 
        |b| b.iter(||{
            
            let (mut nft_a,mut nft_b) = nft
            .transfer_split_bench(
                &owner_a_priv,
                [BigInt::from(1), BigInt::from(2)],
                &owner_a_pub,
                &owner_b_pub,
                &mut ledger_authority,
            )
            .unwrap();
            
        })
    );
}


//benchmarks the nft burn

fn nft_burn(c: &mut Criterion){
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

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            TREE_DEPTH,
            trust_list
        );
            

        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();



        let mut nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let mut nft = nft_wrap.unwrap();
        
    c.bench_function("NFT burn local full", 
        |b| b.iter(||{
            nft.burn_bench(&owner_a_priv, &mut ledger_authority);
        })
    );
}


fn nft_mint_ledger_local(c: &mut Criterion){
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
        
    c.bench_function("NFT mint local ledger", 
        |b| b.iter(||{
            
            let nft = ledger_authority.mint(&nft_proof).unwrap();
            
        })
    );
}

fn nft_mint_ledger_remotel(c: &mut Criterion){
    let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        let mut ledger_authority = RemoteLedgerAuthority::new("http://192.168.178.87:8000".to_string());

        
        let proof = ledger_authority.get_trust_proof(0).unwrap();

        let nft_proof_wrap =
            ShieldedFractionalNFT::new_bench_ledger(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft_proof = nft_proof_wrap.unwrap();
        
    c.bench_function("NFT mint remote ledger", 
        |b| b.iter(||{
            
            let nft = ledger_authority.mint(&nft_proof).unwrap();
            
        })
    );
}

fn nft_transfer_ledger_local(c: &mut Criterion){
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



        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft = nft_wrap.unwrap();
        println!("root: {}", ledger_authority.get_tree()[0].get_root());
        ledger_authority.update();
        println!("root: {}", ledger_authority.get_tree()[0].get_root());

        let proof = nft
        .transfer_split_bench_ledger(
            &owner_a_priv,
            [BigInt::from(1), BigInt::from(2)],
            &owner_a_pub,
            &owner_b_pub,
            &mut ledger_authority,
        )
        .unwrap();
    c.bench_function("NFT transfer local ledger", 
        |b| b.iter(||{
            ledger_authority.transfer_bench(&proof);
            
        })
    );
}

fn nft_transfer_ledger_remote(c: &mut Criterion){
    let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        let mut ledger_authority = RemoteLedgerAuthority::new("http://192.168.178.87:8000".to_string());
        
        let proof = ledger_authority.get_trust_proof(0).unwrap();



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
    c.bench_function("NFT transfer remote ledger", 
        |b| b.iter(||{
            ledger_authority.transfer_bench(&proof).unwrap();
            
        })
    );
}



criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_millis(10000));
    targets = nft_mint,nft_transfer,nft_burn);

    //nft_transfer_ledger_local,nft_mint_ledger_local,nft_mint_ledger_remotel, nft_transfer_ledger_remote         
criterion_main!(benches);