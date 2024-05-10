use codec::{ Encode, Decode, MaxEncodedLen };
use frame_support::sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct ConnectionDetails<AccountId, ConnectionId, BlockNumber, Proof> {
	/// The owner of this item.
	pub owner: AccountId,
	pub connection: ConnectionId,
	pub start_block: BlockNumber,
	pub proof: Proof,
}

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct PerceptDetails<Status, Caps> {
	pub status: Status,
	pub caps: Caps, 
}

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct CognitoDetails<Status, Caps> {
	/// The owner of this item.
	pub status: Status,
	pub caps: Caps, 
}
