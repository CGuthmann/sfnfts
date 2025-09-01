#! /bin/bash

PTAU_FILE=./ppot_0080_18.ptau
if test -f "$PTAU_FILE"; then 
    echo "powers of tau available"
else
    wget -q https://pse-trusted-setup-ppot.s3.eu-central-1.amazonaws.com/pot28_0080/ppot_0080_18.ptau
fi

circom --r1cs --wasm --c nft_burn.circom

snarkjs groth16 setup nft_burn.r1cs ./ppot_0080_18.ptau nft_burn_0.zkey
snarkjs zkey contribute nft_burn_0.zkey nft_burn.zkey -entropy="HelloWorld" -v 

snarkjs zkey export verificationkey nft_burn.zkey nft_burn_vkey.json

cp nft_burn_cpp/nft_burn.cpp ../witnesscalc/src/nft_burn.cpp 
cp nft_burn_cpp/nft_burn.dat ../witnesscalc/src/nft_burn.dat 

cp nft_burn_cpp/nft_burn.dat ../../backend/nft/lib 
cp nft_burn.zkey ../../backend/nft/lib 