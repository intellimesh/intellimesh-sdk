#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>

/// 
/// https://docs.substrate.io/tutorials/build-application-logic/use-macros-in-a-custom-pallet/
/// 

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: crate::weights::WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	pub(super) type Claims<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, BlockNumberFor<T>)>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a claim has been created.
		ClaimCreated { who: T::AccountId, claim: T::Hash },
		/// Event emitted when a claim is revoked by the owner.
		ClaimRevoked { who: T::AccountId, claim: T::Hash },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The claim already exists.
		AlreadyClaimed,
		/// The claim does not exist, so it cannot be revoked.
		NoSuchClaim,
		/// The claim is owned by another account, so caller can't revoke it.
		NotClaimOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
		  // Check that the extrinsic was signed and get the signer.
		  // This function will return an error if the extrinsic is not signed.
		  let sender = ensure_signed(origin)?;
	   
		  // Verify that the specified claim has not already been stored.
		  ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);
	   
		  // Get the block number from the FRAME System pallet.
		  let current_block = <frame_system::Pallet<T>>::block_number();
	   
		  // Store the claim with the sender and block number.
		  Claims::<T>::insert(&claim, (&sender, current_block));
	   
		  // Emit an event that the claim was created.
		  Self::deposit_event(Event::ClaimCreated { who: sender, claim });
	   
		  Ok(())
		}
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(1)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
		  // Check that the extrinsic was signed and get the signer.
		  // This function will return an error if the extrinsic is not signed.
		  let sender = ensure_signed(origin)?;
	   
		  // Get owner of the claim, if none return an error.
		  let (owner, _) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;
	   
		  // Verify that sender of the current call is the claim owner.
		  ensure!(sender == owner, Error::<T>::NotClaimOwner);
	   
		  // Remove claim from storage.
		  Claims::<T>::remove(&claim);
	   
		  // Emit an event that the claim was erased.
		  Self::deposit_event(Event::ClaimRevoked { who: sender, claim });
		  Ok(())
		}	   
	}
}
