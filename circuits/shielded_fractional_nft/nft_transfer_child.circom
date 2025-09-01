pragma circom 2.0.6;

include "../circomlib/circuits/comparators.circom";
include "../circomlib/circuits/poseidon.circom";
include "./nft.circom";

template ShieldedFractionalNFTTransferChild(depth){
    signal input private_key;


    //Root NFT
    signal input root_source_hash;
    signal input root_public_key_owner;
    signal input root_total_quantity;
    signal input root_fraction[2]; 
    signal input root_consumed;
    signal input root_is_child;
    signal input root_salt;

    //Child NFT
    signal input child_source_hash;
    signal input child_public_key_owner;
    signal input child_total_quantity;
    signal input child_fraction[2]; 
    signal input child_consumed;
    signal input child_is_child;
    signal input child_salt;

    signal input child_tree_index;
    signal input child_id;
    signal input child_path[depth];
    signal input child_lemma[depth + 2];

    //New NFT
    signal input new_public_key_owner;
    signal input new_is_child;



    signal output child_hash; //0
    signal output child_tree_index_out; //1
    signal output child_tree_root; //2
    signal output child_owner_id; //3
    signal output new_hash; //4
    signal output root_nft_id; //5


    /*
    * Verifying ownership
    */

    component private_key_hasher = Poseidon(1);
    private_key_hasher.inputs[0] <== private_key;
    private_key_hasher.out === root_public_key_owner;

    /*
    * Constraining the Parameter
    */

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

    child_hash <== child_nft.nft_hash;

    child_tree_index_out <== child_tree_index;
    child_tree_root <==  child_lemma[depth+1];

    child_is_child === 1;


    component new_nft = ShieldedFractionalNFTbase();
    new_nft.source_hash <== child_source_hash;
    new_nft.public_key_owner <== new_public_key_owner;
    new_nft.total_quantity <== child_total_quantity;
    new_nft.fraction[0] <== child_fraction[0];
    new_nft.fraction[1] <== child_fraction[1];
    new_nft.consumed <== child_consumed;
    new_nft.is_child <== new_is_child;
    new_nft.salt <== child_salt;
    new_hash <== new_nft.nft_hash;
    



    child_owner_id <== child_nft.public_key_owner;

    component root_id_hasher = Poseidon(2);
    root_id_hasher.inputs[0] <== root_source_hash;
    root_id_hasher.inputs[1] <== root_salt;

    root_nft_id <== root_id_hasher.out;
}

component main = ShieldedFractionalNFTTransferChild(10);