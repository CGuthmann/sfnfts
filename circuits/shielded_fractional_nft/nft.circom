pragma circom 2.0.6;

include "../circomlib/circuits/poseidon.circom";
include "../heimdall-circuits/heimdall/lib/merkleproof.circom";
include "../heimdall-circuits/heimdall/lib/hashTree.circom";

template ShieldedFractionalNFTbase() {
    //NFT
    signal input source_hash;
    signal input public_key_owner;
    signal input total_quantity;
    signal input fraction[2]; 
    signal input consumed;
    signal input is_child;
    signal input salt;

    signal output nft_hash; //1

    component hash[8]; 
    for (var i = 0; i < 8; i++) {
            hash[i] = Poseidon(1);
    }
    
    component nftTree = hashTree(2,3);

    hash[0].inputs[0] <== source_hash;
    hash[1].inputs[0] <== public_key_owner;
    hash[2].inputs[0] <== total_quantity;
    hash[3].inputs[0] <== fraction[0];
    hash[4].inputs[0] <== fraction[1];
    hash[5].inputs[0] <== consumed;
    hash[6].inputs[0] <== is_child;
    hash[7].inputs[0] <== salt;


    nftTree.values[0] <== hash[0].out;
    nftTree.values[1] <== hash[1].out;
    nftTree.values[2] <== hash[2].out;
    nftTree.values[3] <== hash[3].out;
    nftTree.values[4] <== hash[4].out;
    nftTree.values[5] <== hash[5].out;
    nftTree.values[6] <== hash[6].out;
    nftTree.values[7] <== hash[6].out;

    nft_hash <== nftTree.root;
}

template ShieldedFractionalNFT(depth) {
    //NFT
    signal input source_hash;
    signal input public_key_owner;
    signal input total_quantity;
    signal input fraction[2]; 
    signal input consumed;
    signal input is_child;
    signal input salt;

    signal input id;
    signal input lemma[depth + 2];
    signal input path[depth];

    signal output nft_hash;
    
    component shieldedFractionalNFTbase = ShieldedFractionalNFTbase();

    shieldedFractionalNFTbase.source_hash <== source_hash;
    shieldedFractionalNFTbase.public_key_owner <== public_key_owner;
    shieldedFractionalNFTbase.total_quantity <== total_quantity;
    shieldedFractionalNFTbase.fraction[0] <== fraction[0];
    shieldedFractionalNFTbase.fraction[1] <== fraction[1];
    shieldedFractionalNFTbase.consumed <== consumed;
    shieldedFractionalNFTbase.is_child <== is_child;
    shieldedFractionalNFTbase.salt   <== salt;    

    component merkleProof = MerkleProof(depth);


    component hash =  Poseidon(1);
    hash.inputs[0] <== shieldedFractionalNFTbase.nft_hash;
    merkleProof.lemma[0] <== hash.out;
    for (var i = 0; i < depth; i++) {
            merkleProof.path[i] <== path[i];
            merkleProof.lemma[i + 1] <== lemma[i + 1];
    }
    merkleProof.lemma[depth+1] <== lemma[depth+1];

    nft_hash <== shieldedFractionalNFTbase.nft_hash;

}