#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)] // test模式时引入mock子模块
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    pub use frame_support::pallet_prelude::*;
    pub use frame_system::pallet_prelude::*;
    pub use sp_std::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config { // Config模块的配置接口
        #[pallet::constant]
        type MaxClaimLength: Get<u32>; // 存证最大长度
        // 触发事件
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

	#[pallet::storage]
    #[pallet::getter(fn proofs)]
    // 存储单元Proofs
    pub type Proofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxClaimLength>,  // key 是BoundedVec类型的u8，第二个参数是BoundedVec的最大长度
        (T::AccountId, T::BlockNumber), // value是Toupe，第一个元素是AccountId，第二个参数是BlockNumber
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
        ClaimRemoved(T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExist,
        ClaimTooLong,
        ClaimNotExist,
        NotClaimOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn create_claim(
            origin: OriginFor<T>, // 存证交易的发送方
            claim: BoundedVec<u8, T::MaxClaimLength>, // 存证代表的哈希值
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?; // 校验交易是否签名

            // let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
            // 校验Proofs中是否包含claim，包含返回错误（存证已经存在，不能重复插入）
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

            // 如果不存在，则插入这个存在
            Proofs::<T>::insert(
                &claim, // 键
                (sender.clone(), frame_system::Pallet::<T>::block_number()), // 值
            );

            // Emit an event.
            Self::deposit_event(Event::ClaimCreated(sender, claim));

            Ok(().into())
        } 

        #[pallet::call_index(1)]
		#[pallet::weight(0)]
        pub fn remove_claim(
            origin: OriginFor<T>, // 存证交易的发送方
            claim: BoundedVec<u8, T::MaxClaimLength>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?; // 校验交易是否签名
            // let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
            
            // 校验存证的值是否存证，不存在返回一个ClaimNotExist的错误
            // 存在返回owner和blockNumber，blockNumber不关心，忽略
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(owner == sender, Error::<T>::NotClaimOwner);

            Proofs::<T>::remove(&claim);

            // Emit an event.
            Self::deposit_event(Event::ClaimRemoved(sender, claim));

            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn transfer_claim(origin: OriginFor<T>, claim: BoundedVec<u8, T::MaxClaimLength>, dest: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            // let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
            let (owner, _block_number) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			Proofs::<T>::insert(
                &claim,
                (dest, frame_system::Pallet::<T>::block_number()),
            );

            // Emit an event.
            Self::deposit_event(Event::ClaimRemoved(sender, claim));

            Ok(().into())
        }
    }
}
