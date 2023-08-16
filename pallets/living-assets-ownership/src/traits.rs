use crate::{BaseURI, CollectionId};
use sp_core::{H160, U256};

/// The `CollectionManager` trait provides an interface for managing collections in a decentralized system.
///
/// A collection is a container for non-fungible assets, uniquely identified by a `collection_id`. The system allows
/// for the creation and ownership management of these collections.
///
/// # Methods
///
/// - `owner_of_collection`: Retrieve the owner of a specified collection.
/// - `create_collection`: Create a new collection and assign it to an owner.
pub trait CollectionManager<AccountId> {
	type Error: AsRef<[u8]>;

	/// Retrieves the base uri of the specified collection.
	///
	/// # Arguments
	///
	/// * `collection_id` - The unique identifier of the collection.
	///
	/// # Returns
	///
	/// The base URI associated with the specified collection or `None` if the collection doesn't exist.
	fn base_uri(collection_id: CollectionId) -> Option<BaseURI>;

	/// Creates a new collection and assigns it to the specified owner.
	///
	/// # Arguments
	///
	/// * `owner` - The account ID of the new collection's owner.
	///
	/// # Returns
	///
	/// A result containing the `collection_id` of the newly created collection or an error.
	fn create_collection(owner: AccountId, base_uri: BaseURI) -> Result<CollectionId, Self::Error>;
}

/// The `Erc721` trait provides an interface for handling ERC721 tokens in a blockchain environment.
///
/// ERC721 tokens are a standard for representing ownership of unique items on the Ethereum blockchain.
///
/// # Methods
///
/// - `owner_of`: Retrieve the owner of a specific asset within a collection.
pub trait Erc721 {
	type Error: AsRef<[u8]>;

	/// Retrieves the owner of a specific asset within the specified collection.
	///
	/// # Arguments
	///
	/// * `collection_id` - The unique identifier for the collection.
	/// * `asset_id` - The unique identifier for the asset within the collection.
	///
	/// # Returns
	///
	/// The Ethereum address (`H160`) of the asset's owner or an error.
	fn owner_of(collection_id: CollectionId, asset_id: U256) -> Result<H160, Self::Error>;

	fn token_uri(collection_id: CollectionId, asset_id: U256) -> Result<Vec<u8>, Self::Error>;
}
