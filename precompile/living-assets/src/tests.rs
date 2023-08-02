use super::*;
use helpers::*;
use sp_core::H160;
use sp_runtime::DispatchResult;

#[test]
fn check_selectors() {
	assert_eq!(Action::CreateCollection as u32, 0x1EAF2516);
	assert_eq!(Action::OwnerOfCollection as u32, 0xFB34AE53);
}

#[test]
fn test_directly() {
	define_precompile_mock!(Ok(()), Some(H160::zero()));

	let create_collection_0 = "1eaf25160000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b7469c43535c826e29c30d25a9f3a035759cf132";
	let mut handle = create_mock_handle(create_collection_0, 0);
	let result = PrecompileMock::execute(&mut handle);
	assert!(result.is_ok());
}

mod helpers {
	use evm::Context;
	use pallet_evm_test_vector_support::MockHandle;

	/// Macro to define a precompile mock for testing.
	///
	/// This macro creates mock implementations of the `LivingAssetsOwnership` trait,
	/// allowing you to test how your code interacts with the precompiled contracts.
	///
	/// # Example
	///
	/// ```
	/// define_precompile_mock!(Ok(()), Some(H160::zero()));
	/// ```
	#[macro_export]
	macro_rules! define_precompile_mock {
		($create_collection_result:expr, $owner_of_collection_result:expr) => {
			type AccountId = H160;
			type CollectionId = u64;
			type AddressMapping = pallet_evm::IdentityAddressMapping;

			struct CollectionManagerMock;

			impl pallet_living_assets_ownership::LivingAssetsOwnership<AccountId, CollectionId>
				for CollectionManagerMock
			{
				fn create_collection(
					_collection_id: CollectionId,
					_who: AccountId,
				) -> DispatchResult {
					$create_collection_result
				}

				fn owner_of_collection(_collection_id: CollectionId) -> Option<AccountId> {
					$owner_of_collection_result
				}
			}

			type PrecompileMock = LivingAssetsOwnershipPrecompile<
				AddressMapping,
				AccountId,
				CollectionId,
				CollectionManagerMock,
			>;
		};
	}

	/// Create a mock handle for testing precompiled contracts.
	///
	/// This function takes an input string representing the data to be sent to the precompiled contract
	/// and a cost value, returning a `MockHandle` that can be used for testing.
	///
	/// # Arguments
	///
	/// * `input` - The input data as a hexadecimal string.
	/// * `cost` - A cost value as u64.
	///
	/// # Example
	///
	/// ```
	/// let handle = create_mock_handle("68656c6c6f", 0);
	/// ```
	pub fn create_mock_handle(input: &str, cost: u64) -> MockHandle {
		let i: Vec<u8> = hex::decode(input).expect("todo");

		let context: Context = Context {
			address: Default::default(),
			caller: Default::default(),
			apparent_value: From::from(0),
		};

		MockHandle::new(i, Some(cost), context)
	}
}
