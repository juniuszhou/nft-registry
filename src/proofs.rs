use codec::{Decode, Encode};
use sp_core::H256;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Proof {
    pub leaf_hash: H256,
    pub sorted_hashes: Vec<H256>,
}

impl Proof {
    pub fn new(hash: H256, sorted_hashes: Vec<H256>) -> Self {
        Self {
            leaf_hash: hash,
            sorted_hashes,
        }
    }
}

/// Validates each proof and return true if all the proofs are valid else returns false
///
/// This is an optimized Merkle proof checker. It caches all valid leaves in an array called
/// matches. If a proof is validated, all the intermediate hashes will be added to the array.
/// When validating a subsequent proof, that proof will stop being validated as soon as a hash
/// has been computed that has been a computed hash in a previously validated proof.
///
/// When submitting a list of proofs, the client can thus choose to chop of all the already proven
/// nodes when submitting multiple proofs.
///
/// matches: matches will have a pre computed hashes provided by the client and document root of the
/// reference anchor. static proofs are used to computed the pre computed hashes and the result is
/// checked against document root provided.
pub fn validate_proofs(doc_root: H256, proofs: &Vec<Proof>, static_proofs: [H256; 3]) -> bool {
    if proofs.len() < 1 {
        return false;
    }

    let (valid, mut matches) = pre_matches(static_proofs, doc_root);
    if !valid {
        return false;
    }

    return proofs
        .iter()
        .map(|proof| validate_proof(&mut matches, proof.leaf_hash, proof.sorted_hashes.clone()))
        .fold(true, |acc, b| acc && b);
}

// computes blake2 256 sorted hash of the a and b
// if a < b: blake256(a+b)
// else: blake256(b+a)
fn sort_hash_of(a: H256, b: H256) -> H256 {
    let mut h: Vec<u8> = Vec::with_capacity(64);
    if a < b {
        h.extend_from_slice(&a[..]);
        h.extend_from_slice(&b[..]);
    } else {
        h.extend_from_slice(&b[..]);
        h.extend_from_slice(&a[..]);
    }

    sp_io::hashing::blake2_256(&h).into()
}

// computes blake2 256 hash of the a + b
fn hash_of(a: H256, b: H256) -> H256 {
    let mut h: Vec<u8> = Vec::with_capacity(64);
    h.extend_from_slice(&a[..]);
    h.extend_from_slice(&b[..]);
    sp_io::hashing::blake2_256(&h).into()
}

// validates the proof by computing a sorted hash of the provided proofs with hash as initial value.
// each calculated hash is memoized.
// Validation stops as soon as the any computed hash is found in the matches.
// if no computed hash is found in the matches, validation fails.
fn validate_proof(matches: &mut Vec<H256>, hash: H256, proofs: Vec<H256>) -> bool {
    // if hash is already cached earlier
    if matches.contains(&hash) {
        return true;
    }

    let mut hash = hash;
    for proof in proofs.into_iter() {
        matches.push(proof);
        hash = sort_hash_of(hash, proof);
        if matches.contains(&hash) {
            return true;
        }
        matches.push(hash)
    }

    false
}

// pre_matches takes 3 static proofs and calculate document root.
// the calculated document root is then compared with the document root that is passed.
// if the calculated document root matches, returns true and array of precomputed hashes
// precomputed hashes are used while validating the proofs.
//
//
// Computing Document Root:
//                      DocumentRoot
//                      /          \
//          Signing Root            Signature Root
//          /          \
//   data root 1     data root 2
fn pre_matches(static_proofs: [H256; 3], doc_root: H256) -> (bool, Vec<H256>) {
    let mut matches = Vec::new();
    let basic_data_root = static_proofs[0];
    let zk_data_root = static_proofs[1];
    let signature_root = static_proofs[2];
    matches.push(basic_data_root);
    matches.push(zk_data_root);
    let signing_root = hash_of(basic_data_root, zk_data_root);
    matches.push(signing_root);
    matches.push(signature_root);
    let calc_doc_root = hash_of(signing_root, signature_root);
    matches.push(calc_doc_root);
    (calc_doc_root == doc_root, matches)
}

// appends deposit_address and all the hashes from the proofs and returns keccak hash of the result.
pub fn bundled_hash(proofs: Vec<Proof>, deposit_address: [u8; 20]) -> H256 {
    let hash = proofs
        .into_iter()
        .fold(deposit_address.to_vec(), |mut acc, proof: Proof| {
            acc.extend_from_slice(&proof.leaf_hash[..]);
            acc
        });

    sp_io::hashing::keccak_256(hash.as_slice()).into()
}
