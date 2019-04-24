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


// type Balance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64> + MaybeSerializeDebug;
// type Balance: Member + Parameter + SimpleArithmetic + Default + Copy;
pub trait Trait: assets::Trait + balances::Trait
	where Self: assets::Trait,
	      Self: balances::Trait,
	      // <Self as balances::Trait>::Balance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64> + MaybeSerializeDebug
	{
	// type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


decl_storage! {
	trait Store for Module<T: Trait> as AKT {
		Stake get(value): map T::AccountId => T::Balance;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		pub fn issue_token_airdrop(origin) -> Result {
			const ACC0: u64 = 1;
			const ACC1: u64 = 2;
			const RECIPIENTS: [u64;2] = [ACC0, ACC1];
			const FIXED_SUPPLY: u64 = 100;

			ensure!(!RECIPIENTS.len().is_zero(), "Divide by zero error.");

			let sender = ensure_signed(origin)?;
			let asset_id = Self::next_asset_id();

			// storage:
			<NextAssetId<T>>::mutate(|asset_id| *asset_id += 1);
			<Balances<T>>::insert((asset_id, &ACC0), FIXED_SUPPLY / RECIPIENTS.len() as u64);
			<Balances<T>>::insert((asset_id, &ACC1), FIXED_SUPPLY / RECIPIENTS.len() as u64);
			<TotalSupply<T>>::insert(asset_id, FIXED_SUPPLY);

			Self::deposit_event(RawEvent::Issued(asset_id, sender, FIXED_SUPPLY));
			Ok(())
		}
	}
}


decl_event!(
	pub enum Event<T>
		where Balance = <T as balances::Trait>::Balance,
		      AccountId = <T as system::Trait>::AccountId,
	{
		Issuedw(u16, AccountId, u64),
		Stake(Balance, AccountId),
		Withdraw(Balance, AccountId),
	}
);
