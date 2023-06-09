use crate::{*};
use frame_support::{
  pallet_prelude::*,
  storage::StoragePrefixedMap,
  traits::GetStorageVersion,
  weights::Weight,
};

use frame_system::pallet_prelude::*;
use frame_support::{migration::storage_key_iter, Blake2_128Concat};

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
pub struct OldKitty(pub [u8; 16]); // 老版本的数据结构(Kitty的数据结构（u8类型16位）)

pub fn migrate<T: Config>() -> Weight { // 查看链上的版本与设置的版本是否匹配，不匹配则升级
  // 获取链上的版本
  let on_chain_version = Pallet::<T>::on_chain_storage_version();
  // 获取lib中设置的版本常量
  let current_version = Pallet::<T>::current_storage_version();

  if on_chain_version != 0 { // 老的版本号不等于0就退出，因为不支持其它的升级路径
    return Weight::zero();
  }

  if current_version != 1 {
    return Weight::zero();
  }

  // 升级需要的变量
  let module = Kitties::<T>::module_prefix();
  let item = Kitties::<T>::storage_prefix();

  // 相当于kitty中所有的数据存储的map的值取出，用drain方法去掉，去掉的同时使用老的数据和默认的名字组成一个新的数据，再插入到kitties中
  for (index, kitty) in storage_key_iter::<KittyId, OldKitty, Blake2_128Concat>(module, item).drain() { //.drain()老的数据不要了
    
    let new_Kitty = Kitty { 
      dna: kitty.0, // 从第一个数据断言，拷贝到新的dna中
      name: *b"abcdabcd", // 因为老的数据没有name，新设置数据
    };

    Kitties::<T>::insert(index, &new_Kitty);


  }

  Weight::zero()

}