use support::StorageMap;
use support::StorageValue;
use support::dispatch::Result;
use support::{decl_module, decl_storage, decl_event};
use support::{ensure, fail};
// use system::ensure_signed;
use system::{ensure_signed, ensure_root, ensure_inherent};
use runtime_primitives::traits::{As, Hash, Zero};

#[cfg(feature = "std")]
use serde_derive::{Serialize, Deserialize};
use parity_codec::{Encode, Decode};

use support::traits::{Currency, ReservableCurrency, OnDilution, OnUnbalanced, Imbalance};
use runtime_io::print;


// type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
// type PositiveImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
// type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;


pub struct Token;

// impl<AccountId, Balance, PositiveImbalance, NegativeImbalance> Currency<AccountId> for Akro {
// 	type Balance = Self::Balance;
// 	type PositiveImbalance = Self::PositiveImbalance;
// 	type NegativeImbalance = Self::NegativeImbalance;
// }


/// The module's configuration trait.
pub trait Trait: balances::Trait {
	// type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
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
		where Balance = <T as balances::Trait>::Balance ,
		AccountId = <T as system::Trait>::AccountId,
		{

		Stake(Balance, AccountId),
		Withdraw(Balance, AccountId),
	}
);
