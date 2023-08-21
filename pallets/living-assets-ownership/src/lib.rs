#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use sp_core::{H160, U256};
use sp_std::vec::Vec;
use parity_scale_codec::alloc::string::ToString;

mod functions;
pub mod traits;

#[frame_support::pallet]
pub mod pallet {
	use crate::functions::convert_asset_id_to_owner;

	use super::*;
	use frame_support::{
		pallet_prelude::{OptionQuery, ValueQuery, *},
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;

	/// Collection id type
	pub type CollectionId = u64;

	/// Base URI type
	pub type BaseURI<T> = BoundedVec<u8, <T as Config>::BaseURILimit>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Specifies the advised maximum length for a Base URI.
		//s
		/// The URI standard (RFC 3986) doesn't dictates a limit for the length of URIs.
		/// However it seems the max supported length in browsers is 2,048 characters.
		///
		/// The base should be capped at 2,015 characters in length. This ensures room for
		/// the token URI formation, where it combines `BaseURILimit`, a `'/'`, and a `tokenID`
		/// (which takes up 33 characters).
		#[pallet::constant]
		type BaseURILimit: Get<u32>;
	}

	/// Collection counter
	#[pallet::storage]
	#[pallet::getter(fn collection_counter)]
	pub(super) type CollectionCounter<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	/// Collection base URI
	#[pallet::storage]
	#[pallet::getter(fn collection_base_uri)]
	pub(super) type CollectionBaseURI<T: Config> =
		StorageMap<_, Blake2_128Concat, CollectionId, BaseURI<T>, OptionQuery>;

	/// Pallet events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Collection created
		/// parameters. [collection_id, who]
		CollectionCreated { collection_id: CollectionId, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// Collection id overflow
		CollectionIdOverflow,
		/// Unexistent collection
		UnexistentCollection,
	}

	impl<T: Config> AsRef<[u8]> for Error<T> {
		fn as_ref(&self) -> &[u8] {
			match self {
				Error::__Ignore(_, _) => b"__Ignore",
				Error::CollectionIdOverflow => b"CollectionIdOverflow",
				Error::UnexistentCollection => b"UnexistentCollection",
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())] // TODO set proper weight
		pub fn create_collection(origin: OriginFor<T>, base_uri: BaseURI<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match Self::do_create_collection(who, base_uri) {
				Ok(_) => Ok(()),
				Err(err) => Err(err.into()),
			}
		}
	}

	impl<T: Config> traits::CollectionManager for Pallet<T> {
		type Error = Error<T>;
		type AccountId = T::AccountId;
		type BaseURI = BaseURI<T>;

		fn base_uri(collection_id: CollectionId) -> Option<Self::BaseURI> {
			CollectionBaseURI::<T>::get(collection_id)
		}

		fn create_collection(
			owner: T::AccountId,
			base_uri: Self::BaseURI,
		) -> Result<CollectionId, Self::Error> {
			Self::do_create_collection(owner, base_uri)
		}
	}

	impl<T: Config> traits::Erc721 for Pallet<T> {
		type Error = Error<T>;

		fn owner_of(collection_id: CollectionId, asset_id: U256) -> Result<H160, Self::Error> {
			match CollectionBaseURI::<T>::get(collection_id) {
				Some(_) => Ok(convert_asset_id_to_owner(asset_id)),
				None => Err(Error::UnexistentCollection),
			}
		}

		fn token_uri(collection_id: CollectionId, asset_id: U256) -> Result<Vec<u8>, Self::Error> {
			let base_uri = match CollectionBaseURI::<T>::get(collection_id) {
				Some(base_uri) => base_uri,
				None => return Err(Error::UnexistentCollection),
			};
			// contatenate base_uri with asset_id
			let mut token_uri = base_uri.to_vec();
			token_uri.push(b'/');
			token_uri.extend_from_slice(&asset_id.to_string().as_bytes());
			Ok(token_uri)
		}
	}
}

/// `ASSET_PRECOMPILE_ADDRESS_PREFIX` is a predefined prefix used to identify collection addresses.
///
/// All addresses that start with this prefix are considered as collection addresses.
/// Since `CollectionId` is represented as a `u64`, it leaves these bits free to be
/// utilized for such a prefix.
///
/// Usage of this prefix provides a consistent and recognizable pattern for distinguishing
/// collection addresses from other types of addresses in the system.
pub const ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[0xff; 12];

/// Enum representing possible errors related to collections.
#[derive(Debug, PartialEq)]
pub enum CollectionError {
	/// Error indicating that the provided address does not have the correct prefix.
	InvalidPrefix,
}

/// Converts a `CollectionId` into an `H160` address format.
///
/// This function takes the given `CollectionId`, which is assumed to be a `u64`,
/// and maps it into an `H160` address, prepending it with the `ASSET_PRECOMPILE_ADDRESS_PREFIX`.
///
/// # Arguments
///
/// * `collection_id`: The ID of the collection to be converted.
///
/// # Returns
///
/// * An `H160` representation of the collection ID.
pub fn collection_id_to_address(collection_id: CollectionId) -> H160 {
	let mut bytes = [0u8; 20];
	bytes[12..20].copy_from_slice(&collection_id.to_be_bytes());
	for (i, byte) in ASSET_PRECOMPILE_ADDRESS_PREFIX.iter().enumerate() {
		bytes[i] = *byte;
	}
	H160(bytes)
}

/// Converts an `H160` address into a `CollectionId` format.
///
/// This function takes the given `H160` address, checks for the correct prefix, and extracts
/// the `CollectionId` from it. If the prefix is incorrect, it returns a `CollectionError::InvalidPrefix` error.
///
/// # Arguments
///
/// * `address`: The `H160` address to be converted.
///
/// # Returns
///
/// * A `Result` which is either the `CollectionId` or an error indicating the address is invalid.
pub fn address_to_collection_id(address: H160) -> Result<CollectionId, CollectionError> {
	if &address.0[0..12] != ASSET_PRECOMPILE_ADDRESS_PREFIX {
		return Err(CollectionError::InvalidPrefix);
	}
	let id_bytes: [u8; 8] = address.0[12..].try_into().unwrap();
	Ok(CollectionId::from_be_bytes(id_bytes))
}

/// Checks if a given `H160` address is a collection address.
///
/// This function examines the prefix of the given `H160` address to determine if it is a
/// collection address, based on the `ASSET_PRECOMPILE_ADDRESS_PREFIX`.
///
/// # Arguments
///
/// * `address`: The `H160` address to be checked.
///
/// # Returns
///
/// * A boolean indicating if the address is a collection address.
pub fn is_collection_address(address: H160) -> bool {
	&address.to_fixed_bytes()[0..12] == ASSET_PRECOMPILE_ADDRESS_PREFIX
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
