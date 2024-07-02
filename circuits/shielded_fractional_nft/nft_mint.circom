pragma circom 2.0.6;
include "./nft.circom";
include "../heimdall-circuits/heimdall/lib/merkleproof.circom";
include "../circomlib/circuits/poseidon.circom";

template MintShieldedFractionalNFT(depth){

    signal input source_hash;
    signal input public_key_owner;
    signal input total_quantity;
    signal input fraction[2]; 
    signal input consumed;
    signal input salt;

    signal input private_key;
    signal input pathTrust[depth];
    signal input lemmaTrust[depth+2];

    signal output nft_hash; //0
    signal output trust_root; //1
    
    component shieldedFractionalNFTbase = ShieldedFractionalNFTbase();

    shieldedFractionalNFTbase.source_hash <== source_hash;
    shieldedFractionalNFTbase.public_key_owner <== public_key_owner;
    shieldedFractionalNFTbase.total_quantity <== total_quantity;
    shieldedFractionalNFTbase.fraction[0] <== fraction[0];
    shieldedFractionalNFTbase.fraction[1] <== fraction[1];
    shieldedFractionalNFTbase.consumed <== consumed;
    shieldedFractionalNFTbase.salt   <== salt;    

    nft_hash <== shieldedFractionalNFTbase.nft_hash;

    component private_key_hasher = Poseidon(1);
    private_key_hasher.inputs[0] <== private_key;

    
    component public_key_hasher = Poseidon(1);
    public_key_hasher.inputs[0] <== private_key_hasher.out;

    public_key_hasher.out === lemmaTrust[0];

    component merkleProof = MerkleProof(depth);
    merkleProof.lemma[0] <== lemmaTrust[0];
    for (var i = 0; i < depth; i++) {
            merkleProof.path[i] <== pathTrust[i];
            merkleProof.lemma[i + 1] <== lemmaTrust[i + 1];
    }	
    merkleProof.lemma[depth + 1] <== lemmaTrust[depth+1];

    trust_root <== merkleProof.lemma[depth + 1];
}

component main = MintShieldedFractionalNFT(10);