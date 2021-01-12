#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use codec::{Encode,Decode};
use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
	decl_error,ensure,
	dispatch,
	traits::{Currency, Get, LockIdentifier, LockableCurrency, WithdrawReasons,Randomness},
	Parameter};
use frame_system::ensure_signed;
use frame_support::dispatch::DispatchError;
use sp_std::{vec::Vec,ops::Add};
use sp_io::hashing::blake2_128;
use sp_runtime::traits::{Member,MaybeSerializeDeserialize,MaybeDisplay,AtLeast32BitUnsigned,Bounded,MaybeMallocSizeOf,One};
use sp_std::fmt::Debug;



#[derive(Encode,Decode)]
pub struct Kitty(pub [u8;16]);

#[derive(Encode,Decode)]
pub struct KittyData<KittyIndex: PartialEq>{
	pub parent_one: Option<KittyIndex>,
	pub parent_two: Option<KittyIndex>,
	pub brothers: Vec<KittyIndex>,
	pub children: Vec<KittyIndex>,
	pub breeds: Vec<KittyIndex>
}

impl <KittyIndex: PartialEq> Default for KittyData<KittyIndex> {

	fn default() -> Self {
		KittyData{
			parent_one: None,
			parent_two: None,
			brothers: vec![],
			children: vec![],
			breeds: vec![],
		}
	}
}

impl <KittyIndex: PartialEq> KittyData<KittyIndex> {
	pub fn add_brother(&mut self, kitty_id: KittyIndex) {
		self.brothers.push(kitty_id);
	}

	pub fn add_children(&mut self, kitty_id: KittyIndex) {
		self.children.push(kitty_id);
	}

	pub fn add_breeds(&mut self, kitty_id: KittyIndex) {

		for id  in self.breeds.iter() {
			if (*id == kitty_id) {
				return;
			}
		}
		self.breeds.push(kitty_id);
	}

}



#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const LOCK_ID: LockIdentifier = *b"kitty123";


pub type BalanceOf<T> =
<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type RandomnessSource: Randomness<Self::Hash>;
	type KittyIndex: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay +
	AtLeast32BitUnsigned + Default + Bounded + Copy + sp_std::hash::Hash +
	sp_std::str::FromStr + MaybeMallocSizeOf;
	type Currency: LockableCurrency<Self::AccountId>;
	type StakingCreate: Get<BalanceOf<Self>>;
}


decl_storage! {

	trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
		pub KittiesList get(fn kitties_list): map hasher(blake2_128_concat) T::AccountId => Vec<T::KittyIndex>;
		pub KittiesData get(fn kitties_data): map hasher(blake2_128_concat) T::KittyIndex => KittyData<T::KittyIndex> ;
		pub StakingData get(fn staking_data): map hasher(blake2_128_concat) T::AccountId => BalanceOf<T>;

	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId,
	 		KittyIndex = <T as Trait>::KittyIndex{
    	Created(AccountId, KittyIndex),
        Transferred(AccountId,AccountId,KittyIndex),
        Breeded(AccountId,KittyIndex,KittyIndex,KittyIndex),
    }
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
        KittiesCountOverFlow,
        InvalidKittyId,
        RequireDifferentParent,
        RequireOwner,
        KittyMustBeNotInclude,
        KittyMustBeInclude,
		InsufficientFunds,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        const Staking: BalanceOf<T> = T::StakingCreate::get();


        #[weight = 0]
        pub fn create(origin) -> dispatch::DispatchResult {

            let sender = ensure_signed(origin)?;
            let staking = T::StakingCreate::get();
            ensure!(staking <= T::Currency::free_balance(&sender), Error::<T>::InsufficientFunds);
			let mut current_staking = StakingData::<T>::get(&sender);
			current_staking += staking;
			StakingData::<T>::insert(&sender,current_staking);

			T::Currency::set_lock(
				LOCK_ID,
				&sender,
				current_staking,
				WithdrawReasons::all()
			);

            let kitty_id = Self::next_kitty_id()?;

			let dna = Self::random_value(&sender);

			let kitty = Kitty(dna);

			Self::insert_kitty(&sender,kitty_id,kitty);

			Self::deposit_event(RawEvent::Created(sender,kitty_id));
            Ok(())
        }

        #[weight = 0]
        pub fn transfer(origin, to: T::AccountId, kitty_id : T::KittyIndex) {
        	let sender = ensure_signed(origin)?;
        	// fix bug ,not check the owner of the kitty
        	let owner = KittyOwners::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
        	ensure!(sender == owner, Error::<T>::RequireOwner);

            let staking = T::StakingCreate::get();

			ensure!(staking <= T::Currency::free_balance(&to), Error::<T>::InsufficientFunds);
			let mut current_staking = StakingData::<T>::get(&to);
			current_staking += staking;
			StakingData::<T>::insert(&to,current_staking);
			T::Currency::set_lock(
				LOCK_ID,
				&to,
				current_staking,
				WithdrawReasons::all()
			);

			let mut current_staking = StakingData::<T>::get(&sender);
			current_staking -= staking;
			StakingData::<T>::insert(&sender,current_staking);
			T::Currency::set_lock(
				LOCK_ID,
				&sender,
				current_staking,
				WithdrawReasons::all()
			);

        	<KittyOwners<T>>::insert(kitty_id, to.clone());
        	Self::deposit_event(RawEvent::Transferred(sender,to,kitty_id));
        }

        #[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			ensure!(Kitties::<T>::contains_key(kitty_id_1),Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2),Error::<T>::InvalidKittyId);
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

			let staking = T::StakingCreate::get();
            ensure!(staking <= T::Currency::free_balance(&sender), Error::<T>::InsufficientFunds);
			let mut current_staking = StakingData::<T>::get(&sender);
			current_staking += staking;
			StakingData::<T>::insert(&sender,current_staking);

			T::Currency::set_lock(
				LOCK_ID,
				&sender,
				current_staking,
				WithdrawReasons::all()
			);

			let kitty_id = Self::do_breed(&sender,kitty_id_1,kitty_id_2)?;

			Self::add_children(kitty_id_1,kitty_id);
			Self::add_children(kitty_id_2,kitty_id);
			Self::add_breeds(kitty_id_1,kitty_id_2);
			Self::create_kitty_data(kitty_id,kitty_id_1,kitty_id_2);
			Self::deposit_event(RawEvent::Breeded(sender,kitty_id_1,kitty_id_2,kitty_id));
		}
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	(selector & dna1) | (!selector & dna2)
}

impl <T:Trait> Module<T> {

	fn create_kitty_data(kitty_id: T::KittyIndex, parent_id_1: T::KittyIndex, parent_id_2: T::KittyIndex) {
		let (p1,p2) = match parent_id_1 < parent_id_2 {
			true => (parent_id_1,parent_id_2),
			false => (parent_id_2,parent_id_1)
		};
		let mut data = KittyData::default();
		data.parent_one = Some(p1);
		data.parent_two = Some(p2);
		<KittiesData<T>>::insert(kitty_id,data);

	}

	fn add_breeds(kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
		// update kitty_id 1
		let mut data = match <KittiesData<T>>::contains_key(kitty_id_1) {
			true => {
				Self::kitties_data(kitty_id_1)
			},
			false => {
				KittyData::default()
			}
		};
		data.add_breeds(kitty_id_2);
		<KittiesData<T>>::insert(kitty_id_1,data);

		// update kitty_id 2
		let mut data = match <KittiesData<T>>::contains_key(kitty_id_2) {
			true => {
				Self::kitties_data(kitty_id_2)
			},
			false => {
				KittyData::default()
			}
		};
		data.add_breeds(kitty_id_1);
		<KittiesData<T>>::insert(kitty_id_2,data);
	}

	fn add_children(parent_id: T::KittyIndex, child_id: T::KittyIndex) {
		let mut data = match <KittiesData<T>>::contains_key(parent_id) {
			true => {
				Self::kitties_data(parent_id)
			},
			false => {
				KittyData::default()
			}
		};
		data.add_children(child_id);
		<KittiesData<T>>::insert(parent_id,data);
	}

	fn add_to_kitties_list(sender: &T::AccountId, kitty_id: T::KittyIndex)  -> sp_std::result::Result<(),DispatchError> {
		let list = match <KittiesList<T>>::contains_key(sender) {
			true => {
				let mut l = Self::kitties_list(sender);
				for i in l.iter() {
					ensure!(*i != kitty_id, Error::<T>::KittyMustBeNotInclude);
				}
				l.push(kitty_id);
				l
			},
			_ => {
				vec![kitty_id]
			}
		};
		if list.is_empty() {
			<KittiesList<T>>::remove(sender);
		}else {
			<KittiesList<T>>::insert(sender, list);
		}
		Ok(())
	}

	fn del_from_kitties_list(sender: &T::AccountId, kitty_id: T::KittyIndex)  -> sp_std::result::Result<(),DispatchError> {
		let exists = <KittiesList<T>>::contains_key(sender);
		ensure!(exists, Error::<T>::KittyMustBeInclude);

		let mut list = Self::kitties_list(sender);
		let mut founded: bool = false;
		for i in list.iter() {
			if (*i == kitty_id) {
				founded = true;
			}
		}
		ensure!(founded, Error::<T>::KittyMustBeInclude);
		list.retain(|&x| x != kitty_id);

		if list.is_empty() {
			<KittiesList<T>>::remove(sender);
		}else {
			<KittiesList<T>>::insert(sender, list);
		}
		Ok(())
	}

	fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex,DispatchError> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err(Error::<T>::KittiesCountOverFlow.into());
		}
		Ok(kitty_id)
	}

	fn random_value(sender: &T::AccountId) -> [u8;16] {
		let payload = (T::RandomnessSource::random_seed(),&sender,<frame_system::Module<T>>::extrinsic_index(),);
		payload.using_encoded(blake2_128)
	}

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
		<Kitties<T>>::insert(kitty_id,kitty);
		<KittiesCount<T>>::mutate(|id| *id += One::one());
		<KittyOwners<T>>::insert(kitty_id,owner);
	}

	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex,DispatchError> {
		let kitty1 = Self::kitties(kitty_id_1).unwrap();
		let kitty2 = Self::kitties(kitty_id_2).unwrap();

		let kitty_id = Self::next_kitty_id()?;
		let kitty1_dna = kitty1.0;
		let kitty2_dna = kitty2.0;
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8;16];
		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i],kitty2_dna[i],selector[i]);
		}
		Self::insert_kitty(sender,kitty_id,Kitty(new_dna));
		Ok(kitty_id)
	}

	pub fn get_kitties(who: impl sp_std::borrow::Borrow<T::AccountId>) -> Vec<T::KittyIndex> {
		let result = match <KittiesList<T>>::contains_key(who.borrow()) {
			true => {
				Self::kitties_list(who.borrow())
			},
			false => { vec![]}
		};
		result
	}

	pub fn get_kitty_data(kitty_id: T::KittyIndex) -> Option<KittyData<T::KittyIndex>> {
		match <KittiesData<T>>::contains_key(kitty_id) {
			false => None,
			true => {
				let mut data = Self::kitties_data(kitty_id);
				let data1 = if let Some(d) = data.parent_one {
					let temp = match <KittiesData<T>>::contains_key(d) {
						true => Self::kitties_data(d),
						false => KittyData::default()
					};
					temp
				}else {
					KittyData::default()
				};

				let data2 = if let Some(d) = data.parent_two {
					let temp = match <KittiesData<T>>::contains_key(d) {
						true => Self::kitties_data(d),
						false => KittyData::default()
					};
					temp
				}else {
					KittyData::default()
				};

				let mut brothers = data1.children.clone();

				for i in data2.children.iter() {
					let id = *i;
					for j in data1.children.iter() {
						if id != *j  {
							brothers.push(*i);
						}
					}
				}
				brothers.retain(|&x| x != kitty_id);
				data.brothers = brothers;
				Some(data)
			}
		}
	}
}
