#! /bin/bash

PTAU_FILE=./ppot_0080_18.ptau
if test -f "$PTAU_FILE"; then 
    echo "powers of tau available"
else
    wget -q https://pse-trusted-setup-ppot.s3.eu-central-1.amazonaws.com/pot28_0080/ppot_0080_18.ptau
fi

circom --r1cs --wasm --c nft_child_parent.circom

snarkjs groth16 setup nft_child_parent.r1cs ./ppot_0080_18.ptau nft_child_parent_0.zkey
snarkjs zkey contribute nft_child_parent_0.zkey nft_child_parent.zkey -entropy="HelloWorld" -v 

snarkjs zkey export verificationkey nft_child_parent.zkey nft_child_parent_vkey.json

cp nft_child_parent_cpp/nft_child_parent.cpp ../witnesscalc/src/nft_child_parent.cpp 
cp nft_child_parent_cpp/nft_child_parent.dat ../witnesscalc/src/nft_child_parent.dat 

cp nft_child_parent_cpp/nft_child_parent.dat ../../backend/nft/lib 
cp nft_child_parent.zkey ../../backend/nft/lib 



circom --r1cs --wasm --c nft_transfer_child.circom

snarkjs groth16 setup nft_transfer_child.r1cs ./ppot_0080_18.ptau nft_transfer_child_0.zkey
snarkjs zkey contribute nft_transfer_child_0.zkey nft_transfer_child.zkey -entropy="HelloWorld" -v 

snarkjs zkey export verificationkey nft_transfer_child.zkey nft_transfer_child_vkey.json

cp nft_transfer_child_cpp/nft_transfer_child.cpp ../witnesscalc/src/nft_transfer_child.cpp 
cp nft_transfer_child_cpp/nft_transfer_child.dat ../witnesscalc/src/nft_transfer_child.dat 

cp nft_transfer_child_cpp/nft_transfer_child.dat ../../backend/nft/lib 
cp nft_transfer_child.zkey ../../backend/nft/lib 