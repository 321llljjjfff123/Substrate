#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;

mod migrations;

#[cfg(test)]
mod mock; // 基本的测试环境

#[cfg(test)]
mod tests1; // 测试用例2
// #[cfg(test)]
// mod tests; // 测试用例


#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, ensure};
	use frame_system::pallet_prelude::*;

	// 使用 Blake2 128 位哈希算法生成 128 位的比特值
	use sp_io::hashing::blake2_128;
	use crate::migrations;

	// 使用 Randomness trait 生成高质量的随机数
	// ReservableCurrency质押模块
	use frame_support::traits::{Randomness, Currency, ExistenceRequirement}; // , ReservableCurrency

	use frame_support::PalletId; // 添加数据类型         
	use sp_runtime::traits::AccountIdConversion; // 转换函数（id转换成账号，需要转账时，可以往其中转账）

	pub type KittyId = u32;
	
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// 链上存储数据，需要的特征
	#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
	// pub struct Kitty(pub [u8; 16]); // Kitty的数据结构（u8类型16位）
	pub struct Kitty {
		pub dna: [u8; 16],
		pub name: [u8; 8],
	}
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1); // 将版本改号为1,如果之前版本是0，当变为1时，可以去调用升级的程序

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)] // 版本号定义在pallet的属性中
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>; // 让trait绑定一个产生随机数的方法

		// 定义trait时没有绑定balance这个trait，在Runtiem可以绑定任意的pallet到Currency上都是可以的，只要pallet支持Currency里的所有的方法（软件设计的方法，尽量通过这些trait去解耦，也就是接口的方法，不会把两者之间非常紧密的联系在一起）

		type Currency: Currency<Self::AccountId>; //ReservableCurrency
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>; // 添加Pallet的id（按照一定规范，id可以转换成账号）
	}
	
	// The pallet's runtime storage items.
	#[pallet::storage] // 下一个Kitty的id
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)] // kitty值
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)] // 拥有者
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage] 
	#[pallet::getter(fn kitty_parents)] // 父子关系
	pub type KittyParents<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	
	#[pallet::storage] 
	#[pallet::getter(fn kitty_on_sale)] // 出售kitty
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty }, // 初始化Kitty事件
		KittyBred {who: T::AccountId, kitty_id: KittyId, kitty: Kitty}, // 产生child kitty值事件
		KittyTransferred {who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId}, // 发送Kitty事件
		KittyOnSale {who: T::AccountId, kitty_id: KittyId}, // Kitty出售事件
		KittyBought {who: T::AccountId, kitty_id: KittyId}, // Kitty已买事件
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId, // 无效的Kitty id
		SameKittyId, // 同样的kitty id
		NotOwner, // 不是kitty拥有者
		AlreadyOnSale, // kitty已经销售
		NotOnSale, // kitty没有处于出售的状态
		AlreadyOwned, // Kitty已经拥有了
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			migrations::v1::migrate::<T>()
			// migrations::v2_0::migrate::<T>() // 从版本0，升级到版本2
			// migrations::v2_1::migrate::<T>() // 从本吧1，升级到版本2
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// 创建初始化的kitty
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 得到id时，调用方法进行更新
			let kitty_id = Self::get_next_id()?;
			let dna = Self::random_value(&who); // 调用方法，获取随机值
			let kitty = Kitty { dna, name };

			let price = T::KittyPrice::get(); // 获取Kitty的价格

			// T::Currency::reserve(&who, price)?; // 插入价格(签名人， 价格)

			// 更改经济模型，crate时不是做reserve，而是做transfer的操作
			// 从发起交易的账号,转移到我们自己的account_id， price价格
			// ExistenceRequirement这个trait，在发送transfer交易时，需要指定对一个账号是不是KeepAlive的，保存该账号有最小的余额在里面
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?; 

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// 产生child kitty值
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn bred(origin: OriginFor<T>, kitty_id_1: KittyId, kitty_id_2: KittyId, name: [u8; 8]) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId); // 判断这两个id不一样，为同一个则拒绝为它产生一个child Kitty

			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId); // 判断kitty是否创建
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?; // 得到新的kitty id
			let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?; //取出kitty的值
			let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			// 使用简单的算法得到Kitty的数据
			let selector = Self::random_value(&who);
			let mut dna = [0u8; 16];
			// for i in 0..kitty_1.0.len() {
			// 	// 0 choose kitty2, and 1 choose kitty1 
			// 	// 产生一个随机数，随机数去和kittys的每个数据进行异或的操作，当随机数是零时，取前面一个（kitty_1.0[i] & selector{i}），为一时取后面一个（(kitty2.0[i] & !selector[i])）
			// 	// 使新的Kitty的数据来源于它的parent
			// 	data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			// }
			let kitty = Kitty{dna, name}; // Kitty的数据
			
			let price = T::KittyPrice::get(); // 获取Kitty的价格
			// T::Currency::reserve(&who, price)?; // 插入价格(签名人， 价格)
			// 更改经济模型，crate时不是做reserve，而是做transfer的操作
			// 从发起交易的账号,转移到我们自己的account_id， price价格
			// ExistenceRequirement这个trait，在发送transfer交易时，需要指定对一个账号是不是KeepAlive的，保存该账号有最小的余额在里面
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?; 

			// 链上数据的update
			Kitties::<T>::insert(kitty_id, &kitty); // 存储kitty
			KittyOwner::<T>::insert(kitty_id, &who); // owner的信息
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2)); //parent的信息

			Self::deposit_event(Event::KittyBred{ who, kitty_id, kitty});
			Ok(())

		} 


		// 发送kitty
		#[pallet::call_index(2)] 
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer(origin: OriginFor<T>, kitty_id: u32, recipient: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(KittyOwner::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner == who, Error::<T>::NotOwner);

			KittyOwner::<T>::insert(kitty_id, &recipient);
			Self::deposit_event(Event::KittyTransferred {who, recipient, kitty_id});
			Ok(())
		}

		// 售卖kitty
		#[pallet::call_index(3)] 
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn sale (
			origin: OriginFor<T>,
			kitty_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?; // 判断是否有这个Kitty的id

			// 判断是不是Kitty的拥有者
			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);
			// 判断Kitty是否已经在出售
			ensure!(!Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::AlreadyOnSale);

			<KittyOnSale<T>>::insert(kitty_id, ()); // 出售后将kitty_id对应的状态改为变为空元组（此时只是出售，还没买，所以没有拥有者）
			Self::deposit_event(Event::KittyOnSale{ who, kitty_id}); // 判断Kitty是否为出售的状态

		

			Ok(())
		}

		// 购买Kitty
		#[pallet::call_index(4)] 
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn buy (
			origin: OriginFor<T>,
			kitty_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?; // 判断是否有这个Kitty的id

			let owner = Self::kitty_owner(kitty_id).ok_or::<DispatchError>(Error::<T>::NotOwner.into())?;
			ensure!(owner != who, Error::<T>::AlreadyOwned); // 判断交易方和买方不是一个用户
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::NotOnSale); // 判断Kitty有没有处于出售状态

			let price = T::KittyPrice::get(); // 获取价格
			// T::Currency::reserve(&who, price)?; // 插入新的拥有人，及对应的价格
			// T::Currency::unreserve(&owner, price); // 移除对应的拥有人，及对应的价格

			// transfer直接从旧的买家转移到新的买家
			T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?; 

			<KittyOwner<T>>::insert(kitty_id, &who); // 卖出后将kitty_id的拥有者改为对应的用户
			<KittyOnSale<T>>::remove(kitty_id); // 将Kitty的出售状态移除

			Self::deposit_event(Event::KittyBought {who, kitty_id});

			Ok(())
		}

	}

	impl<T: Config> Pallet<T> { // kitty_id自增1
		pub fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id.checked_add(1).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}

		// 随机数
		pub fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			); // 无论在哪调用，都会得到这几个值的组合，做到区分
			payload.using_encoded(blake2_128)
		}

		// 获取PalletId对应的账号
		fn get_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
