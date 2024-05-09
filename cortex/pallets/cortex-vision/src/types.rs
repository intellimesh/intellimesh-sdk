use codec::{ Encode, Decode, MaxEncodedLen };
use frame_support::sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct ConnectionDetails<AccountId, ConnectionId, BlockNumber> {
	/// The owner of this item.
	pub owner: AccountId,
	pub connection: ConnectionId,
	pub start_block: BlockNumber,
}

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct PerceptDetails<AccountId> {
	/// The owner of this item.
	pub owner: AccountId,
}

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct CognitoDetails<AccountId> {
	/// The owner of this item.
	pub owner: AccountId,
}
