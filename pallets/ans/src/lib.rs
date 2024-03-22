#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, storage::child::exists};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The minimum length a name may be.
		#[pallet::constant]
		type MinLength: Get<u32>;

		/// The maximum length a name may be.
		#[pallet::constant]
		type MaxLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A name was set.
		Reserved {
			/// The account for which the name was set.
			who: T::AccountId,
			// name being reserved
			name: Vec<u8>
		},
		Transferred {
			// the original account
			from: T::AccountId,
			// the new account
			to: T::AccountId,
			// name being reserved
			name: Vec<u8>
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A name is too short.
		TooShort,
		/// A name is too long.
		TooLong,
		/// Name is already taken.
		AlreadyReserved,
		/// Could not find existing reservation.
		NotFound,
		/// Not the owner of this reservation.
		NotOwner
	}

	/// This maps names to accounts.
	#[pallet::storage]
	#[pallet::getter(fn get_entry)]
	pub(super) type AnsOf<T: Config> =
		StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxLength>, T::AccountId>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);



	#[pallet::call]
	impl<T: Config> Pallet<T> {
	
		#[pallet::call_index(0)]
		#[pallet::weight({50_000_000})]
		pub fn reserve(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let bounded_name: BoundedVec<_, _> =
				name.clone().try_into().map_err(|_| Error::<T>::TooLong)?;
			ensure!(bounded_name.len() >= T::MinLength::get() as usize, Error::<T>::TooShort);
			ensure!(!<AnsOf<T>>::contains_key(bounded_name.clone()), Error::<T>::AlreadyReserved);

			<AnsOf<T>>::insert(&bounded_name, sender.clone() );
			Self::deposit_event(Event::<T>::Reserved { who: sender, name: name });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({50_000_000})]
		pub fn transfer_to(origin: OriginFor<T>, name: Vec<u8>, to: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			
			let bounded_name: BoundedVec<_, _> =
				name.clone().try_into().map_err(|_| Error::<T>::TooLong)?;

			// make sure that the current owner is sender.
			let existing = <AnsOf<T>>::get(bounded_name.clone());
			match existing {
				Some(current_owner) => {
					
					ensure!(sender == current_owner, Error::<T>::NotOwner);
					<AnsOf<T>>::insert(&bounded_name, to.clone());
					Self::deposit_event(Event::<T>::Transferred { from: sender, to: to.clone(), name: name });				

				},
				None => {
					return frame_support::fail!(Error::<T>::NotFound);
				}
			}

			Ok(())
		}
	}
}
