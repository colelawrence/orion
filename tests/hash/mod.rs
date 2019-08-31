pub mod blake2b_kat;
pub mod other_blake2b;
pub mod sha512_nist_cavp;

extern crate orion;
use self::orion::hazardous::hash::{blake2b, sha512};

fn blake2b_test_runner(input: &[u8], key: &[u8], output: &[u8]) {
	// Only make SecretKey if test case key value is not empty, otherwise it will be
	// BLOCKSIZE zero bytes.
	let mut state = if key.is_empty() {
		blake2b::Blake2b::init(None, output.len()).unwrap()
	} else {
		let secret_key = blake2b::SecretKey::from_slice(key).unwrap();
		blake2b::Blake2b::init(Some(&secret_key), output.len()).unwrap()
	};

	state.update(input).unwrap();
	let digest = state.finalize().unwrap();
	// All KAT test vectors are 64 bytes in length
	assert!(digest.as_ref().len() == output.len());
	assert!(digest.as_ref() == &output[..]);
}

fn sha512_test_runner(data: &[u8], output: &[u8]) {
	// Test streaming
	let mut state = sha512::Sha512::init();
	state.update(data).unwrap();
	let digest = state.finalize().unwrap();
	// Test one-shot function
	let digest_one_shot = sha512::digest(data).unwrap();

	assert!(digest.as_ref() == digest_one_shot.as_ref());
	assert!(digest.as_ref() == output);
}
