pragma circom 2.0.6;

include "../circomlib/circuits/comparators.circom";
include "../circomlib/circuits/poseidon.circom";
include "./nft.circom";

template ShieldedFractionalNFTChildParent(depth){
    //parent NFT
    signal input parent_source_hash;
    signal input parent_public_key_owner;
    signal input parent_total_quantity;
    signal input parent_fraction[2]; 
    signal input parent_consumed;
    signal input parent_is_child;
    signal input parent_salt;

    signal input parent_id;
    signal input parent_index;
    signal input parent_path[depth];
    signal input parent_lemma[depth + 2];

    //Child NFT
    signal input child_source_hash;
    signal input child_public_key_owner;
    signal input child_total_quantity;
    signal input child_fraction[2]; 
    signal input child_consumed;
    signal input child_is_child;
    signal input child_salt;

    signal input child_id;
    signal input child_index;
    signal input child_path[depth];
    signal input child_lemma[depth + 2];

    signal output child_hash;   //0
    signal output parent_hash;  //1
    signal output child_root;   //2
    signal output parent_root;  //3
    signal output child_nft_id; //4
    signal output parent_nft_id;//5
    signal output child_index_out;   //6
    signal output parent_index_out;  //7

    ///NFT Validity checks
    //child_nft
    component child_nft = ShieldedFractionalNFT(depth);
    child_nft.source_hash <== child_source_hash;
    child_nft.public_key_owner <== child_public_key_owner;
    child_nft.total_quantity <== child_total_quantity;
    child_nft.fraction[0] <== child_fraction[0];
    child_nft.fraction[1] <== child_fraction[1];
    child_nft.consumed <== child_consumed;
    child_nft.is_child <== child_is_child;
    child_nft.salt <== child_salt;
    child_nft.id <== child_id;

    child_nft.lemma[0] <== child_lemma[0];
    for (var i = 0; i < depth; i++) {
            child_nft.path[i] <== child_path[i];
            child_nft.lemma[i + 1] <== child_lemma[i + 1];
    }	
    child_nft.lemma[depth + 1] <== child_lemma[depth+1];

    child_root <==  child_lemma[depth+1];


    child_hash <== child_nft.nft_hash;
    
    //parent_nft
    component parent_nft = ShieldedFractionalNFT(depth);
    parent_nft.source_hash <== parent_source_hash;
    parent_nft.public_key_owner <== parent_public_key_owner;
    parent_nft.total_quantity <== parent_total_quantity;
    parent_nft.fraction[0] <== parent_fraction[0];
    parent_nft.fraction[1] <== parent_fraction[1];
    parent_nft.consumed <== parent_consumed;
    parent_nft.is_child <== parent_is_child;
    parent_nft.salt <== parent_salt;
    parent_nft.id <== parent_id;


    parent_nft.lemma[0] <== parent_lemma[0];
    for (var i = 0; i < depth; i++) {
            parent_nft.path[i] <== parent_path[i];
            parent_nft.lemma[i + 1] <== parent_lemma[i + 1];
    }	
    parent_nft.lemma[depth + 1] <== parent_lemma[depth+1];

    parent_root <==  parent_lemma[depth+1];

    
    parent_hash <== parent_nft.nft_hash;
    //sanity checks

    child_is_child === 1;

    component child_id_hasher = Poseidon(2);
    child_id_hasher.inputs[0] <== child_source_hash;
    child_id_hasher.inputs[1] <== child_salt;

    child_nft_id <== child_id_hasher.out;

    

    component parent_id_hasher = Poseidon(2);
    parent_id_hasher.inputs[0] <== parent_source_hash;
    parent_id_hasher.inputs[1] <== parent_salt;

    parent_nft_id <== parent_id_hasher.out;

    //relationship check
    child_public_key_owner === parent_nft_id;

    child_index_out <== child_index;
    parent_index_out <== parent_index;
}

component main = ShieldedFractionalNFTChildParent(10);