#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error,ensure,dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_std::vec::Vec;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}


decl_storage! {

	trait Store for Module<T: Trait> as PoeModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
        Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        /// 在声明存证后触发该事件 [who, clain]
        ClaimCreated(AccountId, Vec<u8>),
        /// 在撤销存证后触发该事件 [who, clain]
        ClaimRevoked(AccountId, Vec<u8>),
    }
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
        /// 存证已经被声明了
        ProofAlreadyClaimed,
        /// 存证不存在，因此不能被撤销
        NoSuchProof,
        /// 存证被另一个用户声明，因此调用者不能撤销
        NotProofOwner,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// 声明存证函数
        #[weight = 10_000]
        fn create_claim(origin, proof: Vec<u8>) -> dispatch::DispatchResult {

            let sender = ensure_signed(origin)?;

            ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

            let current_block = <frame_system::Module<T>>::block_number();

            Proofs::<T>::insert(&proof, (&sender, current_block));

            Self::deposit_event(RawEvent::ClaimCreated(sender, proof));

            Ok(())
        }

        /// 撤销存证函数
        #[weight = 10_000]
        fn revoke_claim(origin, proof: Vec<u8>) -> dispatch::DispatchResult {

            let sender = ensure_signed(origin)?;

            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            let (owner, _) = Proofs::<T>::get(&proof);

            ensure!(sender == owner, Error::<T>::NotProofOwner);

            Proofs::<T>::remove(&proof);

            Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));

            Ok(())
        }

        #[weight = 10_000]
        fn transfer_claim(origin, proof: Vec<u8>, dest: T::AccountId) -> dispatch::DispatchResult  {
			let sender = ensure_signed(origin)?;

			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            let (owner, _) = Proofs::<T>::get(&proof);

            ensure!(sender == owner, Error::<T>::NotProofOwner);

            let current_block = <frame_system::Module<T>>::block_number();

            Proofs::<T>::insert(&proof, (&dest, current_block));

			Ok(())
        }
    }
}
