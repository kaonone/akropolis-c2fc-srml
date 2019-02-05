#![allow(non_upper_case_globals)]

use srml_support::StorageMap;
use srml_support::StorageValue;
use srml_support::dispatch::Result;
// ensure_signed, ensure_root, ensure_inherent
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash, Zero};
use parity_codec::Encode;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct Bucket<Hash, Balance, AccountId, BlockNumber> {
	// id: AccountId,
	id: Hash,

	promise: Option<Promise<Hash, Balance, AccountId, BlockNumber>>,

	/// price for selling the bucket
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
	// period: u64,

	/// filled value for current period
	filled: Balance,
	/// time (in blocks) when current period was started
	acception_dt: BlockNumber,
}

/// Describes not accepted "free promise"
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct FreePromise<Hash, Balance, BlockNumber> {
	id: Hash,
	/// promised value to fullfill
	value: Balance,
	/// time (number of blocks)
	period: BlockNumber,
	// period: u64,
}

// pub trait Trait: system::Trait {}
pub trait Trait: balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId,
		<T as system::Trait>::Hash,
		<T as balances::Trait>::Balance
	{
		BucketCreated(AccountId, Hash),
		/// OwnerSet: from, to, bucket
		PriceSet(AccountId, Hash, Balance),
		Transferred(AccountId, AccountId, Hash),
		Bought(AccountId, AccountId, Hash, Balance),


		/// FreePromise is created.
		PromiseCreated(AccountId, Hash),
		/// FreePromise is changed.
		PromiseChanged(Hash),
		/// FreePromise is accepted by owner of bucket.
		/// (PromiseID:Hash, BucketID:Hash)
		PromiseAccepted(Hash, Hash),
		/// (bucket_id:Hash, promise_id:Hash, value:Balance)
		PromiseFilled(Hash, Hash, Balance),
		/// (bucket_id:Hash, promise_id:Hash)
		PromiseFullilled(Hash, Hash),
		/// (bucket_id:Hash, promise_id:Hash, missed_deposit:Balance)
		PromiseBreached(Hash, Hash, Balance),
	}
);

decl_storage! {
	trait Store for Module<T: Trait> as C2FC {
		Buckets get(bucket): map T::Hash => Bucket<T::Hash, T::Balance, T::AccountId, T::BlockNumber>;
		BucketOwner get(owner_of): map T::Hash => Option<T::AccountId>;
		/// same as `AcceptedPromiseBucket` but by bucket_id
		BucketContributor get(contributor_of): map T::Hash => Option<T::AccountId>;

		AllBucketsArray get(bucket_by_index): map u64 => T::Hash;
		AllBucketsCount get(all_buckets_count): u64;
		AllBucketsIndex: map T::Hash => u64;

		OwnedBucketsArray get(bucket_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
		OwnedBucketsCount get(owned_bucket_count): map T::AccountId => u64;
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

		/// returns `bucket_id` for specified `promise_id`
		AcceptedPromiseBucket get(bucket_by_promise): map T::Hash => T::Hash;

		Nonce: u64;
	}
}


decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

		fn create_bucket(origin) -> Result {
			let sender = ensure_signed(origin)?;
			let nonce = <Nonce<T>>::get();
			let bucket_id = (<system::Module<T>>::random_seed(), &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);

			let new_bucket = Bucket {
					id: bucket_id,
					promise: None,
					price: <T::Balance as As<u64>>::sa(0),
			};

			Self::mint_bucket(sender, bucket_id, new_bucket)?;

			<Nonce<T>>::mutate(|n| *n += 1);

			Ok(())
		}

		fn create_promise(origin, value: T::Balance, period: T::BlockNumber) -> Result {
			let sender = ensure_signed(origin)?;
			let nonce = <Nonce<T>>::get();
			let promise_id = (<system::Module<T>>::random_seed(), &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);

			let new_promise = FreePromise {
				id: promise_id,
				value: value,
				period: period,
			};

			Self::mint_promise(sender, promise_id, new_promise)?;

			<Nonce<T>>::mutate(|n| *n += 1);

			Ok(())
		}

		fn edit_promise(origin, promise_id: T::Hash, value: T::Balance, period: T::BlockNumber) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Promises<T>>::exists(promise_id), "This promise does not exist");

			let owner = Self::owner_of(promise_id).ok_or("No owner for this promise")?;
			ensure!(owner == sender, "You do not own this promise");

			let mut promise = Self::promise(promise_id);
			promise.value = value;
			promise.period = period;

			<Promises<T>>::insert(promise_id, promise);
			Self::deposit_event(RawEvent::PromiseChanged(promise_id));

			Ok(())
		}


		/// Accept specified free promise and add it to specified bucket.
		/// Only owner of the bucket can do it.
		fn accept_promise(origin, promise_id: T::Hash, bucket_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Buckets<T>>::exists(bucket_id), "This bucket does not exist");
			ensure!(<Promises<T>>::exists(promise_id), "This promise does not exist");
			ensure!(<AcceptedPromiseBucket<T>>::exists(promise_id), "This promise is already accepted");


			let bucket_owner = Self::owner_of(bucket_id).ok_or("No owner for this bucket")?;
			ensure!(bucket_owner == sender, "You do not own this promise");

			let promise_owner = Self::owner_of_promise(promise_id).ok_or("No owner for this promise")?;
			ensure!(promise_owner != sender, "You can not accept your own promise");

			let mut bucket = Self::bucket(bucket_id);
			ensure!(bucket.promise.is_none(), "Bucket already contains another promise");

			// get current (latest) block:
			let current_block = <system::Module<T>>::block_number();

			let free_promise = Self::promise(promise_id);
			let promise = Promise {
				id: free_promise.id,
				// in the near future `owner` can be removed
				owner: promise_owner.clone(),
				value: free_promise.value,
				period: free_promise.period,
				acception_dt: current_block,
				filled: <T::Balance as As<u64>>::sa(0),
			};

			bucket.promise = Some(promise);
			<Buckets<T>>::insert(bucket_id, bucket);
			<AcceptedPromiseBucket<T>>::insert(promise_id, bucket_id);

			// incrmnt the counter & push to maps:
			{
				let accepted_promises_count = Self::accepted_promises_count();
				let new_accepted_promises_count = accepted_promises_count
					.checked_add(1)
					.ok_or("Overflow adding a new promise to total supply")?;

				<BucketContributor<T>>::insert(bucket_id, promise_owner);

				<AcceptedPromisesArray<T>>::insert(accepted_promises_count, promise_id);
				<AcceptedPromisesCount<T>>::put(new_accepted_promises_count);
				<AcceptedPromisesIndex<T>>::insert(promise_id, accepted_promises_count);
			}

			<Nonce<T>>::mutate(|n| *n += 1);

			Self::deposit_event(RawEvent::PromiseAccepted(promise_id, bucket_id));

			Ok(())
		}


		// selling & trasfering a bucket //

		fn set_price(origin, bucket_id: T::Hash, new_price: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Buckets<T>>::exists(bucket_id), "This bucket does not exist");

			let owner = Self::owner_of(bucket_id).ok_or("No owner for this bucket")?;
			ensure!(owner == sender, "You do not own this bucket");

			let mut bucket = Self::bucket(bucket_id);
			bucket.price = new_price;

			<Buckets<T>>::insert(bucket_id, bucket);

			Self::deposit_event(RawEvent::PriceSet(sender, bucket_id, new_price));

			Ok(())
		}

		fn transfer(origin, to: T::AccountId, bucket_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(bucket_id).ok_or("No owner for this bucket")?;
			ensure!(owner == sender, "You do not own this bucket");

			Self::transfer_from(sender, to, bucket_id)?;

			Ok(())
		}

		fn buy_bucket(origin, bucket_id: T::Hash, max_price: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Buckets<T>>::exists(bucket_id), "This bucket does not exist");

			let owner = Self::owner_of(bucket_id).ok_or("No owner for this bucket")?;
			ensure!(owner != sender, "You can't buy your own bucket");

			let mut bucket = Self::bucket(bucket_id);

			let bucket_price = bucket.price;
			ensure!(!bucket_price.is_zero(), "The bucket you want to buy is not for sale");
			ensure!(bucket_price <= max_price, "The bucket you want to buy costs more than your max price");

			<balances::Module<T>>::make_transfer(&sender, &owner, bucket_price)?;

			Self::transfer_from(owner.clone(), sender.clone(), bucket_id)?;

			bucket.price = <T::Balance as As<u64>>::sa(0);
			<Buckets<T>>::insert(bucket_id, bucket);

			Self::deposit_event(RawEvent::Bought(sender, owner, bucket_id, bucket_price));

			Ok(())
		}


		// do/fill the promises //

		fn fill_bucket(origin, bucket_id: T::Hash, deposit: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Buckets<T>>::exists(bucket_id), "This bucket does not exist");

			let owner = Self::owner_of(bucket_id).ok_or("No owner for this bucket")?;
			ensure!(owner != sender, "You can't fill your own bucket");

			let mut bucket = Self::bucket(bucket_id);
			ensure!(bucket.promise.is_some(), "This bucket does not contains promise");


			if let Some(ref mut promise) = bucket.promise {
				let promise_id = promise.id;

				ensure!(!promise.value.is_zero(), "The promise in the bucket you want to fill is invalid");
				ensure!(promise.filled <= promise.value, "The bucket you want to fill is already fullfilled");

				<balances::Module<T>>::make_transfer(&sender, &owner, deposit)?;

				promise.filled = deposit + promise.filled;

				Self::deposit_event(RawEvent::PromiseFilled(bucket_id, promise_id, deposit));

				if promise.filled >= promise.value {
					Self::deposit_event(RawEvent::PromiseFullilled(bucket_id, promise_id));
				}
			}

			// re-store the bucket
			<Buckets<T>>::insert(bucket_id, bucket);

			Ok(())
		}

		fn fullfill_bucket(origin, bucket_id: T::Hash) -> Result {
			let deposit = {
				ensure!(<Buckets<T>>::exists(bucket_id), "This bucket does not exist");
				let bucket = Self::bucket(bucket_id);
				let promise = &bucket.promise.ok_or("This bucket doesnt contains an accepted promise")?;
				let deposit = promise.filled - promise.value;
				deposit
			};

			Self::fill_bucket(origin, bucket_id, deposit)
		}


		// combine the buckets //

		// fn mix_buckets(origin, bucket_id_1: T::Hash, bucket_id_2: T::Hash) -> Result{
		// 	use rstd::cmp;
		// 	let sender = ensure_signed(origin)?;

		// 	ensure!(<Buckets<T>>::exists(bucket_id_1), "This bucket 1 does not exist");
		// 	ensure!(<Buckets<T>>::exists(bucket_id_2), "This bucket 2 does not exist");

		// 	{
		// 		let owner_1 = Self::owner_of(bucket_id_1).ok_or("No owner for this bucket")?;
		// 		let owner_2 = Self::owner_of(bucket_id_2).ok_or("No owner for this bucket")?;
		// 		ensure!(owner_1 == sender, "You can mix your own bucket only");
		// 		ensure!(owner_2 == sender, "You can mix your own bucket only");
		// 	}

		// 	let nonce = <Nonce<T>>::get();
		// 	let new_bucket_id = (<system::Module<T>>::random_seed(), &sender, nonce)
		// 			.using_encoded(<T as system::Trait>::Hashing::hash);

		// 	let bucket_1 = Self::bucket(bucket_id_1);
		// 	let bucket_2 = Self::bucket(bucket_id_2);

		// 	// TODO: impl mix for Promise

		// 	let new_bucket = Bucket {
		// 			id: new_bucket_id,
		// 			...
		// 			price: <T::Balance as As<u64>>::sa(0),
		// 	};

		// 	Self::mint_bucket(sender, new_bucket_id, new_bucket)?;

		// 	<Nonce<T>>::mutate(|n| *n += 1);

		// 	// TODO: stale/kill both buckets: bucket_id_1 & bucket_id_1

		// 	Ok(())
		// }


		/// Check the breach of promise at end of the each block.
		/// Simple timer here.
		fn on_finalise(n: T::BlockNumber) {
			let accepted_promises_count = Self::accepted_promises_count();

			for i in 0..accepted_promises_count {
				let promise_id = Self::accepted_promise_by_index(i);
				let bucket_id = Self::bucket_by_promise(promise_id);

				if <Buckets<T>>::exists(bucket_id) {
					let bucket = Self::bucket(bucket_id);
					// skip if bucket doesn't contains a promise
					if let Some(promise) = &bucket.promise {
						let lifetime = n - promise.acception_dt;
						let wanted_deposit = promise.filled - promise.value;
						// if (lifetime % promise.period).is_zero() && !wanted_deposit.is_zero() {
						if (lifetime % promise.period).is_zero() {
							// TODO: reset `promise.filled` to zero because new period starts.

							if wanted_deposit > <T::Balance>::zero() {
								// here we should to emit Event about *failed promise*.
								Self::deposit_event(RawEvent::PromiseBreached(bucket_id, promise_id, wanted_deposit));
								// <BucketContributor<T>>::...(bucket_id,);
							}
						}
					}
				}
			}
		}
	}
}


// private & utils //

use rstd::result;

impl<T: Trait> Module<T> {
	fn mint_bucket(to: T::AccountId, bucket_id: T::Hash, new_bucket: Bucket<T::Hash, T::Balance, T::AccountId, T::BlockNumber>) -> Result {
		ensure!(!<BucketOwner<T>>::exists(bucket_id), "Bucket already exists");

		let owned_bucket_count = Self::owned_bucket_count(&to);

		let new_owned_bucket_count = owned_bucket_count.checked_add(1)
		                                               .ok_or("Overflow adding a new bucket to account balance")?;

		let all_buckets_count = Self::all_buckets_count();

		let new_all_buckets_count = all_buckets_count.checked_add(1)
		                                             .ok_or("Overflow adding a new bucket to total supply")?;

		<Buckets<T>>::insert(bucket_id, new_bucket);
		<BucketOwner<T>>::insert(bucket_id, &to);

		<AllBucketsArray<T>>::insert(all_buckets_count, bucket_id);
		<AllBucketsCount<T>>::put(new_all_buckets_count);
		<AllBucketsIndex<T>>::insert(bucket_id, all_buckets_count);

		<OwnedBucketsArray<T>>::insert((to.clone(), owned_bucket_count), bucket_id);
		<OwnedBucketsCount<T>>::insert(&to, new_owned_bucket_count);
		<OwnedBucketsIndex<T>>::insert(bucket_id, owned_bucket_count);

		Self::deposit_event(RawEvent::BucketCreated(to, bucket_id));

		Ok(())
	}

	fn mint_promise(to: T::AccountId, promise_id: T::Hash, new_promise: FreePromise<T::Hash, T::Balance, T::BlockNumber>) -> Result {
		ensure!(!<PromiseOwner<T>>::exists(promise_id), "Promise already exists");

		let owned_promise_count = Self::owned_promise_count(&to);

		let new_owned_promise_count = owned_promise_count.checked_add(1)
		                                                 .ok_or("Overflow adding a new promise to account balance")?;

		let free_promises_count = Self::free_promises_count();

		let new_free_promises_count = free_promises_count.checked_add(1)
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

	fn transfer_from(from: T::AccountId, to: T::AccountId, bucket_id: T::Hash) -> Result {
		let owner = Self::owner_of(bucket_id).ok_or("No owner for this bucket")?;

		ensure!(owner == from, "'from' account does not own this bucket");

		let owned_bucket_count_from = Self::owned_bucket_count(&from);
		let owned_bucket_count_to = Self::owned_bucket_count(&to);

		let new_owned_bucket_count_to = owned_bucket_count_to.checked_add(1)
		                                                     .ok_or("Transfer causes overflow of 'to' bucket balance")?;

		let new_owned_bucket_count_from = owned_bucket_count_from.checked_sub(1)
		                                                         .ok_or("Transfer causes underflow of 'from' bucket balance")?;

		// "Swap and pop"
		let bucket_index = <OwnedBucketsIndex<T>>::get(bucket_id);
		if bucket_index != new_owned_bucket_count_from {
			let last_bucket_id = <OwnedBucketsArray<T>>::get((from.clone(), new_owned_bucket_count_from));
			<OwnedBucketsArray<T>>::insert((from.clone(), bucket_index), last_bucket_id);
			<OwnedBucketsIndex<T>>::insert(last_bucket_id, bucket_index);
		}

		<BucketOwner<T>>::insert(&bucket_id, &to);
		<OwnedBucketsIndex<T>>::insert(bucket_id, owned_bucket_count_to);

		<OwnedBucketsArray<T>>::remove((from.clone(), new_owned_bucket_count_from));
		<OwnedBucketsArray<T>>::insert((to.clone(), owned_bucket_count_to), bucket_id);

		<OwnedBucketsCount<T>>::insert(&from, new_owned_bucket_count_from);
		<OwnedBucketsCount<T>>::insert(&to, new_owned_bucket_count_to);

		Self::deposit_event(RawEvent::Transferred(from, to, bucket_id));

		Ok(())
	}

	// utilites //

	#[inline]
	pub fn is_promise_accepted(promise_id: T::Hash) -> result::Result<bool, &'static str> {
		ensure!(<Promises<T>>::exists(promise_id), "This promise does not exist");
		Ok(<AcceptedPromiseBucket<T>>::exists(promise_id))
	}
}
