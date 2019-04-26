use crate::Balances;
use support::StorageMap;
use support::StorageValue;
use support::dispatch::Result;
use support::{decl_module, decl_storage, decl_event};
use support::{ensure, fail};
// use system::ensure_signed;
use system::{ensure_signed, ensure_root, ensure_inherent};
use runtime_primitives::traits::{As, Hash, Zero};
use assets::*;

#[cfg(feature = "std")]
use serde_derive::{Serialize, Deserialize};
use parity_codec::{Encode, Decode};

use support::traits::{Currency, ReservableCurrency, OnDilution, OnUnbalanced, Imbalance};
use runtime_io::print;


// pub trait Trait: balances::Trait where Self: Currency<Self::AccountId> {
// pub trait Trait: balances::Trait where Self: Currency<<Self as system::Trait>::AccountId> {
pub trait Trait: balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


decl_storage! {
	trait Store for Module<T: Trait> as Akt {
		Stake get(value): map T::AccountId => T::Balance;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

	}
}


decl_event!(
	pub enum Event<T>
		where Balance = <T as balances::Trait>::Balance,
		      AccountId = <T as system::Trait>::AccountId,
	{
		Issued(u16, AccountId, u64),
		Stake(Balance, AccountId),
		Withdraw(Balance, AccountId),
	}
);
