// MIT License

// Copyright (c) 2018-2019 The orion Developers

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![allow(non_snake_case)]

use crate::errors::UnknownCryptoError;

#[cfg(feature = "safe_api")]
pub fn StreamCipherTestRunner<Encryptor, Decryptor, Key, Nonce>(
	encryptor: Encryptor,
	decryptor: Decryptor,
	key: Key,
	nonce: Nonce,
	counter: u32,
	input: &[u8],
) where
	Encryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
	Decryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
{
	// Skip tests that require non-empty input.
	// The tests that check for behavior in empty and non-empty
	// input, do not take an input parameter for that reason.
	if !input.is_empty() {
		encrypt_decrypt_same_plaintext(&encryptor, &decryptor, &key, &nonce, counter, input);
		encrypt_decrypt_out_length(&encryptor, &decryptor, &key, &nonce, input);
	}

	encrypt_decrypt_input_empty(&encryptor, &decryptor, &key, &nonce);
	initial_counter_overflow_err(&encryptor, &decryptor, &key, &nonce);
	initial_counter_max_ok(&encryptor, &decryptor, &key, &nonce);
}

/// Given a input length `a` find out how many times
/// the initial counter on encrypt()/decrypt() would
/// increase.
fn counter_increase_times(a: f32) -> u32 {
	// Otherwise a overflowing subtration would happen
	if a <= 64f32 {
		return 0;
	}

	let check_with_floor = (a / 64f32).floor();
	let actual = a / 64f32;

	assert!(actual >= check_with_floor);
	// Subtract one because the first 64 in length
	// the counter does not increase
	if actual > check_with_floor {
		(actual.ceil() as u32) - 1
	} else {
		(actual as u32) - 1
	}
}

#[cfg(feature = "safe_api")]
fn return_if_counter_will_overflow<Encryptor, Decryptor, Key, Nonce>(
	encryptor: &Encryptor,
	decryptor: &Decryptor,
	key: &Key,
	nonce: &Nonce,
	counter: u32,
	input: &[u8],
) -> bool
where
	Encryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
	Decryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
{
	assert!(!input.is_empty());
	let mut dst_out = vec![0u8; input.len()];

	// Overflow will occur and the operation should fail
	let enc_res = encryptor(key, nonce, counter, &[0u8; 0], &mut dst_out).is_err();
	let dec_res = decryptor(key, nonce, counter, &[0u8; 0], &mut dst_out).is_err();

	return enc_res && dec_res;
}

fn encrypt_decrypt_input_empty<Encryptor, Decryptor, Key, Nonce>(
	encryptor: &Encryptor,
	decryptor: &Decryptor,
	key: &Key,
	nonce: &Nonce,
) where
	Encryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
	Decryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
{
	let mut dst_out = [0u8; 64];
	assert!(encryptor(key, nonce, 0, &[0u8; 0], &mut dst_out).is_err());
	assert!(decryptor(key, nonce, 0, &[0u8; 0], &mut dst_out).is_err());
}

#[cfg(feature = "safe_api")]
fn encrypt_decrypt_out_length<Encryptor, Decryptor, Key, Nonce>(
	encryptor: &Encryptor,
	decryptor: &Decryptor,
	key: &Key,
	nonce: &Nonce,
	input: &[u8],
) where
	Encryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
	Decryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
{
	assert!(!input.is_empty());

	let mut dst_out_empty = vec![0u8; 0];
	assert!(encryptor(key, nonce, 0, input, &mut dst_out_empty).is_err());
	assert!(decryptor(key, nonce, 0, input, &mut dst_out_empty).is_err());

	let mut dst_out_less = vec![0u8; input.len() - 1];
	assert!(encryptor(key, nonce, 0, input, &mut dst_out_less).is_err());
	assert!(decryptor(key, nonce, 0, input, &mut dst_out_less).is_err());

	let mut dst_out_exact = vec![0u8; input.len()];
	assert!(encryptor(key, nonce, 0, input, &mut dst_out_exact).is_ok());
	assert!(decryptor(key, nonce, 0, input, &mut dst_out_exact).is_ok());

	let mut dst_out_greater = vec![0u8; input.len() + 1];
	assert!(encryptor(key, nonce, 0, input, &mut dst_out_greater).is_ok());
	assert!(decryptor(key, nonce, 0, input, &mut dst_out_greater).is_ok());
}

#[cfg(feature = "safe_api")]
/// Test that decrypting encrypted data produces correct plaintext.
fn encrypt_decrypt_same_plaintext<Encryptor, Decryptor, Key, Nonce>(
	encryptor: &Encryptor,
	decryptor: &Decryptor,
	key: &Key,
	nonce: &Nonce,
	counter: u32,
	input: &[u8],
) where
	Encryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
	Decryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
{
	assert!(!input.is_empty());

	// Check if the counter would overflow. If yes, ensure that both encryptor and
	// decryptor returned errors.
	if counter_increase_times(input.len() as f32)
		.checked_add(counter)
		.is_none()
	{
		assert!(return_if_counter_will_overflow(
			encryptor, decryptor, key, nonce, counter, input
		));
	}

	let mut dst_out_ct = vec![0u8; input.len()];
	encryptor(key, nonce, counter, input, &mut dst_out_ct).unwrap();

	let mut dst_out_pt = vec![0u8; input.len()];
	decryptor(key, nonce, counter, &dst_out_ct, &mut dst_out_pt).unwrap();

	assert_eq!(input, &dst_out_pt[..]);
}

/// Test that a initial counter will not overflow the internal.
fn initial_counter_overflow_err<Encryptor, Decryptor, Key, Nonce>(
	encryptor: &Encryptor,
	decryptor: &Decryptor,
	key: &Key,
	nonce: &Nonce,
) where
	Encryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
	Decryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
{
	let mut dst_out = [0u8; 128];
	assert!(encryptor(
		key,
		nonce,
		u32::max_value(),
		&[0u8; 65], //  CHACHA_BLOCKSIZE + 1 one to trigger internal block counter addition.
		&mut dst_out,
	)
	.is_err());
	assert!(decryptor(
		key,
		nonce,
		u32::max_value(),
		&[0u8; 65], //  CHACHA_BLOCKSIZE + 1 one to trigger internal block counter addition.
		&mut dst_out,
	)
	.is_err());
}

/// Test that processing one block does not fail on the largest possible initial block counter.
fn initial_counter_max_ok<Encryptor, Decryptor, Key, Nonce>(
	encryptor: &Encryptor,
	decryptor: &Decryptor,
	key: &Key,
	nonce: &Nonce,
) where
	Encryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
	Decryptor: Fn(&Key, &Nonce, u32, &[u8], &mut [u8]) -> Result<(), UnknownCryptoError>,
{
	let mut dst_out = [0u8; 64];
	assert!(encryptor(
		key,
		nonce,
		u32::max_value(),
		&[0u8; 64], // Only needs to process one keystream
		&mut dst_out,
	)
	.is_ok());
	assert!(decryptor(
		key,
		nonce,
		u32::max_value(),
		&[0u8; 64], // Only needs to process one keystream
		&mut dst_out,
	)
	.is_ok());
}
