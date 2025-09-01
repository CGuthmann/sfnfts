#! /bin/bash

PTAU_FILE=./ppot_0080_18.ptau
if test -f "$PTAU_FILE"; then 
    echo "powers of tau available"
else
    wget -q https://pse-trusted-setup-ppot.s3.eu-central-1.amazonaws.com/pot28_0080/ppot_0080_18.ptau
fi

circom --r1cs --wasm --c nft_transfer.circom

snarkjs groth16 setup nft_transfer.r1cs ./ppot_0080_18.ptau nft_transfer_0.zkey
snarkjs zkey contribute nft_transfer_0.zkey nft_transfer.zkey -entropy="HelloWorld" -v 

snarkjs zkey export verificationkey nft_transfer.zkey nft_transfer_vkey.json

cp nft_transfer_cpp/nft_transfer.cpp ../witnesscalc/src/nft_transfer.cpp 
cp nft_transfer_cpp/nft_transfer.dat ../witnesscalc/src/nft_transfer.dat 

cp nft_transfer_cpp/nft_transfer.dat ../../backend/nft/lib 
cp nft_transfer.zkey ../../backend/nft/lib 