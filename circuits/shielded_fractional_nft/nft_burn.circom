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

    signal input tree_index;
    signal input id;
    signal input pathNFT[depth];
    signal input lemmaNFT[depth+2];


    signal output base_hash; //0
    signal output base_tree_index_out; //1
    signal output base_tree_root; //2
    signal output burned_hash; //3

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
    shieldedFractionalNFT.id   <== id;  


    shieldedFractionalNFT.lemma[0] <== lemmaNFT[0];
    for (var i = 0; i < depth; i++) {
            shieldedFractionalNFT.path[i] <== pathNFT[i];
            shieldedFractionalNFT.lemma[i + 1] <== lemmaNFT[i + 1];
    }	
    shieldedFractionalNFT.lemma[depth + 1] <== lemmaNFT[depth+1];

    base_hash <== shieldedFractionalNFT.nft_hash;

    base_tree_index_out <== tree_index;

    base_tree_root <== lemmaNFT[depth+1];

    
    component shieldedFractionalNFTburned = ShieldedFractionalNFTbase();

    shieldedFractionalNFTburned.source_hash <== source_hash;
    shieldedFractionalNFTburned.public_key_owner <== public_key_owner;
    shieldedFractionalNFTburned.total_quantity <== total_quantity;
    shieldedFractionalNFTburned.fraction[0] <== fraction[0];
    shieldedFractionalNFTburned.fraction[1] <== fraction[1];
    shieldedFractionalNFTburned.consumed <== 1;
    shieldedFractionalNFTburned.salt   <== salt;  

    burned_hash <==  shieldedFractionalNFTburned.nft_hash;
}

component main = BurnShieldedFractionalNFT(3);