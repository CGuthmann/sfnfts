#! /bin/bash

PTAU_FILE=./powersOfTau28_hez_final_16.ptau
if test -f "$PTAU_FILE"; then 
    echo "powers of tau available"
else
    wget -q https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_16.ptau
fi

circom --r1cs --wasm --c nft_mint.circom

snarkjs groth16 setup nft_mint.r1cs ./powersOfTau28_hez_final_16.ptau nft_mint_0.zkey
snarkjs zkey contribute nft_mint_0.zkey nft_mint.zkey -entropy="HelloWorld" -v 

snarkjs zkey export verificationkey nft_mint.zkey nft_mint_vkey.json

cp nft_mint_cpp/nft_mint.cpp ../witnesscalc/src/nft_mint.cpp 
cp nft_mint_cpp/nft_mint.dat ../witnesscalc/src/nft_mint.dat 

cp nft_mint_cpp/nft_mint.dat ../../backend/nft/lib 
cp nft_mint.zkey ../../backend/nft/lib 