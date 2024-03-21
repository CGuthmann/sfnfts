pragma circom 2.0.6;
include "./nft.circom";
include "../heimdall-circuits/heimdall/lib/merkleproof.circom";
include "../circomlib/circuits/poseidon.circom";

template BurnShieldedFractionalNFT(depth){

    signal input source_hash;
    signal input public_key_owner;
    signal input total_quantity;
    signal input fraction[2]; 
    signal input consumed;
    signal input salt;

    signal input private_key;

    signal input pathNFT[depth];
    signal input lemmaNFT[depth+2];

    signal output nft_hash_burned; //0
    signal output root; //1
    signal output nft_path[depth]; //2+depth

    /*
    * Verifying ownership
    */

    component private_key_hasher = Poseidon(1);
    private_key_hasher.inputs[0] <== private_key;
    private_key_hasher.out === public_key_owner;
    
    component shieldedFractionalNFT = ShieldedFractionalNFT(depth);

    shieldedFractionalNFT.source_hash <== source_hash;
    shieldedFractionalNFT.public_key_owner <== public_key_owner;
    shieldedFractionalNFT.total_quantity <== total_quantity;
    shieldedFractionalNFT.fraction[0] <== fraction[0];
    shieldedFractionalNFT.fraction[1] <== fraction[1];
    shieldedFractionalNFT.consumed <== 0;
    shieldedFractionalNFT.salt   <== salt;  

    shieldedFractionalNFT.lemma[0] <== lemmaNFT[0];
    for (var i = 0; i < depth; i++) {
            shieldedFractionalNFT.path[i] <== pathNFT[i];
            shieldedFractionalNFT.lemma[i + 1] <== lemmaNFT[i + 1];
    }	
    shieldedFractionalNFT.lemma[depth + 1] <== lemmaNFT[depth+1];

    root <== lemmaNFT[depth+1];
    
    component shieldedFractionalNFTbase = ShieldedFractionalNFTbase();

    shieldedFractionalNFTbase.source_hash <== source_hash;
    shieldedFractionalNFTbase.public_key_owner <== public_key_owner;
    shieldedFractionalNFTbase.total_quantity <== total_quantity;
    shieldedFractionalNFTbase.fraction[0] <== fraction[0];
    shieldedFractionalNFTbase.fraction[1] <== fraction[1];
    shieldedFractionalNFTbase.consumed <== 1;
    shieldedFractionalNFTbase.salt   <== salt;  

    nft_hash_burned <==  shieldedFractionalNFTbase.nft_hash;
}

component main = BurnShieldedFractionalNFT(3);