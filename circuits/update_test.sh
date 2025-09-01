#!/bin/bash

cp witnesscalc/package/lib/* test/
cp witnesscalc/src/*.dat test/

cp shielded_fractional_nft/*.zkey test/
cp shielded_fractional_nft/*.r1cs test/


cp shielded_fractional_nft/nft_transfer_child_js/*.wasm test/
cp shielded_fractional_nft/nft_child_parent_js/*.wasm test/
cp shielded_fractional_nft/nft_mint_js/*.wasm test/
cp shielded_fractional_nft/nft_transfer_js/*.wasm test/
cp shielded_fractional_nft/nft_burn_js/*.wasm test/