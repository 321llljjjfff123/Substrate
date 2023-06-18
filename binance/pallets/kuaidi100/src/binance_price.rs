use core::fmt;

use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use scale_info::TypeInfo;
use serde::{Deserialize, Deserializer};
use sp_core::ConstU32;
use sp_runtime::BoundedVec;

/// 快递100价格查询返回的数据
#[derive(Deserialize, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct BinancePrice {
	#[serde(deserialize_with = "de_string_to_bounded_bytes")]
	pub name: BoundedVec<u8, ConstU32<32>>,
	#[serde(deserialize_with = "de_string_to_bounded_bytes", rename(deserialize = "kuaidiCom"))]
	pub kuaidi_com: BoundedVec<u8, ConstU32<32>>,
	#[serde(deserialize_with = "de_string_to_u32", rename(deserialize = "totalprice"))]
	pub total_price: u32,
}

impl fmt::Debug for BinancePrice {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("BinancePrice")
			.field("name", &sp_std::str::from_utf8(&self.name).map_err(|_| fmt::Error)?)
			.field("kuaidi_com", &sp_std::str::from_utf8(&self.kuaidi_com).map_err(|_| fmt::Error)?)
			.field("total_price", &self.total_price)
			.finish()
	}
}

/// 快递100价格查询响应
#[derive(Deserialize, Debug, Encode, Decode)]
pub struct BinancePriceResponse {
	pub status: i32,
	#[serde(deserialize_with = "de_vec_to_bounded_vec")]
	pub data: BoundedVec<BinancePrice, ConstU32<10>>,
}

/// 反序列化字符串到 BoundedVec<u8, ConstU32<32>>
pub fn de_string_to_bounded_bytes<'de, D>(de: D) -> Result<BoundedVec<u8, ConstU32<32>>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(BoundedVec::<u8, ConstU32<32>>::try_from(s.as_bytes().to_vec())
		.map_err(|_| serde::de::Error::custom("BoundedVec error"))?)
}

/// 反序列化字符串到 u32
pub fn de_string_to_u32<'de, D>(de: D) -> Result<u32, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	let price = s.parse::<f32>().map_err(|_| serde::de::Error::custom("Invalid f32"))?;
	let price = (price * 100f32) as u32;
	Ok(price)
}

/// 反序列化 Vec<BinancePrice> 到 BoundedVec<BinancePrice, ConstU32<10>>
pub fn de_vec_to_bounded_vec<'de, D>(
	de: D,
) -> Result<BoundedVec<BinancePrice, ConstU32<10>>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: Vec<BinancePrice> = Deserialize::deserialize(de)?;
	let a = s
		.into_iter()
		.take(10)
		.collect::<Vec<BinancePrice>>()
		.try_into()
		.map_err(|_| serde::de::Error::custom("BoundedVec error"))?;

	Ok(a)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deserialize_works() {
		let s = r#"{
			"mins": 5,
			"price": "26521.30818281"
	}"#;
		let response = serde_json::from_str::<BinancePriceResponse>(s).unwrap();

		assert!(response.status == 200);
		assert!(!response.data.is_empty());

		println!("{:#?}", response);
	}
}
