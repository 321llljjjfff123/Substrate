#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;

#[cfg(test)]
mod mock; // 基本的测试环境

#[cfg(test)]
mod tests; // 测试用例

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	// 使用 Blake2 128 位哈希算法生成 128 位的比特值
	use sp_io::hashing::blake2_128;
	// 使用 Randomness trait 生成高质量的随机数
	use frame_support::traits::Randomness;

	pub type KittyId = u32;
	// 链上存储数据，需要的特征
	#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]); // Kitty的数据结构（u8类型16位）

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>; // 让trait绑定一个产生随机数的方法
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

	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBreed {who: T::AccountId, kitty_id: KittyId, kitty: Kitty},
		KittyTransferred {who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId, // 无效的Kitty id
		SameKittyId, // 同样的kitty id
		NotOwner, // 不是kitty拥有者
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// 创建初始化的kitty
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 得到id时，调用方法进行更新
			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty(Self::random_value(&who)); // 调用方法，获取随机值

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// 产生child kitty值
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyId, kitty_id_2: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId); // 判断这两个id不一样，为同一个则拒绝为它产生一个child Kitty

			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId); // 判断kitty是否创建
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?; // 得到新的kitty id
			let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?; //取出kitty的值
			let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			// 使用简单的算法得到Kitty的数据
			let selector = Self::random_value(&who);
			let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				// 0 choose kitty2, and 1 choose kitty1 
				// 产生一个随机数，随机数去和kittys的每个数据进行异或的操作，当随机数是零时，取前面一个（kitty_1.0[i] & selector{i}），为一时取后面一个（(kitty2.0[i] & !selector[i])）
				// 使新的Kitty的数据来源于它的parent
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}
			let kitty = Kitty(data); // Kitty的数据

			// 链上数据的update
			Kitties::<T>::insert(kitty_id, &kitty); // 存储kitty
			KittyOwner::<T>::insert(kitty_id, &who); // owner的信息
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2)); //parent的信息

			Self::deposit_event(Event::KittyBreed{ who, kitty_id, kitty});
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

	}

	impl<T: Config> Pallet<T> { // kitty_id自增1
		pub fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id.checked_add(1).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}

		pub fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			); // 无论在哪调用，都会得到这几个值的组合，做到区分
			payload.using_encoded(blake2_128)
		}
	}
}
