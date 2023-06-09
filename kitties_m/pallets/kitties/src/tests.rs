use crate::{mock::*, Error, Event, Kitty as Kitty_cs_};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_create_or_evnet() { // 初始化创建测试
	new_test_ext().execute_with(|| {
		let kitty_id = 0; // 全新的环境，所以kitty的id为0
		let account_id = 1; // 用它签名，发送一笔交易

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id); //判断kitty的id是否为零
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id))); // 判断是否正确的创建kitty

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1); //判断kitty的id是否为更新

		assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true); // 判断kitty是否有值了
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id)); // 测试kitty的拥有者
		assert_eq!(KittiesModule::kitty_parents(kitty_id), None); // 测试kitty的parents

		// 设置最大值
		crate::NextKittyId::<Test>::set(crate::KittyId::max_value());
		assert_noop!(
			KittiesModule::create(RuntimeOrigin::signed(account_id)),
			Error::<Test>::InvalidKittyId
		);

		// let a = crate::Kitties::kitties(kitty_id).unwrap();
		let kitty_random = KittiesModule::random_value(&account_id); // 获取随机数
		let kitty = crate::Kitty(kitty_random); // 创建Kitty类型的数据
		let _kitty_cs = Kitty_cs_(kitty_random); // 第二种获取Kitty的方法
		
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

		assert_eq!(System::events().len(), 1) // 测试发出的事件数量

		})
}

#[test]
fn it_works_for_breed_or_evnet() { // 随机数测试
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		assert_noop!( // 判断插入同样的，虽然kitty没有创建，但是会先判断，可以考虑在lib中调整判断的顺序
			KittiesModule::bred(
				RuntimeOrigin::signed(account_id), kitty_id, kitty_id),
			Error::<Test>::SameKittyId
		);

		assert_noop!( // id不存在时，创建Kitty的值
			KittiesModule::bred(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id))); // 初始化创建kitty测试

		// 初始化创建的event测试
		let kitty_1 = crate::Kitty(KittiesModule::random_value(&account_id));
		System::assert_has_event(Event::KittyCreated { who: account_id, kitty_id: kitty_id, kitty: kitty_1 }.into());

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2); // 判断kitty_id是否自增1

		assert_ok!(
			KittiesModule::bred(
				RuntimeOrigin::signed(account_id),
				kitty_id,
				kitty_id + 1)
		); // 产生child Kitty的测试

	
	// 产生child kitty的event的测试
	let kitty_1 = KittiesModule::kitties(kitty_id).unwrap();
	let kitty_2 = KittiesModule::kitties(kitty_id + 1).unwrap();
	let selector = KittiesModule::random_value(&account_id);
	let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				// 0 choose kitty2, and 1 choose kitty_1 
				// 产生一个随机数，随机数去和kittys的每个数据进行异或的操作，当随机数是零时，取前面一个（kitty_1.0[i] & selector{i}），为一时取后面一个（(kitty2.0[i] & !selector[i])）
				// 使新的Kitty的数据来源于它的parent
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}
	// 测试KittyBreed事件
	// 可以按照pallet中的算法，获取随机数
	System::assert_has_event(Event::KittyBred { who: account_id, kitty_id: kitty_id + 2, kitty: crate::Kitty(data) }.into()); 
	// 测试last的事件
	// 也可以直接获取随机数
	System::assert_last_event(Event::KittyBred { who: account_id, kitty_id: kitty_id + 2, kitty: kitty_1 }.into());


		let breed_kitty_id = 2;
		assert_eq!(KittiesModule::next_kitty_id(), breed_kitty_id + 1); // 判断下一个插入的是否加在一
		assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true); // 数据结构是否存储到链上
	
		assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id)); // 查看拥有者
		assert_eq!(
			KittiesModule::kitty_parents(breed_kitty_id),
			Some((kitty_id, kitty_id + 1))
		);// 测试parent


	

		assert_eq!(System::events().len(), 3) // 测试发出的事件数量（KittyCreated：2，KittyBreed： 1）
	})

}


#[test]
fn it_works_for_transfer_or_evnet() { // transfer测试
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let recipient = 2;

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id))); // 初始化创建account_id

		// 初始化创建的event测试
		let kitty_1 = crate::Kitty(KittiesModule::random_value(&account_id));
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


		assert_eq!(System::events().len(), 3) // 测试发出的事件数量(KittyTransferred：2, KittyCreated： 1 )
		
	})
}

