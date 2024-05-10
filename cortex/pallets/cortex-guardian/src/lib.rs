//
// Maintain escrow 
// Validate the connections between percetp and cognito using offchain procecssing
// Start/Pause/Stop connecton
// Pay cognito for the work performed
//

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
		SignedPayload, Signer, SigningTypes, SubmitTransaction,
	},
	pallet_prelude::BlockNumberFor,
};
use lite_json::json::JsonValue;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	offchain::{
		http,
		storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
		Duration,
	},
	traits::Zero,
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
};
use sp_std::vec::Vec;

#[cfg(test)]
mod tests;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
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

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// Configuration parameters

		/// A grace period after we send transaction.
		///
		/// To avoid sending too many transactions, we only attempt to send one
		/// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
		/// sending between distinct runs of this offchain worker.
		#[pallet::constant]
		type GracePeriod: Get<BlockNumberFor<Self>>;

		/// Number of blocks of cooldown after unsigned transaction is included.
		///
		/// This ensures that we only accept unsigned transactions once, every `UnsignedInterval`
		/// blocks.
		#[pallet::constant]
		type UnsignedInterval: Get<BlockNumberFor<Self>>;

		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

		/// Maximum number of prices.
		#[pallet::constant]
		type MaxPrices: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

		fn offchain_worker(block_number: BlockNumberFor<T>) {
			log::info!("Hello World from offchain workers!");

			// We can easily import `frame_system` and retrieve a block hash of the parent block.
			let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::debug!("Current block: {:?} (parent hash: {:?})", block_number, parent_hash);

			let claim: T::Hash =  parent_hash.clone(); // TODO

			let should_send = Self::choose_transaction_type(block_number);
			let res = match should_send {
				TransactionType::Signed => Self::process_connections(claim),
				TransactionType::None => Ok(()),
			};
			if let Err(e) = res {
				log::error!("Error: {}", e);
			}
		}
	}

	/// A public part of the pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn process_payments(origin: OriginFor<T>, claim: T::Hash, category: u32) -> DispatchResultWithPostInfo {
			// Retrieve sender of the transaction.
			let who = ensure_signed(origin)?;
			// Add the price to the on-chain list.
			Self::submit_payment(Some(who), claim, category);
			Ok(().into())
		}
	}

	/// Events for the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when new price is accepted to contribute to the average.
		NewAnalytics { claim: T::Hash, analytics_cid: u32, maybe_who: Option<T::AccountId> },
	}

	/// A vector of recently submitted prices.
	///
	/// This is used to calculate average price, should have bounded size.
	#[pallet::storage]
	pub(super) type Prices<T: Config> = StorageValue<_, BoundedVec<u32, T::MaxPrices>, ValueQuery>;

	/// Defines the block when next unsigned transaction will be accepted.
	///
	/// To prevent spam of unsigned (and unpaid!) transactions on the network,
	/// we only allow one transaction every `T::UnsignedInterval` blocks.
	/// This storage entry defines when new transaction is going to be accepted.
	#[pallet::storage]
	pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;
}

/// Payload used by this example crate to hold price
/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct PricePayload<Public, BlockNumber> {
	block_number: BlockNumber,
	price: u32,
	public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for PricePayload<T::Public, BlockNumberFor<T>> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

enum TransactionType {
	Signed,
	None,
}

use sp_std::prelude::*;
use sp_runtime::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: Option<String>,
    inline_data: Option<InlineData>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct RequestBody {
    contents: Vec<Content>,
}

impl<T: Config> Pallet<T> {
	/// Chooses which transaction type to send.
	///
	/// This function serves mostly to showcase `StorageValue` helper
	/// and local storage usage.
	///
	/// Returns a type of transaction that should be produced in current run.
	fn choose_transaction_type(block_number: BlockNumberFor<T>) -> TransactionType {
		/// A friendlier name for the error that is going to be returned in case we are in the grace
		/// period.
		const RECENTLY_SENT: () = ();

		// Start off by creating a reference to Local Storage value.
		// Since the local storage is common for all offchain workers, it's a good practice
		// to prepend your entry with the module name.
		let val = StorageValueRef::persistent(b"example_ocw::last_send");
	
		let res =
			val.mutate(|last_send: Result<Option<BlockNumberFor<T>>, StorageRetrievalError>| {
				match last_send {
					// If we already have a value in storage and the block number is recent enough
					// we avoid sending another transaction at this time.
					Ok(Some(block)) if block_number < block + T::GracePeriod::get() =>
						Err(RECENTLY_SENT),
					// In every other case we attempt to acquire the lock and send a transaction.
					_ => Ok(block_number),
				}
			});


		match res {
			// The value has been set correctly, which means we can safely send a transaction now.
			Ok(block_number) => {
				// We will send different transactions based on a random number.

				let transaction_type = block_number % 4u32.into();
				if transaction_type == Zero::zero() {
					TransactionType::Signed
				} else {
					TransactionType::None
				}
			},
			// We are in the grace period, we should not send a transaction this time.
			Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) => TransactionType::None,
			// We wanted to send a transaction, but failed to write the block number (acquire a
			// lock). This indicates that another offchain worker that was running concurrently
			// most likely executed the same logic and succeeded at writing to storage.
			// Thus we don't really want to send the transaction, knowing that the other run
			// already did.
			Err(MutateStorageError::ConcurrentModification(_)) => TransactionType::None,
		}
	}

	/// A helper function to fetch the price and send signed transaction.
	fn process_connections(claim: T::Hash) -> Result<(), &'static str> {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			return Err(
				"No local accounts available. Consider adding one via `author_insertKey` RPC.",
			)
		}

		let category = Self::validate_connections(claim).map_err(|_| "Failed to fetch price")?;

		let results = signer.send_signed_transaction(|_account| {
			Call::process_payments {claim, category }
		});

		for (acc, res) in &results {
			match res {
				Ok(()) => log::info!("[{:?}] Submitted price of {} cents", acc.id, category),
				Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
			}
		}

		Ok(())
	}

	pub fn send_request(encoded_image: String) -> Result<u32, http::Error> {
	
		//let json_body = serde_json::to_string(&request_body).expect("Failed to serialize request body");
	
		let json_body: &str = "test";

		let url: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro-vision:generateContent?key=my_api_key";
	
		let request = http::Request::post(url, vec![json_body]);
		
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

		let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;
	
		let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
		if response.code != 200 {
			log::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown)
		}
		let body = response.body().collect::<Vec<u8>>();

		// Create a str slice from the body.
		let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
			log::warn!("No UTF8 body");
			http::Error::Unknown
		})?;

		let category = match Self::parse_analytics(body_str) {
			Some(price) => Ok(price),
			None => {
				log::warn!("Unable to extract price from the response: {:?}", body_str);
				Err(http::Error::Unknown)
			},
		}?;
		Ok(category)
	}

	/// Fetch analytics for a given claim.
	fn process_connection_requests(claim: T::Hash) -> Result<u32, http::Error> {
		// todo
		Ok(0)
	}
	
	/// Fetch analytics for a given claim.
	fn validate_connections(claim: T::Hash) -> Result<u32, http::Error> {
		let test_image = String::from("Test Image");
		let result = Self::send_request(test_image);
		result
	}

	/// Add analytics for the video clip.
	fn submit_payment(maybe_who: Option<T::AccountId>, claim: T::Hash, analytics_cid: u32) {

		// here we are raising the NewAnalytics event
		Self::deposit_event(Event::NewAnalytics { claim, analytics_cid, maybe_who });
	}

	fn parse_analytics(anlytics_str: &str) -> Option<u32> {
		let val = lite_json::parse_json(anlytics_str);
		let category = match val.ok()? {
			JsonValue::Object(obj) => {
				let (_, v) = obj.into_iter().find(|(k, _)| k.iter().copied().eq("CATEGORY".chars()))?;
				match v {
					JsonValue::Number(number) => number,
					_ => return None,
				}
			},
			_ => return None,
		};

		Some(category.integer as u32)
	}	
}
