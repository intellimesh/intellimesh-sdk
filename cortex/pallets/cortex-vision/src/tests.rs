
use crate::{mock::*, Error, Percepts};
use frame_support::{assert_noop, assert_ok, traits::{ ConstU32 }};
use crate::types::*;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let test_val: frame_support::BoundedVec<u8, ConstU32<64>> = (*b"TEST").to_vec().try_into().unwrap();
		assert_ok!(TemplateModule::create_percept(RuntimeOrigin::signed(1)));
		// Read pallet storage and assert an expected result.

		//assert_eq!(Percepts::<Test>::get(1_u32), Some(PerceptDetails{owner: 1_u64}));
	});
}
/*
#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			TemplateModule::cause_error(RuntimeOrigin::signed(1)),
			Error::<Test>::pallet::Error
		);
	});
}
*/