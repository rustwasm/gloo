<div align="center">

  <h1><code>gloo-storage</code></h1>

  <p>
    <a href="https://crates.io/crates/gloo-storage"><img src="https://img.shields.io/crates/v/gloo-storage.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/gloo-storage"><img src="https://img.shields.io/crates/d/gloo-storage.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/gloo-storage"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
  </p>

  <h3>
    <a href="https://docs.rs/gloo-storage">API Docs</a>
    <span> | </span>
    <a href="https://github.com/rustwasm/gloo/blob/master/CONTRIBUTING.md">Contributing</a>
    <span> | </span>
    <a href="https://discordapp.com/channels/442252698964721669/443151097398296587">Chat</a>
  </h3>

  <sub>Built with 🦀🕸 by <a href="https://rustwasm.github.io/">The Rust and WebAssembly Working Group</a></sub>
</div>

This crate provides wrappers for the [Web Storage API] and [IndexedDB API].

The data is stored in JSON form. We use [`serde`](https://serde.rs) for
serialization and deserialization.

# Indexed DB wrapper
This section explains how the IndexedDB wrapper works, and explains the design decisions taken, with
alternatives and rationale for the chosen option.

## Intro
`IndexedDB` (a.k.a `IDB`) is an object-store type database defined by the World-wide Web Consortium
([specification][idb spec]). It replaces earlier `WebSQL`, and is preferred over the [Web Storage API]
when working with larger amounts of data, as it will not block the JavaScript thread when fetching or storing
data. Features include

 - [Named object stores][IDBObjectStore]
 - [Indexes][IDBIndex]
 - [Cursors][IDBCursor] which allow iteration over objects without having to load them all at once, objects
   can be inspected and then modified/deleted without interrupting the cursor

## Guide

IndexedDB is a database that can be used in web browsers or other places JavaScript is used. When you change
or get data from the database, the operation doesn't finish instantly. Instead, you get an request back, and a
way to be notified when it has finished. The IDB wrapper in `gloo-storage` turns these requests into `Future`s
so you can use them the same way you'd use any other Rust future.

TODO more content. I don't think I need to recreate the docs, a worked example would be more useful and
shorter.

## Motivation

The reason for writing a wrapper for IDB is that it is really quite hard to use, even if we were writing
JavaScript. It requires the user to understand the purpose and operation of [IDBRequest], which looks
quite like a Rust future, but where concepts are named differently and callbacks are required to respond to
events. We can wrap the API to provide much more ideomatic Rust APIs with no or very little overhead over
what would be required anyway. We can also close off entire classes of errors using the type system, which
in JavaScript require exceptions (JS is loosely typed so cannot do what we can).

## Internal explanation
The internal workings of our wrapper.

### [IDBRequest]
The `[IDBRequest]` interface is the core of the IDB API. It provides the mechanism for operations to take place
asynchronously. It is created immediately by many IDB operations, and will fire events when the operation
completes, whether successfully or otherwise. Usually it completes at most once, but when using cursors it can
complete many times, moving back and forth between pending and done. The main methods on the interface are:

 - `readyState`: contains the state of the operation, and is a string matching either `"pending"` or `"done"`.
   Its value won't change within the same JS task, but can change between tasks.
 - `result`: contains the result of the request (or this result if it's a sequence) if it was successful.
 - `error`: contains the error if the request failed, or `NoError` if it succeeded.
 - `transaction`: the transaction object that created this request. Not all requests are associated
   with transactions.
 - The `success` event: indicates the request has succeeded and its `result` is available.
 - The `error` event: indicates the request has failed and the `error` is available.

We will ignore the `source` property, because the user will always already have access to the source (since
they needed it to create the request), and we can in theory prevent some errors by preventing it from being
accessed here.

The API above looks almost identical to a Rust `Future`. When we poll a request, we check its `readyState` to
see if we can complete straight away. If not, we make sure we are woken on completion by setting event handlers
for success and error. Putting this all together we get the implementation:

```rust
struct Request {
    // the raw request we wrap
    inner: IdbRequest,
    // by default, errors bubble up to cancel the transaction. We provide a flag to turn this off.
    bubble_errors: bool,
    // the event listeners
    success_listener: Option<EventListener>,
    error_listener: Option<EventListener>,
}

impl Request {
    fn new(inner: IdbRequest, bubble_errors: bool) -> Self {
        Self {
            inner,
            bubble_errors,
            success_listener: None,
            error_listener: None,
        }
    }
}

impl Future for Request {
    type Output = Result<JsValue, DomException>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.ready_state() {
            IdbRequestReadyState::Pending => {
                if self.success_listener.is_none() {
                    self.success_listener = Some(EventListener::once(&self.inner, "success", {
                        let waker = cx.waker().clone();
                        move |_| waker.wake()
                    }))
                }
                if self.error_listener.is_none() {
                    let opts = if self.bubble_errors {
                        EventListenerOptions::enable_prevent_default()
                    } else {
                        EventListenerOptions::default()
                    };
                    self.error_listener = Some(EventListener::once_with_options(
                        &self.inner,
                        "error",
                        opts,
                        {
                            let waker = cx.waker().clone();
                            let bubble_errors = self.bubble_errors;
                            move |event| {
                                waker.wake();
                                if !bubble_errors {
                                    event.prevent_default();
                                }
                            }
                        },
                    ))
                }
                Poll::Pending
            }
            IdbRequestReadyState::Done => {
                if let Some(error) = self.inner.error().unreachable_throw() {
                    Poll::Ready(Err(error))
                } else {
                    // no error = success
                    Poll::Ready(Ok(self.inner.result().unreachable_throw()))
                }
            }
            _ => unreachable_throw(),
        }
    }
}
```

I'm not going to post all of the implementation, but since this is the core of the whole wrapper, it's worth
reproducing verbatim. To put it simply, the operation of both the `Future` interface in Rust and the callback
interface of `IDBRequest` are the same: check if the operation is already done, then if not ask to be told
when it is. The only difference is the use of callbacks vs. task wakers, which this wrapper handles
transparently, so the user doesn't even need to be aware.

There are two other variants of the `Request` data. One is `OpenDbRequest`, which works the same except for
handling an extra event `blocked`. This event tells the user if the database is already in use. Since this
often indicates a programmer mistake, we provide an option to turn this event into an error.

> This wrapper doesn't provide any other way of handling the event currently. Open question: should it?

The other is a wrapper for `IDBRequest`s that can change back from `done` to `pending`. This only happens
when using cursors, so we create two different wrappers to make the other cases simpler. The streaming
version implements `Stream`, which is Rust's abstraction for futures that return multiple times.

### Transactions, Object stores, Indexes, and Cursors

A lot of the contents of the IDB wrapper just forward straight to their `web_sys` analogs, so there really
isn't that much to say about them. One exception to this rule is how the different types of transactions
are handled. These are `"readonly"`, `"readwrite"`, and `"versionchange"`. The three types of transaction
determine what you can do with the database: you need to be in a *versionchange* transaction to alter the
structure of the database (the other two types are self-explanatory). In the IDB spec this information is
stored internally, but we have the opportunity to use Rust's type system to make this explicit. The advantage
of this approach is that we can catch invalid use of the API (for example updating a record in a readonly
transaction) at compile-time! This is currently implemented using "uninhabited enums": enums that have no
variants so can never be constructed. They are simply used as generic markers so that our other structs
know what operations they are allowed to do.

> Alternatives are to not have different Rust types for different types of transaction. In this case we push
> the errors back to runtime, which seems like a regression to me. We could also have different types for each
> transaction, objecstore, ... (which would look like `ReadOnlyTransaction`, ReadWriteTransaction`, and so on).
> The downside to this is that we have more types and duplcated methods. The upside is that there are no
> generics, so it might be easier to understand for newcomers.

### Options

Where there is more than one argument to a function, this wrapper prefer using a special "Options" struct
(for example `ObjectStoreOptions`), with a `Default` impl so you can do e.g. `ObjectStoreOptions::default()`
if you don't want to change any of the defaults. They use the non-borrowing builder struct pattern.

### Key and KeyPath

The `Key` and `KeyPath` are new types introduced in this wrapper: the equivalent is untyped in JS. Strongly
typing this values enables us to catch more errors when they are created rather than used, which should aid
debugging. We can also provide more useful error messages.

> The alternative here is either to make a trait for valid values, or just accept untyped `JsValue`s. I think
> both are worse. The trait alternative means learning another new trait, rather than just using `From` and
> `TryFrom`, while the untyped option makes the API much more error-prone (methods will throw for invalid
> `Key`s and `KeyPath`s. The downsides to the used design is that mistakes in implementing the validity
> tests will result in opaque error messages. We could possibly make these messages better, or better in debug
> builds if there was a perf cost.

**TODO note**: currently the `KeyPath` uses the custom trait alternative design. I'm planning to change this
before marking the work ready for merge.

### Query

The concept of a "query" doesn't exist in IDB, but this wrapper introduces it to encapsulate filtering a
set of records. It can be 'all records', a single record, or a range of records. The JS equivalent involves
calling different methods depending on the type of query, and if its a range, the type of range as well.

> There are a number of choices we could make here. We could do away with the `Query` and just expose more
> methods for the different cases, but I think this makes the API significantly more cumbersome and
> error-prone. The `Query` name is somewhat arbitrary, and we could bikeshed alternatives (maybe `Filter`?).
> The `(&Key, bool)` tuple could be a named struct if people think this would aid readability. And
> the error semantics are currently designed to match the underlying JS - we could make them more like
> Rust `Range` semantics.

### Opening a database

When opening a database, we have to provide the user with some way of specifying how the database should
be updated. This wrapper uses a callback, with the new and old versions provided, along with an object for
modifying the database.

> There could conceivably be some declarative alternative, where you specify what the DB looks like in each
> version and library code works out exactly what to do to make it so. I think this is certainly not in
> gloo's remit as a 'middle-layer' and so should be discounted.

### Error handling

I think there is a potentially really good design for error handling, where most error types are the same,
but I haven't quite got it nailed yet. Will update once it's sorted.

[Web Storage API]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API
[IndexedDB API]: https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API
[idb spec]: https://w3c.github.io/IndexedDB/
[IDBObjectStore]: https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore
[IDBIndex]: https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex
[IDBCursor]: https://developer.mozilla.org/en-US/docs/Web/API/IDBCursor
