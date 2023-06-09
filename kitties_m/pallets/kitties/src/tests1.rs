use crate::{mock::*, Error, Event, Kitty as Kitty_cs_};
use frame_support::{assert_ok, assert_noop};
use pallet_balances::Error::InsufficientBalance;

#[test]
fn it_works_for_create_or_evnet() { // 初始化创建测试
	new_test_ext().execute_with(|| {
		let kitty_id = 0; // 全新的环境，所以kitty的id为0
		let account_id = 1; // 用它签名，发送一笔交易
		let name: [u8; 8] = [1,3,1,3,1,3,1,3];

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id); //判断kitty的id是否为零

		// 余额不足时的创建测试
		assert_noop!(KittiesModule::create(RuntimeOrigin::signed(account_id), name), InsufficientBalance::<Test, _>);


		let _ = Balance::set_balance(
			RuntimeOrigin::root()	,
			account_id,
		 	100000000,
		 	0
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), name)); // 判断是否正确的创建kitty

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1); //判断kitty的id是否为更新

		assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true); // 判断kitty是否有值了
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id)); // 测试kitty的拥有者
		assert_eq!(KittiesModule::kitty_parents(kitty_id), None); // 测试kitty的parents

		// 设置最大值
		crate::NextKittyId::<Test>::set(crate::KittyId::max_value());
		assert_noop!(
			KittiesModule::create(RuntimeOrigin::signed(account_id), name),
			Error::<Test>::InvalidKittyId
		);

		// let a = crate::Kitties::kitties(kitty_id).unwrap();
		let kitty_random = KittiesModule::random_value(&account_id); // 获取随机数
		let kitty = crate::Kitty{dna: kitty_random, name}; // 创建Kitty类型的数据
		let _kitty_cs = Kitty_cs_{dna: kitty_random, name}; // 第二种获取Kitty的方法
		
		// 测试初始化的事件
		System::assert_has_event(
			crate::Event::KittyCreated { 
				who: account_id,
				kitty_id: kitty_id,
				kitty: kitty
			}.into()
		);

		// 测试末尾事件
		System::assert_last_event(
			crate::Event::KittyCreated { 
				who: account_id,
				kitty_id: kitty_id,
				kitty
			}.into()
		);

		assert_eq!(System::events().len(), 7) // 测试发出的事件数量

		})
}

#[test]
fn it_works_for_breed_or_evnet() { // 随机数测试
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let name: [u8; 8] = [1,3,1,3,1,3,1,3];

		let _ = Balance::set_balance(
			RuntimeOrigin::root()	,
			account_id,
		 	100000000,
		 	0
		);

		assert_noop!( // 判断插入同样的，虽然kitty没有创建，但是会先判断，可以考虑在lib中调整判断的顺序
			KittiesModule::bred(
				RuntimeOrigin::signed(account_id), kitty_id, kitty_id, name),
			Error::<Test>::SameKittyId
		);

		assert_noop!( // id不存在时，创建Kitty的值
			KittiesModule::bred(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1, name),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), name)); // 初始化创建kitty测试

		// 初始化创建的event测试
		let kitty_1 = crate::Kitty{dna: KittiesModule::random_value(&account_id), name};
		System::assert_has_event(Event::KittyCreated { who: account_id, kitty_id: kitty_id, kitty: kitty_1 }.into());

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), name));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2); // 判断kitty_id是否自增1

		assert_ok!(
			KittiesModule::bred(
				RuntimeOrigin::signed(account_id),
				kitty_id,
				kitty_id + 1,
				name)
		); // 产生child Kitty的测试

	
	// // 产生child kitty的event的测试
	// let kitty_1 = KittiesModule::kitties(kitty_id).unwrap();
	// let kitty_2 = KittiesModule::kitties(kitty_id + 1).unwrap();
	// let selector = KittiesModule::random_value(&account_id);
	// let mut data = [0u8; 16];
	// 		for i in 0..kitty_1.0.len() {
	// 			// 0 choose kitty2, and 1 choose kitty_1 
	// 			// 产生一个随机数，随机数去和kittys的每个数据进行异或的操作，当随机数是零时，取前面一个（kitty_1.0[i] & selector{i}），为一时取后面一个（(kitty2.0[i] & !selector[i])）
	// 			// 使新的Kitty的数据来源于它的parent
	// 			data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
	// 		}
	// // 测试KittyBreed事件
	// // 可以按照pallet中的算法，获取随机数
	// System::assert_has_event(Event::KittyBred { who: account_id, kitty_id: kitty_id + 2, kitty: crate::Kitty(data) }.into()); 
	
	// 测试last的事件
	// 也可以直接获取随机数
	// System::assert_last_event(Event::KittyBred { who: account_id, kitty_id: kitty_id + 2, kitty: kitty_1 }.into()); //这一行测试不通过


		let breed_kitty_id = 2;
		assert_eq!(KittiesModule::next_kitty_id(), breed_kitty_id + 1); // 判断下一个插入的是否加在一
		assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true); // 数据结构是否存储到链上
	
		assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id)); // 查看拥有者
		assert_eq!(
			KittiesModule::kitty_parents(breed_kitty_id),
			Some((kitty_id, kitty_id + 1))
		);// 测试parent


	

		assert_eq!(System::events().len(), 11) // 测试发出的事件数量
	})

}


#[test]
fn it_works_for_transfer_or_evnet() { // transfer测试
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let recipient = 2;
		let name: [u8; 8] = [1,3,1,3,1,3,1,3];


		let _ = Balance::set_balance(
			RuntimeOrigin::root()	,
			account_id,
		 	100000000,
		 	0
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), name)); // 初始化创建account_id

		// 初始化创建的event测试
		let kitty_1 = crate::Kitty{dna: KittiesModule::random_value(&account_id), name};
		System::assert_has_event(Event::KittyCreated { who: account_id, kitty_id: kitty_id, kitty: kitty_1 }.into());


		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id)); // 测试拥有着人

		assert_noop!(KittiesModule::transfer(
			RuntimeOrigin::signed(recipient),
			kitty_id,
			account_id
		), Error::<Test>::NotOwner); // 测试不是拥有者发送

		assert_ok!(KittiesModule::transfer(
			RuntimeOrigin::signed(account_id),
			kitty_id,
			recipient
		)); // 正确的发送

		// transfer的event测试
		System::assert_has_event(Event::KittyTransferred { who: account_id, recipient: recipient, kitty_id: kitty_id }.into());

		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(recipient)); // 判断拥有者
	
		assert_ok!(KittiesModule::transfer(
			RuntimeOrigin::signed(recipient),
			kitty_id,
			account_id
			)
		);

		// transfer的event测试
		System::assert_has_event(Event::KittyTransferred { who: recipient, recipient: account_id, kitty_id: kitty_id }.into());
		// 测试last的事件
		System::assert_last_event(Event::KittyTransferred { who: recipient, recipient: account_id, kitty_id: kitty_id }.into());


		assert_eq!(System::events().len(), 9) // 测试发出的事件数量
		
	})
}


#[test]
fn it_works_for_sale_or_event() {  // sale测试
	new_test_ext().execute_with( || {
		let kitty_id = 0; // 全新的环境，所以kitty的id为0
		let account_id = 1; // 用它签名，发送一笔交易
		let name: [u8; 8] = [1,3,1,3,1,3,1,3];


		let _ = Balance::set_balance(
			RuntimeOrigin::root()	,
			account_id,
		 	100000000,
		 	0
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), name)); // 判断是否正确的创建kitty

		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id)); // 测试拥有着人

		assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(),  false); // 判断是否出售，此时未出售为false
		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id)); // 将kitties的状态改为出售

		// 测试是否发出KittyOnSale事件
		System::assert_has_event(Event::KittyOnSale { who: account_id, kitty_id: kitty_id }.into());


		assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(),  true); // 判断是否出售，此时出售为true

		// 再次售卖Kitty，测试是否报出AlreadyOnSale错误
		assert_noop!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id), Error::<Test>::AlreadyOnSale);

		System::assert_last_event(Event::KittyOnSale { who: account_id, kitty_id: kitty_id }.into());

		assert_eq!(System::events().len(), 8); // 测试发出的事件数量

	})
}

#[test]
fn it_works_for_bug_or_event() {
	new_test_ext().execute_with( || {
		let kitty_id = 0; // 全新的环境，所以kitty的id为0
		let account_id = 1; // 用它签名，发送一笔交易
		let account_id2 =  2; // 用它买kitty
		let name: [u8; 8] = [1,3,1,3,1,3,1,3];
		

		let _ = Balance::set_balance(
			RuntimeOrigin::root()	,
			account_id,
		 	100000000,
		 	0
		);

		// 余额不足时的购买测试
		assert_noop!(KittiesModule::buy(RuntimeOrigin::signed(account_id2), kitty_id), Error::<Test>::InvalidKittyId);

		let _ = Balance::set_balance(
			RuntimeOrigin::root()	,
			account_id2,
		 	100000000,
		 	0
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), name)); // 判断是否正确的创建kitty

		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id)); // 测试拥有着人
		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id)); // 将kitties的状态改为出售
		// 测试是否发出KittyOnSale事件
		System::assert_has_event(Event::KittyOnSale { who: account_id, kitty_id: kitty_id }.into());

		// 测试同一个人购买，是否报出AlreadyOwned错误
		assert_noop!(KittiesModule::buy(RuntimeOrigin::signed(account_id), kitty_id), Error::<Test>::AlreadyOwned);
		// 购买测试
		assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(account_id2), kitty_id));

		// 测试购买后是否发出KittyBought事件
		System::assert_has_event(Event::KittyBought { who: account_id2, kitty_id }.into());
		System::assert_last_event(Event::KittyBought { who: account_id2, kitty_id }.into());

		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id2)); // 测试购买后拥有着人
		assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(), false); // 测试购买后，kitty_id在KittyOnSale里的状态是否移除


		assert_eq!(System::events().len(), 13); // 测试发出的事件数量
	})
}