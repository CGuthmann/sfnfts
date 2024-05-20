use std::{str::FromStr, time::Duration};

use criterion::{
    criterion_group,
    criterion_main,
    Criterion, black_box
};

use crypto::poseidon::PoseidonHasher;
use ledger_authority::{DefaultLedgerAuthority, LedgerAuthority};
use nft::{shielded_fractional_nft::ShieldedFractionalNFT, FractionalNFT, ShieldedNFT, NFT};
use num_bigint::BigInt;

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

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            3,
            vec![
                trusted_pub.to_string(),
                "2".to_owned(),
                "3".to_owned(),
                "4".to_owned(),
                "5".to_owned(),
            ],
        );
        
    c.bench_function("benchmarks the nft minting", 
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

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            3,
            vec![
                trusted_pub.to_string(),
                "2".to_owned(),
                "3".to_owned(),
                "4".to_owned(),
                "5".to_owned(),
            ],
        );
        
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();



        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft = nft_wrap.unwrap();
        
    c.bench_function("benchmarks the nft transfer", 
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

fn nft_mint_ledger_local(c: &mut Criterion){
    let source = BigInt::from_str("333").unwrap();
        let value = BigInt::from_str("120").unwrap();
        
        let trusted_priv = BigInt::from_str("1").unwrap();
        let trusted_pub = BigInt::from_str("18586133768512220936620570745912940619677854269274689475585506675881198879027").unwrap();

        let owner_a_priv = BigInt::from_str("2").unwrap();
        let owner_a_pub = BigInt::from_str("8645981980787649023086883978738420856660271013038108762834452721572614684349").unwrap(); //poseidon of 2

        let owner_b_priv = BigInt::from_str("3").unwrap();
        let owner_b_pub = BigInt::from_str("6018413527099068561047958932369318610297162528491556075919075208700178480084").unwrap(); //poseidon of 3

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            3,
            vec![
                trusted_pub.to_string(),
                "2".to_owned(),
                "3".to_owned(),
                "4".to_owned(),
                "5".to_owned(),
            ],
        );

        
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();

        let nft_proof_wrap =
            ShieldedFractionalNFT::new_bench_ledger(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft_proof = nft_proof_wrap.unwrap();
        
    c.bench_function("benchmarks the nft minting of the ledger authority", 
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

        let mut ledger_authority: DefaultLedgerAuthority<PoseidonHasher, 2> = DefaultLedgerAuthority::new(
            3,
            vec![
                trusted_pub.to_string(),
                "2".to_owned(),
                "3".to_owned(),
                "4".to_owned(),
                "5".to_owned(),
            ],
        );
        
        let proof = ledger_authority.get_trust_list().generate_proof(0).unwrap();



        let nft_wrap =
            ShieldedFractionalNFT::new(&source, &value, &owner_a_pub, &mut ledger_authority, &proof,&BigInt::from_str("1").unwrap())
                ;

        let nft = nft_wrap.unwrap();
        
        let proof = nft
        .transfer_split_bench_ledger(
            &owner_a_priv,
            [BigInt::from(1), BigInt::from(2)],
            &owner_a_pub,
            &owner_b_pub,
            &mut ledger_authority,
        )
        .unwrap();
    c.bench_function("benchmarks the nft transfer of the ledger authority", 
        |b| b.iter(||{
            ledger_authority.transfer_bench(&proof);
            
        })
    );
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_millis(10000));
    targets = nft_mint, nft_transfer,nft_mint_ledger_local,nft_transfer_ledger_local);
criterion_main!(benches);