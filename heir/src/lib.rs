#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{Currency, ReservableCurrency},
	RuntimeDebug,
};
use scale_info::TypeInfo;

pub use pallet::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct WillConfig<BlockNumber, Balance, Heir> {
	delay_period: BlockNumber,
	deposit: Balance,
	heir: Heir,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ActiveWillConfig<BlockNumber, Balance, Index> {
	created: BlockNumber,
	deposit: Balance,
	nonce: Index,
	ready: bool,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::{GetDispatchInfo, PostDispatchInfo},
		pallet_prelude::*,
		Twox64Concat,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{Dispatchable, Zero};

	use crate::BalanceOf;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>;

		/// The amount of currency to create a living will
		#[pallet::constant]
		type ConfigDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	pub type Wills<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		WillConfig<T::BlockNumber, BalanceOf<T>, T::AccountId>,
	>;

	#[pallet::storage]
	pub type ActiveWills<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		T::AccountId,
		ActiveWillConfig<T::BlockNumber, BalanceOf<T>, T::Index>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LivingWillCreated { who: T::AccountId, heir: T::AccountId },
		LivingWillRemoved { who: T::AccountId, heir: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		WillNotExist,
		NotHeir,
		WillNotInitiated,
		ActivateNotReady,
		AccountIsAlive,
		PerformNotReady,
		BalanceNotEmpty,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn create_living_will(
			origin: OriginFor<T>,
			heir: T::AccountId,
			delay: T::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			T::Currency::reserve(&sender, T::ConfigDeposit::get())?;

			let will_config = WillConfig {
				delay_period: delay,
				deposit: T::ConfigDeposit::get(),
				heir: heir.clone(),
			};

			Wills::<T>::insert(&sender, will_config);

			Self::deposit_event(Event::<T>::LivingWillCreated { who: sender, heir });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn delete_living_will(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let will_config = Wills::<T>::get(&sender).ok_or(Error::<T>::WillNotExist)?;

			T::Currency::unreserve(&sender, will_config.deposit);

			Wills::<T>::remove(&sender);

			Self::deposit_event(Event::<T>::LivingWillRemoved {
				who: sender,
				heir: will_config.heir,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn initiate_will(origin: OriginFor<T>, testator: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let will_config = Wills::<T>::get(&testator).ok_or(Error::<T>::WillNotExist)?;
			ensure!(sender == will_config.heir, Error::<T>::NotHeir);

			T::Currency::reserve(&sender, T::ConfigDeposit::get())?;

			let active_will_config = ActiveWillConfig {
				created: <frame_system::Pallet<T>>::block_number(),
				deposit: T::ConfigDeposit::get(),
				nonce: <frame_system::Pallet<T>>::account_nonce(&testator),
				ready: false,
			};

			ActiveWills::<T>::insert(&testator, &sender, active_will_config);

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn activate_will(origin: OriginFor<T>, testator: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let will_config = Wills::<T>::get(&testator).ok_or(Error::<T>::WillNotExist)?;
			ensure!(sender == will_config.heir, Error::<T>::NotHeir);

			let mut active_will_config =
				ActiveWills::<T>::get(&testator, &sender).ok_or(Error::<T>::WillNotInitiated)?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			ensure!(
				current_block > active_will_config.created + will_config.delay_period,
				Error::<T>::ActivateNotReady
			);
			let current_nonce = <frame_system::Pallet<T>>::account_nonce(&testator);
			ensure!(current_nonce == active_will_config.nonce, Error::<T>::AccountIsAlive);

			active_will_config.ready = true;
			ActiveWills::<T>::insert(&testator, &sender, active_will_config);

			Wills::<T>::remove(&testator);
			T::Currency::unreserve(&testator, will_config.deposit);

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn perform_will(
			origin: OriginFor<T>,
			testator: T::AccountId,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let active_will_config =
				ActiveWills::<T>::get(&testator, &sender).ok_or(Error::<T>::WillNotInitiated)?;
			ensure!(active_will_config.ready, Error::<T>::PerformNotReady);

			call.dispatch(frame_system::RawOrigin::Signed(testator).into())
				.map(|_| ())
				.map_err(|e| e.error)
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn close_will(origin: OriginFor<T>, testator: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let active_will_config =
				ActiveWills::<T>::get(&testator, &sender).ok_or(Error::<T>::WillNotInitiated)?;

			let balance = T::Currency::total_balance(&testator);
			ensure!(balance == Zero::zero(), Error::<T>::BalanceNotEmpty);

			T::Currency::unreserve(&sender, active_will_config.deposit);
			ActiveWills::<T>::remove(&testator, &sender);

			Ok(())
		}
	}
}
