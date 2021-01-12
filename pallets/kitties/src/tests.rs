use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;


#[test]
fn owned_kitties_can_append_values() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(KittiesModule::create(Origin::signed(1),), Ok(()));
    })
}

#[test]
fn create_failed_with_insufficient_funds() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_noop!(KittiesModule::create(Origin::signed(3),), Error::<Test>::InsufficientFunds);
    })
}

#[test]
fn transfer_success() {
    new_test_ext().execute_with(|| {
        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(1),);
        run_to_block(100);
        assert_ok!(KittiesModule::transfer(Origin::signed(1),2,0));
    })
}

#[test]
fn transfer_failed_with_invaid_id() {
    new_test_ext().execute_with(|| {
        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(1),);
        run_to_block(100);
        assert_noop!(KittiesModule::transfer(Origin::signed(1),2,1),Error::<Test>::InvalidKittyId);
    })
}

#[test]
fn transfer_failed_with_not_the_owner() {
    new_test_ext().execute_with(|| {
        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(1),);
        run_to_block(100);
        assert_noop!(KittiesModule::transfer(Origin::signed(2),3,0),Error::<Test>::RequireOwner);
    })
}

#[test]
fn transfer_failed_with_insufficient_funds() {
    new_test_ext().execute_with(|| {

        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(1),);
        run_to_block(100);
        assert_noop!(KittiesModule::transfer(Origin::signed(1),3,0),Error::<Test>::InsufficientFunds);
    })
}

#[test]
fn breed_success() {
    new_test_ext().execute_with(|| {

        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(1),);
        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(2),);
        run_to_block(100);
        assert_ok!(KittiesModule::breed(Origin::signed(1),0,1));
    })
}

#[test]
fn breed_failed_insufficient_funds() {
    new_test_ext().execute_with(|| {

        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(1),);
        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(2),);
        run_to_block(100);
        assert_noop!(KittiesModule::breed(Origin::signed(3),0,1),Error::<Test>::InsufficientFunds);
    })
}

#[test]
fn breed_failed_with_invalid_id() {
    new_test_ext().execute_with(|| {

        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(1),);
        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(2),);
        run_to_block(100);
        assert_noop!(KittiesModule::breed(Origin::signed(1),0,3),Error::<Test>::InvalidKittyId);
    })
}

#[test]
fn breed_failed_with_same_parent() {
    new_test_ext().execute_with(|| {

        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(1),);
        run_to_block(100);
        let _ = KittiesModule::create(Origin::signed(2),);
        run_to_block(100);
        assert_noop!(KittiesModule::breed(Origin::signed(1),0,0),Error::<Test>::RequireDifferentParent);
    })
}
