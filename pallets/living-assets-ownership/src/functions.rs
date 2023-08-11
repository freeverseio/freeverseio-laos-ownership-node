//! Contains helper and utility functions of the pallet
use super::*;
use frame_support::{sp_runtime::traits::One, BoundedVec};
use sp_core::{H160, U256};

impl<T: Config> Pallet<T> {
	/// See [Self::create_collection]
	pub fn do_create_collection(
		who: T::AccountId,
		base_uri: Vec<u8>,
	) -> Result<CollectionId, Error<T>> {
		// Retrieve the current collection count to use as the new collection's ID
		let collection_id = Self::collection_counter();

		// Insert a new entry into the OwnerOfCollection map, mapping the new
		// collection's ID to the owner's account ID
		OwnerOfCollection::<T>::insert(collection_id, &who);

		let bounded_base_uri: BoundedVec<_, _> =
			base_uri.try_into().map_err(|_| Error::<T>::TooLong)?;

		CollectionBaseURI::<T>::insert(collection_id, bounded_base_uri);

		// Attempt to increment the collection counter by 1. If this operation
		// would result in an overflow, return early with an error
		let counter =
			collection_id.checked_add(One::one()).ok_or(Error::<T>::CollectionIdOverflow)?;
		CollectionCounter::<T>::put(counter);

		Self::deposit_event(Event::CollectionCreated { collection_id, who });

		Ok(collection_id)
	}
}

pub fn convert_asset_id_to_owner(value: U256) -> H160 {
	let mut bytes = [0u8; 20];
	let value_bytes: [u8; 32] = value.into();
	bytes.copy_from_slice(&value_bytes[value_bytes.len() - 20..]);
	H160::from(bytes)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn check_convert_asset_id_to_owner() {
		let value = U256::from(5);
		let expected_address = H160::from_low_u64_be(5);
		assert_eq!(convert_asset_id_to_owner(value), expected_address);
	}
}
