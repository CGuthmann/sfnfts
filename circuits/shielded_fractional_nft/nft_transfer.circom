pragma circom 2.0.6;

include "../circomlib/circuits/comparators.circom";
include "../circomlib/circuits/poseidon.circom";
include "./nft.circom";

template ShieldedFractionalNFTTransfer(depth){
    signal input private_key;

    //Parameters
    signal input fraction[2];

    

    //Base NFT
    signal input base_source_hash;
    signal input base_public_key_owner;
    signal input base_total_quantity;
    signal input base_fraction[2]; 
    signal input base_consumed;
    signal input base_salt;

    signal input base_tree_index;
    signal input base_id;
    signal input base_path[depth];
    signal input base_lemma[depth + 2];

    //New NFT A
    signal input a_source_hash;
    signal input a_public_key_owner;
    signal input a_total_quantity;
    signal input a_fraction[2]; //reduced fraction
    signal input a_consumed;
    signal input a_salt;
    signal input a_divider; //common divider used for the fraction 

    //New NFT B
    signal input b_source_hash;
    signal input b_public_key_owner;
    signal input b_total_quantity;
    signal input b_fraction[2];  //reduced fraction
    signal input b_consumed;
    signal input b_salt;
    signal input b_divider; //common divider for the fraction 


    signal output base_hash; //0
    signal output base_tree_index_out; //1
    signal output base_tree_root; //2
    signal output a_hash; //3
    signal output b_hash; //4


    /*
    * Verifying ownership
    */

    component private_key_hasher = Poseidon(1);
    private_key_hasher.inputs[0] <== private_key;
    private_key_hasher.out === base_public_key_owner;

    /*
    * Constraining the Parameter
    */

    component leqt_frac = LessEqThan(252);
    leqt_frac.in[0] <== fraction[0];
    leqt_frac.in[1] <== fraction[1];
    leqt_frac.out === 1;


    component base_nft = ShieldedFractionalNFT(depth);
    base_nft.source_hash <== base_source_hash;
    base_nft.public_key_owner <== base_public_key_owner;
    base_nft.total_quantity <== base_total_quantity;
    base_nft.fraction[0] <== base_fraction[0];
    base_nft.fraction[1] <== base_fraction[1];
    base_nft.consumed <== base_consumed;
    base_nft.salt <== base_salt;
    base_nft.id <== base_id;

    base_nft.lemma[0] <== base_lemma[0];
    for (var i = 0; i < depth; i++) {
            base_nft.path[i] <== base_path[i];
            base_nft.lemma[i + 1] <== base_lemma[i + 1];
    }	
    base_nft.lemma[depth + 1] <== base_lemma[depth+1];

    base_hash <== base_nft.nft_hash;

    base_tree_index_out <== base_tree_index;
    base_tree_root <==  base_lemma[depth+1];


    component a_nft = ShieldedFractionalNFTbase();
    a_nft.source_hash <== a_source_hash;
    a_nft.public_key_owner <== a_public_key_owner;
    a_nft.total_quantity <== a_total_quantity;
    a_nft.fraction[0] <== a_fraction[0];
    a_nft.fraction[1] <== a_fraction[1];
    a_nft.consumed <== a_consumed;
    a_nft.salt <== a_salt;
    a_hash <== a_nft.nft_hash;

    component b_nft = ShieldedFractionalNFTbase();
    b_nft.source_hash <== b_source_hash;
    b_nft.public_key_owner <== b_public_key_owner;
    b_nft.total_quantity <== b_total_quantity;
    b_nft.fraction[0] <== b_fraction[0];
    b_nft.fraction[1] <== b_fraction[1];
    b_nft.consumed <== b_consumed;
    b_nft.salt <== b_salt;
    b_hash <== b_nft.nft_hash;



    /*
    *   Constraining NFT a
    */
    a_source_hash === base_source_hash;
    a_total_quantity === b_total_quantity;

    //constraining that the fraction obtained by multiplication can be reduced to the fraction of a
    signal a_fraction_unreduced[2];
    a_fraction_unreduced[0] <== base_fraction[0]*fraction[0];
    a_fraction_unreduced[1] <== base_fraction[1]*fraction[1];

    signal a_fraction_div[2];
    a_fraction_div[0] <--  a_fraction_unreduced[0] / a_divider;
    a_fraction_div[1] <--  a_fraction_unreduced[1] / a_divider;

    a_fraction_unreduced[0] === a_fraction_div[0]* a_divider;
    a_fraction_unreduced[1] === a_fraction_div[1]* a_divider;

    a_fraction[0] === a_fraction_div[0];
    a_fraction[1] === a_fraction_div[1];



    /*
    *   Constraining NFT b
    */
    b_source_hash === base_source_hash;
    b_total_quantity === b_total_quantity;

    //constraining that the fraction obtained by multiplication can be reduced to the fraction of a
    signal b_fraction_unreduced[2];
    signal total_nom <== base_fraction[0]*fraction[1];
    b_fraction_unreduced[0] <==  total_nom- base_fraction[0]*fraction[0];
    b_fraction_unreduced[1] <== a_fraction_unreduced[1];

    signal b_fraction_div[2];
    b_fraction_div[0] <--  b_fraction_unreduced[0] / b_divider;
    b_fraction_div[1] <--  b_fraction_unreduced[1] / b_divider;

    b_fraction_unreduced[0] === b_fraction_div[0]* b_divider;
    b_fraction_unreduced[1] === b_fraction_div[1]* b_divider;

    b_fraction[0] === b_fraction_div[0];
    b_fraction[1] === b_fraction_div[1];

    


}

component main = ShieldedFractionalNFTTransfer(10);