#! /bin/bash

PTAU_FILE=./ppot_0080_18.ptau
if test -f "$PTAU_FILE"; then 
    echo "powers of tau available"
else
    wget -q https://pse-trusted-setup-ppot.s3.eu-central-1.amazonaws.com/pot28_0080/ppot_0080_18.ptau
fi

circom --r1cs --wasm --c nft_mint.circom

snarkjs groth16 setup nft_mint.r1cs ./ppot_0080_18.ptau nft_mint_0.zkey
snarkjs zkey contribute nft_mint_0.zkey nft_mint.zkey -entropy="HelloWorld" -v 

snarkjs zkey export verificationkey nft_mint.zkey nft_mint_vkey.json

cp nft_mint_cpp/nft_mint.cpp ../witnesscalc/src/nft_mint.cpp 
cp nft_mint_cpp/nft_mint.dat ../witnesscalc/src/nft_mint.dat 

cp nft_mint_cpp/nft_mint.dat ../../backend/nft/lib 
cp nft_mint.zkey ../../backend/nft/lib 