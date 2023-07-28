#! /bin/bash

PTAU_FILE=./powersOfTau28_hez_final_16.ptau
if test -f "$PTAU_FILE"; then 
    echo "powers of tau available"
else
    wget -q https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_16.ptau
fi

circom --r1cs --wasm --c nft_transfer.circom

snarkjs groth16 setup nft_transfer.r1cs ./powersOfTau28_hez_final_16.ptau nft_transfer_0.zkey
snarkjs zkey contribute nft_transfer_0.zkey nft_transfer.zkey -entropy="HelloWorld" -v 

snarkjs zkey export verificationkey nft_transfer.zkey nft_transfer_vkey.json

cp nft_transfer_cpp/nft_transfer.cpp ../witnesscalc/src/nft_transfer.cpp 
cp nft_transfer_cpp/nft_transfer.dat ../witnesscalc/src/nft_transfer.dat 

cp nft_transfer_cpp/nft_transfer.dat ../../backend/nft/lib 
cp nft_transfer.zkey ../../backend/nft/lib 