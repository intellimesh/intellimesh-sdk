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
mod types;


#[frame_support::pallet]
pub mod pallet {
	use crate::types::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use frame_support::traits::{ EnsureOriginWithArg, Incrementable,};
	//use types::ConnectionDetails;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: crate::weights::WeightInfo;

		#[pallet::constant]
		type MaxLength: Get<u32>;

		type ConnectionId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;
		type PerceptId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;
		type CognitoId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;
		type CognitoStatus: Member + Parameter + MaxEncodedLen + Copy;
		type CognitoCaps: Member + Parameter + MaxEncodedLen + Copy;
		type PerceptStatus: Member + Parameter + MaxEncodedLen + Copy;
		type PerceptCaps: Member + Parameter + MaxEncodedLen + Copy;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	pub(super) type Percepts<T: Config> = StorageDoubleMap<_, Blake2_128Concat,  T::AccountId, Blake2_128Concat, T::PerceptId, PerceptDetails<T::PerceptStatus, T::PerceptCaps>>;

	#[pallet::storage]
	pub(super) type NextPerceptId<T: Config> = StorageValue<_, T::PerceptId, OptionQuery>;

	#[pallet::storage]
	pub(super) type Cognitos<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat,  T::CognitoId, CognitoDetails<T::CognitoStatus, T::CognitoCaps>>;

	#[pallet::storage]
	pub(super) type NextCognitoId<T: Config> = StorageValue<_, T::CognitoId, OptionQuery>;

	#[pallet::storage]
	pub(super) type Connections<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::PerceptId, Blake2_128Concat, T::CognitoId, 
		ConnectionDetails<T::AccountId, T::ConnectionId, BlockNumberFor<T>, T::Hash>>;

	#[pallet::storage]
	pub(super) type NextConnectionId<T: Config> = StorageValue<_, T::ConnectionId, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a claim has been created.
		PerceptCreated { who: T::AccountId, percept_id: T::PerceptId },
		/// Event emitted when a claim is revoked by the owner.
		PerceptRevoked { who: T::AccountId, percept_id: T::PerceptId },
		
		CognitoCreated { who: T::AccountId, cognito_id: T::CognitoId },
		CognitoRevoked { who: T::AccountId, cognito_id: T::CognitoId },

		ConnectionCreated { who: T::AccountId, connection_id: T::ConnectionId },
		ConnectionRevoked { who: T::AccountId, connection_id: T::ConnectionId },
		ConnectionUpdated { who: T::AccountId, connection_id: T::ConnectionId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The claim already exists.
		AlreadyCreated,
		/// The claim does not exist, so it cannot be revoked.
		NoSuchNode,
		/// The claim is owned by another account, so caller can't revoke it.
		NotOwner,
		/// A name is too long.
		TooLong,		
		/// Unknown Percept
		UnknownPercept,
		/// Unknown Connection
		UnknownConnection,
		// Value does not exist
		DoesNotExist,
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
		pub fn create_percept(origin: OriginFor<T>, status: T::PerceptStatus, caps: T::PerceptCaps) -> DispatchResult {

		  let percept_id = NextPerceptId::<T>::get()
			.or(T::PerceptId::initial_value())
			.ok_or(Error::<T>::UnknownPercept)?;

		  // Check that the extrinsic was signed and get the signer.
		  // This function will return an error if the extrinsic is not signed.
		  let sender = ensure_signed(origin)?;
	   
		  // Verify that the specified claim has not already been stored.
		  ensure!(!Percepts::<T>::contains_key(sender.clone(), percept_id), Error::<T>::AlreadyCreated);
	   
		  // Get the block number from the FRAME System pallet.
		  let current_block = <frame_system::Pallet<T>>::block_number();
	   
		  // Store the claim with the sender and block number.
		  Percepts::<T>::insert(sender.clone(), percept_id, PerceptDetails{status: status, caps: caps});
	   
		  // Emit an event that the claim was created.
		  Self::deposit_event(Event::PerceptCreated { who: sender, percept_id });
	   
		  Ok(())
		}
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(1)]
		pub fn revoke_percept(origin: OriginFor<T>, percept_id: T::PerceptId) -> DispatchResult {
		  // Check that the extrinsic was signed and get the signer.
		  // This function will return an error if the extrinsic is not signed.
		  let sender = ensure_signed(origin)?;
	   
		  // Get owner of the claim, if none return an error.
		  let percept_details = Percepts::<T>::get(sender.clone(), percept_id).ok_or(Error::<T>::NoSuchNode)?;
	   	   
		  // Remove claim from storage.
		  Percepts::<T>::remove(sender.clone(), percept_id);
	   
		  // Emit an event that the claim was erased.
		  Self::deposit_event(Event::PerceptRevoked { who: sender, percept_id });

		  let next_percept_id = percept_id.increment();
		  NextPerceptId::<T>::set(next_percept_id);

		  Ok(())
		}	   

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(3)]
		pub fn create_cognito(origin: OriginFor<T>, status: T::CognitoStatus, caps: T::CognitoCaps) -> DispatchResult {
		
		  let cognito_id: <T as Config>::CognitoId = NextCognitoId::<T>::get()
			.or(T::CognitoId::initial_value())
			.ok_or(Error::<T>::UnknownPercept)?;

		  let sender = ensure_signed(origin)?;
		  ensure!(!Cognitos::<T>::contains_key(sender.clone(), cognito_id), Error::<T>::AlreadyCreated);	   
		  let current_block = <frame_system::Pallet<T>>::block_number();
		  Cognitos::<T>::insert(sender.clone(), cognito_id, CognitoDetails{status: status, caps: caps});
	
		  // Emit an event that the claim was created.
		  Self::deposit_event(Event::CognitoCreated { who: sender, cognito_id });

		  let next_cognito_id = cognito_id.increment();
		  NextCognitoId::<T>::set(next_cognito_id);

		  Ok(())
		}
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(4)]
		pub fn revoke_cognito(origin: OriginFor<T>, cognito_id: T::CognitoId) -> DispatchResult {
		  let sender = ensure_signed(origin)?;
		  let cognito_details = Cognitos::<T>::get(sender.clone(), cognito_id).ok_or(Error::<T>::NoSuchNode)?;
		  Cognitos::<T>::remove(sender.clone(), &cognito_id);
		  Self::deposit_event(Event::CognitoRevoked { who: sender, cognito_id });
		  Ok(())
		}	   

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(5)]
		pub fn create_connection(origin: OriginFor<T>, percept_id: T::PerceptId, cognito_id: T::CognitoId) -> DispatchResult {
		
			let connection_id: <T as Config>::ConnectionId = NextConnectionId::<T>::get()
				.or(T::ConnectionId::initial_value())
				.ok_or(Error::<T>::UnknownPercept)?;

		  let sender = ensure_signed(origin)?;
		  ensure!(!Connections::<T>::contains_key(percept_id, cognito_id), Error::<T>::AlreadyCreated);	   
		  let current_block = <frame_system::Pallet<T>>::block_number();
		  Connections::<T>::insert(&percept_id, cognito_id, 
			ConnectionDetails{owner: sender.clone(), connection: connection_id, start_block: current_block, proof: T::Hash::default()});
	
		  // Emit an event that the claim was created.
		  Self::deposit_event(Event::ConnectionCreated { who: sender, connection_id });

		  let next_connection_id = connection_id.increment();
		  NextConnectionId::<T>::set(next_connection_id);

		  Ok(())
		}
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(6)]
		pub fn revoke_connection(origin: OriginFor<T>, percept_id: T::PerceptId, cognito_id: T::CognitoId) -> DispatchResult {
		  let sender = ensure_signed(origin)?;
		  let connection_details = Connections::<T>::get(percept_id, cognito_id).ok_or(Error::<T>::NoSuchNode)?;
		  ensure!(sender == connection_details.owner, Error::<T>::NotOwner);
		  Connections::<T>::remove(&percept_id, cognito_id);
		  Self::deposit_event(Event::ConnectionRevoked { who: sender, connection_id: connection_details.connection });
		  Ok(())
		}
		
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(7)]
		pub fn update_connection_proof(origin: OriginFor<T>, percept_id: T::PerceptId, cognito_id: T::CognitoId, proof: T::Hash) -> DispatchResult {
		  //let mut connection_id = T::ConnectionId::initial_value().unwrap();
		  let sender = ensure_signed(origin)?;
		  
		  ensure!(Connections::<T>::contains_key(percept_id, cognito_id), Error::<T>::DoesNotExist);	   
		  Connections::<T>::try_mutate(percept_id, cognito_id, |maybe_details| {
			let details = maybe_details.as_mut().ok_or(Error::<T>::UnknownConnection)?;
			ensure!(sender == details.owner, Error::<T>::NotOwner);

			details.proof = proof;

			// Emit an event that the connection was created.
		  	Self::deposit_event(Event::ConnectionUpdated { who: sender, connection_id: details.connection });

			Ok::<(), Error::<T>>(())
		  })?;
		  Ok(())
		}
	}

  // Helper functions
  impl<T: Config> Pallet<T> {
	pub fn get_connection_details(origin: OriginFor<T>, percept_id: T::PerceptId, cognito_id: T::CognitoId) -> 
			Result<ConnectionDetails<T::AccountId, T::ConnectionId, BlockNumberFor<T>, T::Hash>, Error<T>> {
		let details = Connections::<T>::get(percept_id, cognito_id).ok_or(Error::<T>::NoSuchNode)?;
		Ok(details.clone())
	}

	pub fn get_percept_details(account_id: T::AccountId, percept_id: T::PerceptId) -> 
			Result<PerceptDetails<T::PerceptStatus, T::PerceptCaps>, Error<T>> {
		let details = Percepts::<T>::get(account_id, percept_id).ok_or(Error::<T>::NoSuchNode)?;
		Ok(details.clone())
	}

	pub fn get_cognito_details(account_id: T::AccountId, cognito_id: T::CognitoId) -> 
			Result<CognitoDetails<T::CognitoStatus, T::CognitoCaps>, Error<T>> {
		let details = Cognitos::<T>::get(account_id, cognito_id).ok_or(Error::<T>::NoSuchNode)?;
		Ok(details.clone())
	}

	pub(crate) fn get_percepts(account_id: T::AccountId) -> Result<Vec<T::PerceptId>, Error<T>> {
	  let results = Percepts::<T>::iter_prefix(&account_id).into_iter().map(
			|(percept_id, _details)|  { percept_id }
	  ).collect();
	  Ok(results)
	}

	pub(crate) fn get_cognitos(account_id: T::AccountId) -> Result<Vec<T::CognitoId>, Error<T>> {
  	  let results = Cognitos::<T>::iter_prefix(&account_id).into_iter().map(
			|(cognito_id, _details)|  { cognito_id }
	  )  .collect();
	  Ok(results)
	}
	  
  }
}
