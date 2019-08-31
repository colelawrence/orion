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

use crate::errors::UnknownCryptoError;

/// Trait to define default streaming contexts that can be tested.
pub trait DefaultTestableStreamingContext<T: PartialEq> {
	/// Interface to streaming API.
	fn init() -> Self;
	fn reset(&mut self) -> Result<(), UnknownCryptoError>;
	fn update(&mut self, input: &[u8]) -> Result<(), UnknownCryptoError>;
	fn finalize(&mut self) -> Result<T, UnknownCryptoError>;
	fn one_shot(input: &[u8]) -> Result<T, UnknownCryptoError>;

	/// Testing utiliy-functions.
	fn compare_states(state_1: &Self, state_2: &Self);
}

pub struct StreamingContextConsistencyTester<R: PartialEq, T: DefaultTestableStreamingContext<R>> {
	_return_type: R,
	_streaming_context: T,
	blocksize: usize,
}

impl<R, T> StreamingContextConsistencyTester<R, T>
where
	R: PartialEq + core::fmt::Debug,
	T: DefaultTestableStreamingContext<R>,
{
	pub fn new(streaming_context: T, return_type: R, blocksize: usize) -> Self {
		Self {
			_return_type: return_type,
			_streaming_context: streaming_context,
			blocksize,
		}
	}

	#[cfg(feature = "safe_api")]
	pub fn run_all_tests_with_input(&self, data: &[u8]) {
		self.consistency(data);
		self.consistency(&[0u8; 0]);
		self.produces_same_state(data);
		self.incremental_processing_with_leftover(self.blocksize);
		self.double_finalize_with_reset_no_update_ok(data);
		self.double_finalize_with_reset_ok(data);
		self.double_finalize_err(data);
		self.update_after_finalize_with_reset_ok(data);
		self.update_after_finalize_err(data);
		self.double_reset_ok(data);
	}

	/// Used when quickcheck is not available to generate input.
	/// Default input `data` is used instead.
	pub fn run_all_tests(&self) {
		// Default input data.
		let data = "Testing streaming context consistency and correctness".as_bytes();

		self.consistency(data);
		self.consistency(&[0u8; 0]);
		self.produces_same_state(data);
		self.double_finalize_with_reset_no_update_ok(data);
		self.double_finalize_with_reset_ok(data);
		self.double_finalize_err(data);
		self.update_after_finalize_with_reset_ok(data);
		self.update_after_finalize_err(data);
		self.double_reset_ok(data);
	}

	/// Related bug: https://github.com/brycx/orion/issues/46
	/// Testing different usage combinations of init(), update(),
	/// finalize() and reset() produce the same Digest.
	///
	/// It is important to ensure this is also called with empty
	/// `data`.
	pub fn consistency(&self, data: &[u8]) {
		// init(), update(), finalize()
		let mut state_1 = T::init();
		state_1.update(data).unwrap();
		let res_1 = state_1.finalize().unwrap();

		// init(), reset(), update(), finalize()
		let mut state_2 = T::init();
		state_2.reset().unwrap();
		state_2.update(data).unwrap();
		let res_2 = state_2.finalize().unwrap();

		// init(), update(), reset(), update(), finalize()
		let mut state_3 = T::init();
		state_3.update(data).unwrap();
		state_3.reset().unwrap();
		state_3.update(data).unwrap();
		let res_3 = state_3.finalize().unwrap();

		// init(), update(), finalize(), reset(), update(), finalize()
		let mut state_4 = T::init();
		state_4.update(data).unwrap();
		let _ = state_4.finalize().unwrap();
		state_4.reset().unwrap();
		state_4.update(data).unwrap();
		let res_4 = state_4.finalize().unwrap();

		assert!(res_1 == res_2);
		assert!(res_2 == res_3);
		assert!(res_3 == res_4);

		// Tests for the assumption that returning Ok() on empty update() calls
		// with streaming API's, gives the correct result. This is done by testing
		// the reasoning that if update() is empty, returns Ok(), it is the same as
		// calling init() -> finalize(). i.e not calling update() at all.
		if data.is_empty() {
			// init(), finalize()
			let mut state_5 = T::init();
			let res_5 = state_5.finalize().unwrap();

			// init(), reset(), finalize()
			let mut state_6 = T::init();
			state_6.reset().unwrap();
			let res_6 = state_6.finalize().unwrap();

			// init(), update(), reset(), finalize()
			let mut state_7 = T::init();
			state_7.update(b"WRONG DATA").unwrap();
			state_7.reset().unwrap();
			let res_7 = state_7.finalize().unwrap();

			assert!(res_4 == res_5);
			assert!(res_5 == res_6);
			assert!(res_6 == res_7);
		}
	}

	/// Related bug: https://github.com/brycx/orion/issues/46
	/// Testing different usage combinations of init(), update(),
	/// finalize() and reset() produce the same Digest.
	pub fn produces_same_state(&self, data: &[u8]) {
		// init()
		let state_1 = T::init();

		// init(), reset()
		let mut state_2 = T::init();
		state_2.reset().unwrap();

		// init(), update(), reset()
		let mut state_3 = T::init();
		state_3.update(data).unwrap();
		state_3.reset().unwrap();

		// init(), update(), finalize(), reset()
		let mut state_4 = T::init();
		state_4.update(data).unwrap();
		let _ = state_4.finalize().unwrap();
		state_4.reset().unwrap();

		T::compare_states(&state_1, &state_2);
		T::compare_states(&state_2, &state_3);
		T::compare_states(&state_3, &state_4);
	}

	#[cfg(feature = "safe_api")]
	// Test for issues when incrementally processing data
	// with leftover in the internal buffer.
	pub fn incremental_processing_with_leftover(&self, blocksize: usize) {
		for len in 0..blocksize * 4 {
			let data = vec![0u8; len];
			let mut state = T::init();
			let mut other_data: Vec<u8> = Vec::new();

			other_data.extend_from_slice(&data);
			state.update(&data).unwrap();

			if data.len() > blocksize {
				other_data.extend_from_slice(b"");
				state.update(b"").unwrap();
			}
			if data.len() > blocksize * 2 {
				other_data.extend_from_slice(b"Extra");
				state.update(b"Extra").unwrap();
			}
			if data.len() > blocksize * 3 {
				other_data.extend_from_slice(&[0u8; 256]);
				state.update(&[0u8; 256]).unwrap();
			}
            
            let streaming_result = state.finalize().unwrap();
			let one_shot_result = T::one_shot(&other_data).unwrap();

			assert!(streaming_result == one_shot_result);
		}
	}

	pub fn double_finalize_with_reset_no_update_ok(&self, data: &[u8]) {
		let mut state = T::init();
		state.update(data).unwrap();
		let _ = state.finalize().unwrap();
		state.reset().unwrap();
		assert!(state.finalize().is_ok());
	}

	pub fn double_finalize_with_reset_ok(&self, data: &[u8]) {
		let mut state = T::init();
		state.update(data).unwrap();
		let _ = state.finalize().unwrap();
		state.reset().unwrap();
		state.update(data).unwrap();
		assert!(state.finalize().is_ok());
	}

	pub fn double_finalize_err(&self, data: &[u8]) {
		let mut state = T::init();
		state.update(data).unwrap();
		let _ = state.finalize().unwrap();
		assert!(state.finalize().is_err());
	}

	pub fn update_after_finalize_with_reset_ok(&self, data: &[u8]) {
		let mut state = T::init();
		state.update(data).unwrap();
		let _ = state.finalize().unwrap();
		let _ = state.reset();
		assert!(state.update(data).is_ok());
	}

	/// Related bug: https://github.com/brycx/orion/issues/28
	pub fn update_after_finalize_err(&self, data: &[u8]) {
		let mut state = T::init();
		state.update(data).unwrap();
		let _ = state.finalize().unwrap();
		assert!(state.update(data).is_err());
	}

	pub fn double_reset_ok(&self, data: &[u8]) {
		let mut state = T::init();
		state.update(data).unwrap();
		let _ = state.finalize().unwrap();
		let _ = state.reset();
		assert!(state.reset().is_ok());
	}
}
