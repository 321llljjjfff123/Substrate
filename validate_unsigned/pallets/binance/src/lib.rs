#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;



use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");
// 这个模块是签名需要用到的
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {

	use frame_support::pallet_prelude::*;
	use frame_system::{
		offchain::{
			AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
			SigningTypes,
		},
		pallet_prelude::*,
	};

	use frame_support::inherent::Vec;
	use sp_runtime::{
		offchain::{http, Duration},
		transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
		RuntimeDebug,
	};

	// use crate::binance_price::{BinancePrice, BinancePriceResponse};

	use serde::{Deserialize, Deserializer};
	use core::fmt;
	use core::cmp::Eq;
	#[derive(Deserialize, Encode, Decode, Clone, TypeInfo, PartialEq, Eq)]
	struct GithubInfo {
			#[serde(deserialize_with = "de_string_to_bytes")]
			login: Vec<u8>,
			#[serde(deserialize_with = "de_string_to_bytes")]
			blog: Vec<u8>,
			public_repos: u32,
	}

	pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
        where
        D: Deserializer<'de>,
        {
            let s: &str = Deserialize::deserialize(de)?;
            Ok(s.as_bytes().to_vec())
        }

    impl fmt::Debug for GithubInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{{ login: {}, blog: {}, public_repos: {} }}",
                sp_std::str::from_utf8(&self.login).map_err(|_| fmt::Error)?,
                sp_std::str::from_utf8(&self.blog).map_err(|_| fmt::Error)?,
                &self.public_repos
                )
        }
    }


	const ONCHAIN_TX_KEY: &[u8] = b"binance::indexing_parcel_weight";

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct Payload<Public> {
		binance_price_data: GithubInfo,
		public: Public,	
	}

	impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	#[derive(Debug, Encode, Decode, Default)]
	struct IndexingData(BoundedVec<u8, ConstU32<4>>);

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	// pub trait Config: frame_system::Config +
	// frame_system::offchain::SendTransactionTypes<Call<Self>> {
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
	}


		// Errors inform users that something went wrong.
		#[pallet::error]
		pub enum Error<T> {
			/// Error names should be descriptive.
			NoneValue,
			/// Errors should have helpful documentation associated with them.
			StorageOverflow,
		}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn unsigned_extrinsic_with_signed_payload(origin: OriginFor<T>, payload: Payload<T::Public>, _signature: T::Signature,) -> DispatchResult {
			ensure_none(origin)?;

      log::info!("OCW ==> in call unsigned_extrinsic_with_signed_payload: {:?}", payload.binance_price_data);
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

	}

	// 发送未签名交易时需要实现的 trait
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const UNSIGNED_TXS_PRIORITY: u64 = 100;
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("my-pallet")
					.priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			match call {
				Call::unsigned_extrinsic_with_signed_payload { ref payload, ref signature } => {
					if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
						return InvalidTransaction::BadProof.into()
					}
					valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Offchain worker entry point.
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("OCW ==> Hello World from offchain workers!: {:?}", block_number);
			if let Ok(info) = Self::fetch_github_info() {
          log::info!("OCW ==> Github Info: {:?}", info);

					
			// let value: u64 = 42;
			// // This is your call to on-chain extrinsic together with any necessary parameters.
			// let call = Call::submit_data_unsigned { key: value };

			// // `submit_unsigned_transaction` returns a type of `Result<(), ()>`
			// //	 ref: https://paritytech.github.io/substrate/master/frame_system/offchain/struct.SubmitTransaction.html
			// _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
			// 	.map_err(|_| {
			// 	log::error!("OCW ==> Failed in offchain_unsigned_tx");
			// });
			
			// let number: u64 = 42;
			// Retrieve the signer to sign the payload
			let signer = Signer::<T, T::AuthorityId>::any_account();

			// `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(), ()>)>`.
			//	 The returned result means:
			//	 - `None`: no account is available for sending transaction
			//	 - `Some((account, Ok(())))`: transaction is successfully sent
			//	 - `Some((account, Err(())))`: error occurred when sending the transaction
			if let Some((_, res)) = signer.send_unsigned_transaction(
				// this line is to prepare and return payload
				|acct| Payload { binance_price_data: info.clone(), public: acct.public.clone() },
				|payload, signature| Call::unsigned_extrinsic_with_signed_payload { payload, signature },
			) {
				match res {
					Ok(()) => {log::info!("OCW ==> unsigned tx with signed payload successfully sent.");}
					Err(()) => {log::error!("OCW ==> sending unsigned tx with signed payload failed.");}
				};
			} else {
				// The case of `None`: no account is available for sending
				log::error!("OCW ==> No local account available");
			}

      } else {
          log::info!("OCW ==> Error while fetch github info!");
      }
			log::info!("OCW ==> Leave from offchain workers!: {:?}", block_number);
		}
	}



	impl<T: Config> Pallet<T> {
		fn fetch_github_info() -> Result<GithubInfo, http::Error> {
				// prepare for send request
				let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
				let request =
						http::Request::get("https://api.github.com/orgs/substrate-developer-hub");
				let pending = request
						.add_header("User-Agent", "Substrate-Offchain-Worker")
						.deadline(deadline).send().map_err(|_| http::Error::IoError)?;
				let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
				if response.code != 200 {
						log::warn!("Unexpected status code: {}", response.code);
						return Err(http::Error::Unknown)
				}
				let body = response.body().collect::<Vec<u8>>();
				let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
						log::warn!("No UTF8 body");
						http::Error::Unknown
				})?;

				// parse the response str
				let gh_info: GithubInfo =
						serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

				Ok(gh_info)
		}
	}

}
