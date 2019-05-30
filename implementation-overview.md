

### What is Commitments to Future Cashflows:

More about it you can read [here](https://wiki.akropolis.io/c2fc/overview/)

### Design

Core elements of C2FC on Substrate implementation:

- User accounts
- FreePromise
- Promise
- Bucket
- Storage-module C2FC

Creation of Commitment to Future Cashflow consists of two steps: at first, you create *Free Promise* and then assign a payee to it (*Promise* stage). While C2FC is in the *Free Promise stage*, it hasn’t recipient of payments (issuer is a recipient).
Any user can create an unlimited number of *FreePromises*. *FreePromise* has parameteres:

- Id
- Due amount (value)
- Term (period)
- Issuer’s Account id.

*Promise* has parameters:

- Id (the same for both *FreePromise* and *Promise*)
- Due amount (value)
- Term (period)
- Amount already deposited to *Promise* (filled amount)
- *Promise* Issuance date
- Bucket id.

The only option to create *Promise* is to assign a payee to *FreePromise*. When the recipient includes Promise in his *Bucket*, *FreePromise* becomes *Promise*.

*Bucket* has its price. Price is set by *Bucket's* owner. If price is not 0, anyone could buy this bucket for this price.

*Bucket* has parameters:

- Bucket Id
- Price
- *Promise* (could be null value).

*Storage C2FC* implements business logic of *Bucket* creation/adding *Promise* to *Bucket*.

### Process Flow

**Transfer FreePromise to Bucket**

Initial conditions:

- Bucket owner isn't FreePromise issuer
- Bucket's *Promise* field has null value


Bucket's owner calls method ```accept_promise(..., promise_id, bucket_id)``` to transfer FreePromise to Bucket, where promise_id - id of FreePromise, Bucket_id - id of Bucket, that will own Promise.

When the function ```accept_promise(..., promise_id, bucket_id)``` is called the following happens:

- Verify the ability to perform an operation
```rust
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
```

- *Promise* is created and its has copy of all data from *FreePromise* (!they have the same id).

- *Promise* keeps number of current last block.

- *Promise* is assigned to *Bucket*.

- Bucket is registered in navigational hashmaps.
```rust
<Buckets<T>>::insert(bucket_id, bucket);
// associate promise with bucket:
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
```

- Write event to chain.
```rust

Self::deposit_event(RawEvent::PromiseAccepted(promise_id, bucket_id));

```
