use super::*;
use crate::{mock::*, Error}; // mock实现和脚手架函数 Error错误类型 
use frame_support::{assert_noop, assert_ok, BoundedVec}; // assert_noop断言操作没有产生副作用，没有对存储作操作，没有更新存储
                                                         // assert_ok 断言操作ok
#[test] // 表明是一个测试
fn create_claim_works() {
  // new_test_ext()脚手架，方便编写测试用例
  new_test_ext().execute_with(|| {
    let claim = BoundedVec::try_from(vec![0, 1]).unwrap(); // 把vec转会成substrate内置的BoundedVec类型

    assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone())); // 断言初始化交易返回ok
    // AccountId是64位，所以交易的发送方能给1（mock.rs）


    assert_eq!(
        Proofs::<Test>::get(&claim),
        Some((1, frame_system::Pallet::<Test>::block_number()))
    ); // 断言链上状态，两个值相等
  })
}

#[test]
fn create_claim_failed_when_claim_already_exist() { // 当claim存在时，创建存证是失败的
  new_test_ext().execute_with(|| {
    let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
    let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()); // 第一次创建

    assert_noop!( // 断言此操作不会对链上的存储进行更改，并且返回结果是一个错误，错误类型是ProofAlreadyExist
      PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()), // 第二次创建
      Error::<Test>::ProofAlreadyExist
    );
  })
}

#[test]
fn revoke_claim_works() { // 删除存证
  new_test_ext().execute_with(|| {
    let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
    let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

    assert_ok!(PoeModule::remove_claim(RuntimeOrigin::signed(1), claim.clone()));
  })
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() { // 存证不存在时，删除存证
  new_test_ext().execute_with(|| {
    let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
    
    assert_noop!(
      PoeModule::remove_claim(RuntimeOrigin::signed(1), claim.clone()),
      Error::<Test>::ClaimNotExist
    );
  })
}

#[test]
fn revoke_claim_failed_wrong_owner() { // 删除存证，但删除方不是存证的所有人
  new_test_ext().execute_with(|| {
    let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
    let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());
    
    assert_noop!(
      PoeModule::remove_claim(RuntimeOrigin::signed(2), claim.clone()),
      Error::<Test>::NotClaimOwner
    );
  })
}

#[test]
fn transfer_claim_works() { // 转移存证
	  new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap(); 

		let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());


		assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 2));
	
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		); // 断言链上状态，两个值相等
	})
}

#[test]
fn transfer_claim_failed_wrong_owner() { // 转移存证,但不是存证的拥有人
	  new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap(); 

		let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());
	
		assert_noop!(
      		PoeModule::transfer_claim(RuntimeOrigin::signed(2), claim.clone(), 3),
      		Error::<Test>::NotClaimOwner
    	);
		
	})
}

#[test]
fn transfer_claim_failed_when_claim_is_not_exist() { // 转移存证,但存证不存在
	  new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap(); 

		assert_noop!(
      		PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 2),
      		Error::<Test>::ClaimNotExist
    	);
		
	})
}

#[test]
fn transfer_claim_to_oneself() { // 转移存证给自己
	  new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap(); 

		let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

		assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 1));
	
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		); // 断言链上状态，两个值相等
	})
}
