// use core::convert::AsMut;
use rstd::result;

// use primitives::Bytes;
// use primitives::U256;
// use primitives::convert_hash;
use runtime_primitives::traits::{Hash, Zero};

use support::StorageMap;
use support::StorageValue;
use support::dispatch::Result;
use support::{decl_module, decl_storage, decl_event};
use support::{ensure, fail};
use system::ensure_signed;
use balances::BalanceLock;

use support::traits::Currency;
use support::traits::{LockableCurrency, LockIdentifier, WithdrawReason, WithdrawReasons};

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use parity_codec::{Encode, Decode};


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct Bucket<Hash, Balance, AccountId, BlockNumber> {
	id: Hash,

	promise: Option<Promise<Hash, Balance, AccountId, BlockNumber>>,

	/// price for selling the c2fc
	price: Balance,
}

/// Describes an accepted promise
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct Promise<Hash, Balance, AccountId, BlockNumber> {
	id: Hash,

	/// initial author of `this` promise
	/// in the near future `owner` can be removed
	/// because it exists in the global mapping
	owner: AccountId,

	/// promised value to fullfill
	value: Balance,
	/// time (number of blocks)
	period: BlockNumber,
	/// time of the end of promise
	until: Option<BlockNumber>,

	/// filled value for current period
	filled: Balance,
	/// time (in blocks) when current period was started
	acception_dt: BlockNumber,
}

/// Describes not accepted "free promise"
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct FreePromise<Hash, Balance, /* Stake, */ BlockNumber> {
	id: Hash,
	/// promised value to fullfill
	value: Balance,
	/// time (number of blocks)
	period: BlockNumber,
	/// time of the end of promise
	until: Option<BlockNumber>,
}


pub trait Trait: system::Trait + balances::Trait {
	type Stake: LockableCurrency<Self::AccountId, Moment = <Self as system::Trait>::BlockNumber>;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


decl_event!(
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId,
		<T as system::Trait>::Hash,
		<T as balances::Trait>::Balance,
	{
		C2fcCreated(AccountId, Hash),
		/// OwnerSet: from, to, c2fc
		PriceSet(AccountId, Hash, Balance),
		Transferred(AccountId, AccountId, Hash),
		Bought(AccountId, AccountId, Hash, Balance),


		/// FreePromise is created.
		PromiseCreated(AccountId, Hash),
		/// FreePromise is changed.
		PromiseChanged(Hash),
		/// FreePromise is accepted by owner of c2fc.
		/// (PromiseID:Hash, BucketID:Hash)
		PromiseAccepted(Hash, Hash),
		/// (c2fc_id:Hash, promise_id:Hash, value:Balance)
		PromiseFilled(Hash, Hash, Balance),
		/// (c2fc_id:Hash, promise_id:Hash)
		PromiseFullilled(Hash, Hash),
		/// (c2fc_id:Hash, promise_id:Hash, missed_deposit:Balance)
		PromiseBreached(Hash, Hash, Balance),

		// Staking / Locking:
		Stake(Hash, AccountId, Balance),
		// Stake(Hash, AccountId, StakeBalance<Self>),
		Withdraw(Hash, AccountId, Balance),
	}
);


decl_storage! {
	trait Store for Module<T: Trait> as Cashflow {
		Buckets get(c2fc): map T::Hash => Bucket<T::Hash, T::Balance, T::AccountId, T::BlockNumber>;
		BucketOwner get(owner_of_c2fc): map T::Hash => Option<T::AccountId>;
		/// same as `AcceptedPromiseBucket` but by c2fc_id
		BucketContributor get(contributor_of_c2fc): map T::Hash => Option<T::AccountId>;

		AllBucketsArray get(c2fc_by_index): map u64 => T::Hash;
		AllBucketsCount get(all_c2fc_count): u64;
		AllBucketsIndex: map T::Hash => u64;

		OwnedBucketsArray get(c2fc_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
		OwnedBucketsCount get(owned_c2fc_count): map T::AccountId => u64;
		OwnedBucketsIndex: map T::Hash => u64;


		// free promises:
		Promises get(promise): map T::Hash => FreePromise<T::Hash, T::Balance, T::BlockNumber>;
		PromiseOwner get(owner_of_promise): map T::Hash => Option<T::AccountId>;

		FreePromisesArray get(free_promise_by_index): map u64 => T::Hash;
		FreePromisesCount get(free_promises_count): u64;
		FreePromisesIndex: map T::Hash => u64;

		OwnedPromisesArray get(promise_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
		OwnedPromisesCount get(owned_promise_count): map T::AccountId => u64;
		OwnedPromisesIndex: map T::Hash => u64;


		// accepted promises:
		AcceptedPromisesArray get(accepted_promise_by_index): map u64 => T::Hash;
		AcceptedPromisesCount get(accepted_promises_count): u64;
		AcceptedPromisesIndex: map T::Hash => u64;

		/// returns `c2fc_id` for specified `promise_id`
		AcceptedPromiseBucket get(c2fc_by_promise): map T::Hash => T::Hash;

		/// Counter total of locks
		LocksCount get(locks_count): u64;
		/// promise_id -> LockIdentifier
		LockForPromise get(lock_for_promise): map T::Hash => LockIdentifier;

		Nonce: u64;
	}
}


decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		fn create_c2fc(origin) -> Result {
			let sender = ensure_signed(origin)?;
			let nonce = <Nonce<T>>::get();
			let c2fc_id = (<system::Module<T>>::random_seed(), &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);

			let new_c2fc = Bucket {
					id: c2fc_id,
					promise: None,
					price: T::Balance::zero(),
			};

			Self::mint_c2fc(sender, c2fc_id, new_c2fc)?;

			<Nonce<T>>::mutate(|n| *n += 1);

			Ok(())
		}

		fn create_promise_until(origin, value: T::Balance, period: T::BlockNumber, until: T::BlockNumber) -> Result {
			let sender = ensure_signed(origin)?;
			let nonce = <Nonce<T>>::get();
			let promise_id = (<system::Module<T>>::random_seed(), &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);

			let new_promise = FreePromise {
				id: promise_id,
				value,
				period,
				until: if !until.is_zero() { Some(until) } else { None },
			};

			Self::mint_promise(sender, promise_id, new_promise)?;

			<Nonce<T>>::mutate(|n| *n += 1);

			Ok(())
		}

		fn create_promise(origin, value: T::Balance, period: T::BlockNumber) -> Result {
			Self::create_promise_until(origin, value, period, Zero::zero())
		}


		// TODO: fn stake_to_promise(origin, promise_id: T::Hash, amount: StakeBalance<T>) -> Result {
		fn stake_to_promise(origin, promise_id: T::Hash, amount: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Promises<T>>::exists(promise_id), "This promise does not exist");
			let owner = Self::owner_of_promise(promise_id).ok_or("No owner for this promise")?;
			ensure!(owner == sender, "You do not own this promise");

			// get data from existing promise:
			let until = if <AcceptedPromiseBucket<T>>::exists(promise_id) {
				let promise = {
					let c2fc_id = <AcceptedPromiseBucket<T>>::get(promise_id);
					let c2fc = Self::c2fc(c2fc_id);
					let promise = c2fc.promise;
					ensure!(promise.is_some(), "Bucket doesnt contains promise");
					promise.unwrap()
				};
				promise.until
			} else {
				let promise = Self::promise(promise_id);
				promise.until
			}.unwrap_or( unsafe {
				// end of the universe:
				// TODO: use (crate::)BlockNumber::max_value()
				// <T as system::Trait>::BlockNumber::from(crate::BlockNumber::max_value())
				// <T::BlockNumber as As<crate::BlockNumber>>::sa(max as crate::BlockNumber);
				// XXX:
				let max = crate::BlockNumber::max_value();
				(*(max as *const crate::BlockNumber as *const <T as system::Trait>::BlockNumber)).clone()
			});


			let reasons = WithdrawReasons::from(WithdrawReason::Reserve);

			if <LockForPromise<T>>::exists(promise_id) {
				let lock_id = Self::lock_for_promise(promise_id);
				// select lock with specified ID:
				let lock = get_lock::<T>(&sender, &lock_id);
				let lock = { // XXX: test & remove me
					let locks_all = <balances::Module<T>>::locks(&sender);
					let mut locks = locks_all.into_iter().filter_map(|l|
						if l.id == lock_id {
							Some(l)
						} else {
							None
						});
					let lock = locks.next();
					ensure!(lock.is_none(), "Lock not found");
					ensure!(locks.next().is_some(), "Incorrect length of locks with same ID. WTF?!");
					lock.unwrap()
				};

				// TODO: check overflow:
				// ensure!(T::Balance::max_value() - lock.amount >= amount, "Overflow max size of Balance!");
				// e.g. crate::BlockNumber::max_value() - <T::Balance as As<crate::Balance>>::sa(lock.amount as crate::Balance) >= <T::Balance as As<crate::Balance>>::sa(amount as crate::Balance)

				<balances::Module<T>>::extend_lock(lock_id, &sender, lock.amount + amount, until, reasons);
				// <T::Stake>::extend_lock(lock_id, &sender, lock.amount + amount, until, reasons);
			} else {
				let lock_id = Self::next_free_lock_identifier(&promise_id);

				// <T::Stake>::set_lock(lock_id, &sender, amount, until, reasons);
				<balances::Module<T>>::set_lock(lock_id, &sender, amount, until, reasons);

				// TODO: use T::Stake instead T::Balance:
				// <T::Stake>::set_lock(lock_id, &sender, amount, until, reasons);
				// <balances::Module<T::Stake>>::set_lock(lock_id, &sender, amount, until, reasons);

				// register new lock:
				<LockForPromise<T>>::insert(promise_id, lock_id);
				<LocksCount<T>>::mutate(|n| *n += 1);
			}

			Self::deposit_event(RawEvent::Stake(promise_id, sender, amount));

			Ok(())
		}

		fn withdraw_staken(origin, promise_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Promises<T>>::exists(promise_id), "This promise does not exist");

			let owner = Self::owner_of_c2fc(promise_id).ok_or("No owner for this promise")?;
			ensure!(owner == sender, "You do not own this promise");

			if <LockForPromise<T>>::exists(promise_id) {
				let lock_id = Self::lock_for_promise(promise_id);

				let lock = get_lock::<T>(&sender, &lock_id);

				if let Some(lock) = &lock {
					let now = <system::Module<T>>::block_number();
					ensure!(!<AcceptedPromiseBucket<T>>::exists(promise_id), "This promise already accepted so stake cannot withdraw.");
					ensure!(lock.until <= now, "This locked balance period isn't ended and stake cannot withdraw.");
				}

				let free = {
					lock.map(|lock| lock.amount)
				}.unwrap_or(Zero::zero());

				<balances::Module<T>>::remove_lock(lock_id, &sender);

				Self::deposit_event(RawEvent::Withdraw(promise_id, sender, free));
			}

			Ok(())
		}


		fn edit_promise(origin, promise_id: T::Hash, value: T::Balance, period: T::BlockNumber) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Promises<T>>::exists(promise_id), "This promise does not exist");

			let owner = Self::owner_of_c2fc(promise_id).ok_or("No owner for this promise")?;
			ensure!(owner == sender, "You do not own this promise");

			<Promises<T>>::mutate(promise_id, |promise|{
				promise.value = value;
				promise.period = period;
			});

			Self::deposit_event(RawEvent::PromiseChanged(promise_id));

			Ok(())
		}


		/// Accept specified free promise and add it to specified c2fc.
		/// Only owner of the c2fc can do it.
		fn accept_promise(origin, promise_id: T::Hash, c2fc_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Buckets<T>>::exists(c2fc_id), "This c2fc does not exist");
			ensure!(<Promises<T>>::exists(promise_id), "This promise does not exist");
			ensure!(!<AcceptedPromiseBucket<T>>::exists(promise_id), "This promise is already accepted");


			let c2fc_owner = Self::owner_of_c2fc(c2fc_id).ok_or("No owner for this c2fc")?;
			ensure!(c2fc_owner == sender, "You do not own this promise");

			let promise_owner = Self::owner_of_promise(promise_id).ok_or("No owner for this promise")?;
			ensure!(promise_owner != sender, "You can not accept your own promise");

			let mut c2fc = Self::c2fc(c2fc_id);
			ensure!(c2fc.promise.is_none(), "Bucket already contains another promise");

			// get current (latest) block:
			let current_block = <system::Module<T>>::block_number();

			let free_promise = Self::promise(promise_id);
			let promise = Promise {
				id: free_promise.id,
				// in the near future `owner` can be removed
				owner: promise_owner.clone(),
				value: free_promise.value,
				period: free_promise.period,
				until: free_promise.until,
				acception_dt: current_block,
				filled: T::Balance::zero(),
			};

			c2fc.promise = Some(promise);
			<Buckets<T>>::insert(c2fc_id, c2fc);
			<AcceptedPromiseBucket<T>>::insert(promise_id, c2fc_id);

			// incrmnt the counter & push to maps:
			{
				let accepted_promises_count = Self::accepted_promises_count();
				let new_accepted_promises_count = accepted_promises_count
					.checked_add(1)
					.ok_or("Overflow adding a new promise to total supply")?;

				<BucketContributor<T>>::insert(c2fc_id, promise_owner);

				<AcceptedPromisesArray<T>>::insert(accepted_promises_count, promise_id);
				<AcceptedPromisesCount<T>>::put(new_accepted_promises_count);
				<AcceptedPromisesIndex<T>>::insert(promise_id, accepted_promises_count);
			}

			<Nonce<T>>::mutate(|n| *n += 1);

			Self::deposit_event(RawEvent::PromiseAccepted(promise_id, c2fc_id));

			Ok(())
		}


		// selling & trasfering a c2fc //

		fn set_price(origin, c2fc_id: T::Hash, new_price: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Buckets<T>>::exists(c2fc_id), "This c2fc does not exist");

			let owner = Self::owner_of_c2fc(c2fc_id).ok_or("No owner for this c2fc")?;
			ensure!(owner == sender, "You do not own this c2fc");

			let mut c2fc = Self::c2fc(c2fc_id);
			c2fc.price = new_price;

			<Buckets<T>>::insert(c2fc_id, c2fc);

			Self::deposit_event(RawEvent::PriceSet(sender, c2fc_id, new_price));

			Ok(())
		}

		fn transfer(origin, to: T::AccountId, c2fc_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of_c2fc(c2fc_id).ok_or("No owner for this c2fc")?;
			ensure!(owner == sender, "You do not own this c2fc");

			Self::transfer_from(sender, to, c2fc_id)?;

			Ok(())
		}

		fn buy_c2fc(origin, c2fc_id: T::Hash, max_price: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Buckets<T>>::exists(c2fc_id), "This c2fc does not exist");

			let owner = Self::owner_of_c2fc(c2fc_id).ok_or("No owner for this c2fc")?;
			ensure!(owner != sender, "You can't buy your own c2fc");

			let mut c2fc = Self::c2fc(c2fc_id);

			let c2fc_price = c2fc.price;
			ensure!(!c2fc_price.is_zero(), "The c2fc you want to buy is not for sale");
			ensure!(c2fc_price <= max_price, "The c2fc you want to buy costs more than your max price");

			Self::transfer_money(&sender, &owner, c2fc_price)?;
			Self::transfer_from(owner.clone(), sender.clone(), c2fc_id)?;

			c2fc.price = T::Balance::zero();
			<Buckets<T>>::insert(c2fc_id, c2fc);

			Self::deposit_event(RawEvent::Bought(sender, owner, c2fc_id, c2fc_price));

			Ok(())
		}


		// do/fill the promises //

		fn fill_c2fc(origin, c2fc_id: T::Hash, deposit: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Buckets<T>>::exists(c2fc_id), "This c2fc does not exist");

			let owner = Self::owner_of_c2fc(c2fc_id).ok_or("No owner for this c2fc")?;
			ensure!(owner != sender, "You can't fill your own c2fc");

			let mut c2fc = Self::c2fc(c2fc_id);
			ensure!(c2fc.promise.is_some(), "This c2fc does not contains promise");


			if let Some(ref mut promise) = c2fc.promise {
				let promise_id = promise.id;

				ensure!(!promise.value.is_zero(), "The promise in the c2fc you want to fill is invalid");
				ensure!(promise.filled <= promise.value, "The c2fc you want to fill is already fullfilled");

				Self::transfer_money(&sender, &owner, deposit)?;

				promise.filled = deposit + promise.filled;

				Self::deposit_event(RawEvent::PromiseFilled(c2fc_id, promise_id, deposit));

				if promise.filled >= promise.value {
					Self::deposit_event(RawEvent::PromiseFullilled(c2fc_id, promise_id));
				}
			}

			// re-store the c2fc
			<Buckets<T>>::insert(c2fc_id, c2fc);

			Ok(())
		}

		fn fullfill_c2fc(origin, c2fc_id: T::Hash) -> Result {
			let deposit = {
				ensure!(<Buckets<T>>::exists(c2fc_id), "This c2fc does not exist");
				let c2fc = Self::c2fc(c2fc_id);
				let promise = &c2fc.promise.ok_or("This c2fc doesnt contains an accepted promise")?;
				let deposit = promise.filled - promise.value;
				deposit
			};

			Self::fill_c2fc(origin, c2fc_id, deposit)
		}



		/// Check the breach of promise at end of the each block.
		/// Simple timer here.
		fn on_finalize(n: T::BlockNumber) {
			let accepted_promises_count = Self::accepted_promises_count();

			for i in 0..accepted_promises_count {
				let promise_id = Self::accepted_promise_by_index(i);
				let c2fc_id = Self::c2fc_by_promise(promise_id);

				if <Buckets<T>>::exists(c2fc_id) {
					let c2fc = Self::c2fc(c2fc_id);
					// skip if c2fc doesn't contains a promise
					if let Some(promise) = &c2fc.promise {
						let lifetime = n - promise.acception_dt;
						// fix attempt to subtract with overflow:
						// let wanted_deposit = (promise.filled.into() as i128) - (promise.value.into() as i128);
						let wanted_deposit = if promise.filled > promise.value {
							Some(promise.value - promise.value)
						} else { None };

						// if (lifetime % promise.period).is_zero() && !wanted_deposit.is_zero() {
						if let (Some(wanted_deposit), true) = (wanted_deposit, (lifetime % promise.period).is_zero()) {
							// TODO: reset `promise.filled` to zero because new period starts.

							if wanted_deposit > <T::Balance>::zero() {
								// here we should to emit Event about *failed promise*.
								Self::deposit_event(RawEvent::PromiseBreached(c2fc_id, promise_id, wanted_deposit));
								// TODO: slash the stake
								// <BucketContributor<T>>::...(c2fc_id,);
							}
						}
					}
				}
			}
		}
	}
}


// private & utils //

fn get_lock<T: Trait>(
	who: &T::AccountId,
	lock_id: &LockIdentifier,
) -> Option<BalanceLock<T::Balance, T::BlockNumber>> {
	let locks_all = <balances::Module<T>>::locks(who);
	let mut locks = locks_all
		.into_iter()
		.filter_map(|l| if &l.id == lock_id { Some(l) } else { None });
	locks.next()
}


impl<T: Trait> Module<T> {

	/// Create LockIdentifier via simple counter `locks_count`.
	/// Previously was by promise_id.
	fn next_free_lock_identifier(_promise_id: &T::Hash) -> LockIdentifier {
		use core::mem::size_of;
		let locks_count = Self::locks_count() + 1;
		let lid: [u8; size_of::<u64>()] = LockIdentifier::from(locks_count.to_le_bytes());
		lid
	}

	fn mint_c2fc(
		to: T::AccountId,
		c2fc_id: T::Hash,
		new_c2fc: Bucket<T::Hash, T::Balance, T::AccountId, T::BlockNumber>,
	) -> Result {
		ensure!(!<BucketOwner<T>>::exists(c2fc_id), "Bucket already exists");

		let owned_c2fc_count = Self::owned_c2fc_count(&to);

		let new_owned_c2fc_count = owned_c2fc_count
			.checked_add(1)
			.ok_or("Overflow adding a new c2fc to account balance")?;

		let all_c2fc_count = Self::all_c2fc_count();

		let new_all_c2fc_count = all_c2fc_count
			.checked_add(1)
			.ok_or("Overflow adding a new c2fc to total supply")?;

		<Buckets<T>>::insert(c2fc_id, new_c2fc);
		<BucketOwner<T>>::insert(c2fc_id, &to);

		<AllBucketsArray<T>>::insert(all_c2fc_count, c2fc_id);
		<AllBucketsCount<T>>::put(new_all_c2fc_count);
		<AllBucketsIndex<T>>::insert(c2fc_id, all_c2fc_count);

		<OwnedBucketsArray<T>>::insert((to.clone(), owned_c2fc_count), c2fc_id);
		<OwnedBucketsCount<T>>::insert(&to, new_owned_c2fc_count);
		<OwnedBucketsIndex<T>>::insert(c2fc_id, owned_c2fc_count);

		Self::deposit_event(RawEvent::C2fcCreated(to, c2fc_id));

		Ok(())
	}

	fn mint_promise(
		to: T::AccountId,
		promise_id: T::Hash,
		new_promise: FreePromise<T::Hash, T::Balance, T::BlockNumber>,
	) -> Result {
		ensure!(!<PromiseOwner<T>>::exists(promise_id), "Promise already exists");

		let owned_promise_count = Self::owned_promise_count(&to);

		let new_owned_promise_count = owned_promise_count
			.checked_add(1)
			.ok_or("Overflow adding a new promise to account balance")?;

		let free_promises_count = Self::free_promises_count();

		let new_free_promises_count = free_promises_count
			.checked_add(1)
			.ok_or("Overflow adding a new promise to total supply")?;

		<Promises<T>>::insert(promise_id, new_promise);
		<PromiseOwner<T>>::insert(promise_id, &to);

		<FreePromisesArray<T>>::insert(free_promises_count, promise_id);
		<FreePromisesCount<T>>::put(new_free_promises_count);
		<FreePromisesIndex<T>>::insert(promise_id, free_promises_count);

		<OwnedPromisesArray<T>>::insert((to.clone(), owned_promise_count), promise_id);
		<OwnedPromisesCount<T>>::insert(&to, new_owned_promise_count);
		<OwnedPromisesIndex<T>>::insert(promise_id, owned_promise_count);

		Self::deposit_event(RawEvent::PromiseCreated(to, promise_id));

		Ok(())
	}

	fn transfer_from(from: T::AccountId, to: T::AccountId, c2fc_id: T::Hash) -> Result {
		let owner = Self::owner_of_c2fc(c2fc_id).ok_or("No owner for this c2fc")?;

		ensure!(owner == from, "'from' account does not own this c2fc");

		let owned_c2fc_count_from = Self::owned_c2fc_count(&from);
		let owned_c2fc_count_to = Self::owned_c2fc_count(&to);

		let new_owned_c2fc_count_to = owned_c2fc_count_to
			.checked_add(1)
			.ok_or("Transfer causes overflow of 'to' c2fc balance")?;

		let new_owned_c2fc_count_from = owned_c2fc_count_from
			.checked_sub(1)
			.ok_or("Transfer causes underflow of 'from' c2fc balance")?;

		// "Swap and pop"
		let c2fc_index = <OwnedBucketsIndex<T>>::get(c2fc_id);
		if c2fc_index != new_owned_c2fc_count_from {
			let last_c2fc_id = <OwnedBucketsArray<T>>::get((from.clone(), new_owned_c2fc_count_from));
			<OwnedBucketsArray<T>>::insert((from.clone(), c2fc_index), last_c2fc_id);
			<OwnedBucketsIndex<T>>::insert(last_c2fc_id, c2fc_index);
		}

		<BucketOwner<T>>::insert(&c2fc_id, &to);
		<OwnedBucketsIndex<T>>::insert(c2fc_id, owned_c2fc_count_to);

		<OwnedBucketsArray<T>>::remove((from.clone(), new_owned_c2fc_count_from));
		<OwnedBucketsArray<T>>::insert((to.clone(), owned_c2fc_count_to), c2fc_id);

		<OwnedBucketsCount<T>>::insert(&from, new_owned_c2fc_count_from);
		<OwnedBucketsCount<T>>::insert(&to, new_owned_c2fc_count_to);

		Self::deposit_event(RawEvent::Transferred(from, to, c2fc_id));

		Ok(())
	}

	fn transfer_money(from: &T::AccountId, to: &T::AccountId, amount: T::Balance) -> Result {
		<balances::Module<T> as Currency<T::AccountId>>::transfer(&from, &to, amount)
	}


	// utilites //

	#[inline]
	pub fn is_promise_accepted(promise_id: T::Hash) -> result::Result<bool, &'static str> {
		ensure!(<Promises<T>>::exists(promise_id), "This promise does not exist");
		Ok(<AcceptedPromiseBucket<T>>::exists(promise_id))
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = ();
	}
	type CashflowModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default()
			.build_storage()
			.unwrap()
			.0
			.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(CashflowModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(CashflowModule::something(), Some(42));
		});
	}
}
