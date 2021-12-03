// Copyright 2019-2021 @polkadot/wasm-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

use std::convert::TryFrom;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer as _, Verifier as _};
use wasm_bindgen::prelude::*;

/// Keypair helper function.
fn new_from_pair(pair: &[u8]) -> Keypair {
	match Keypair::from_bytes(pair) {
		Ok(p) => p,
		_ => panic!("Provided pair is invalid.")
	}
}

/// Keypair helper function
fn new_from_parts(pubkey: &[u8], seckey: &[u8]) -> Keypair {
	let mut pair = vec![];

	pair.extend_from_slice(seckey);
	pair.extend_from_slice(pubkey);

	new_from_pair(&pair)
}

/// Keypair helper function.
fn new_from_seed(seed: &[u8]) -> Keypair {
	match &SecretKey::from_bytes(seed) {
		Ok(s) => {
			let pubkey: PublicKey = s.into();

			new_from_parts(pubkey.as_bytes(), seed)
		},
		_ => panic!("Invalid seed provided.")
	}
}

/// PublicKey helper
fn new_public(pubkey: &[u8]) -> PublicKey {
	match PublicKey::from_bytes(pubkey) {
		Ok(p) => p,
		_ => panic!("Provided public key is invalid.")
	}
}

/// Generate a key pair.
///
/// * seed: UIntArray with 32 element
///
/// returned vector is the concatenation of first the private key (64 bytes)
/// followed by the public key (32) bytes.
#[wasm_bindgen]
pub fn ext_ed_from_seed(seed: &[u8]) -> Vec<u8> {
	new_from_seed(seed)
		.to_bytes()
		.to_vec()
}

/// Sign a message
///
/// The combination of both public and private key must be provided.
/// This is effectively equivalent to a keypair.
///
/// * pubkey: UIntArray with 32 element
/// * private: UIntArray with 64 element
/// * message: Arbitrary length UIntArray
///
/// * returned vector is the signature consisting of 64 bytes.
#[wasm_bindgen]
pub fn ext_ed_sign(pubkey: &[u8], seckey: &[u8], message: &[u8]) -> Vec<u8> {
	new_from_parts(pubkey, seckey)
		.sign(message)
		.to_bytes()
		.to_vec()
}

/// Verify a message and its corresponding against a public key;
///
/// * signature: UIntArray with 64 element
/// * message: Arbitrary length UIntArray
/// * pubkey: UIntArray with 32 element
#[wasm_bindgen]
pub fn ext_ed_verify(signature: &[u8], message: &[u8], pubkey: &[u8]) -> bool {
	match Signature::try_from(signature) {
		Ok(s) => new_public(pubkey)
			.verify(message, &s)
			.is_ok(),
		_ => false
	}
}

#[cfg(test)]
pub mod tests {
	extern crate rand;

	use hex_literal::hex;
	use super::*;
	use ed25519_dalek::{SIGNATURE_LENGTH, KEYPAIR_LENGTH, SECRET_KEY_LENGTH};

	fn generate_random_seed() -> Vec<u8> {
		(0..32).map(|_| rand::random::<u8>() ).collect()
	}

	#[test]
	fn can_new_keypair() {
		let seed = generate_random_seed();
		let keypair = ext_ed_from_seed(seed.as_slice());

		assert!(keypair.len() == KEYPAIR_LENGTH);
	}

	#[test]
	fn creates_pair_from_known() {
		let seed = b"12345678901234567890123456789012";
		let expected = hex!("2f8c6129d816cf51c374bc7f08c3e63ed156cf78aefb4a6550d97b87997977ee");
		let keypair = ext_ed_from_seed(seed);
		let public = &keypair[SECRET_KEY_LENGTH..KEYPAIR_LENGTH];

		assert_eq!(public, expected);
	}

	#[test]
	fn can_sign_message() {
		let seed = generate_random_seed();
		let keypair = ext_ed_from_seed(seed.as_slice());
		let private = &keypair[0..SECRET_KEY_LENGTH];
		let public = &keypair[SECRET_KEY_LENGTH..KEYPAIR_LENGTH];
		let message = b"this is a message";
		let signature = ext_ed_sign(public, private, message);

		assert!(signature.len() == SIGNATURE_LENGTH);
	}

	#[test]
	fn can_verify_message() {
		let seed = generate_random_seed();
		let keypair = ext_ed_from_seed(seed.as_slice());
		let private = &keypair[0..SECRET_KEY_LENGTH];
		let public = &keypair[SECRET_KEY_LENGTH..KEYPAIR_LENGTH];
		let message = b"this is a message";
		let signature = ext_ed_sign(public, private, message);
		let is_valid = ext_ed_verify(&signature[..], message, public);

		assert!(is_valid);
	}

	#[test]
	fn can_verify_known() {
		let public = hex!("2f8c6129d816cf51c374bc7f08c3e63ed156cf78aefb4a6550d97b87997977ee");
		let message = b"this is a message";
		let signature = hex!("90588f3f512496f2dd40571d162e8182860081c74e2085316e7c4396918f07da412ee029978e4dd714057fe973bd9e7d645148bf7b66680d67c93227cde95202");
		let is_valid = ext_ed_verify(&signature, message, &public);

		assert!(is_valid);
	}

	#[test]
	fn can_verify_known_wrong() {
		let public = hex!("2f8c6129d816cf51c374bc7f08c3e63ed156cf78aefb4a6550d97b87997977ee");
		let message = b"this is a message";
		let signature = &[0u8; 64];
		let is_valid = ext_ed_verify(signature, message, &public);

		assert_eq!(is_valid, false);
	}
}
