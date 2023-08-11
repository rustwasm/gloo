## `console`

### Version 0.3.0 

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)
- Introduces the `FromQuery` and `ToQuery` traits to allow for customizing
  how query strings are encoded and decoded in `gloo_history`. (#364)

### Version "0.2.3"

- Release new gloo versions

### Version "0.2.2"

- feat(gloo-utils): Lift serde-serialization from wasm-bindgen (#242)
- fix: Break dependency cycle by not using serde-serialize (#239)

### Version "0.2.1"

- Fix `utils` crate and `history` docs. (#189)
- Hash-based History type & Unified Location. (#177)
- Fixes `console_dbg!` and `console!` expression output. Bold src info. (#174)

### Version "0.2.0"

- Add console_dbg macro (#170)

### Version "0.1.0"

- Add an `dbg!` equivalent to `gloo-console` (#158)
- Fix dir, dirxml macros in gloo-console (#154)
- Finishing touches

## `dialogs`

### Version "0.2.0"

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)

### Version "0.1.0"

## `events`

### Version "0.2.0"

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)
- Add traits `PartialEq` and `Eq` to `EventListenerOptions`, and `EventListenerPhase` (#363)

### Version "0.1.1"

- Add implementation for rfc-1 (files)
- Update readmes
- Address some mistakes and nits.
- Address code review.
- Update readmes
- Migrate to futures 0.3
- Bump events version

### Version "0.1.0"

- Updating the links in the events crate
- Adding in crate metadata for events
- Annotate trait objects ready for when they become mandatory.
- Fix link to CI build in README template
- Generate `README.md`s for each crate from its top-level docs
- gloo_event: Take events by reference
- Derive Debug impls
- Set web-sys version to avoid warning
- Changing the with_options methods to no longer take EventListenerOptions by reference (since it's Copy anyways)
- Fixing minor nit
- Fixing whitespace
- Adding in unit test for dynamic registration
- Removing once option, and adding in once_with_options method
- Apply suggestions from code review
- Adding some crate-level docs
- Fixing bug with Drop
- Adding in docs, and also making some changes to EventListenerOptions
- Adding in unit tests for gloo-events
- Renaming A parameter to S
- Fixing all the issues with gloo-events

## `file`

### Version "0.3.0"

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)

### Version "0.2.2"

- [rfc] Implement an ObjectUrl wrapper (#231)

### Version "0.2.0"

- impl Clone and PartialEq (#184)
- Hash-based History type & Unified Location. (#177)
- Prepare for 0.4 release (#156)

### Version "0.1.0"

- Remove the unnecessary copy in `Blob::new` (#152)
- Prepare v0.3.0 release (#148)
- Make docs.rs include futures functionality (#116)
- Add wrappers for web storage API (#125)
- Add wrappers for `alert`, `confirm`, and `prompt` functions (#123)
- Adding README for gloo-file
- Fixing Cargo.toml for gloo-file

## `history`

### Version 0.2.0

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)

### Version 0.1.4

- Use `thread_local!`'s lazy initialization instead of `RefCell<Option<_>>` (#358)
- Fix required feature set of serde dependency (#357)

### Version "0.1.3"

- Fix some typos (#313)
- Update serde-wasm-bindgen requirement from 0.4.5 to 0.5.0 (#320)
- Fix clippy for Rust 1.67 (#302)

### Version "0.1.2"

- (history): Drop states borrow before callback invocation (#285)
- Update serde-wasm-bindgen requirement from 0.3.1 to 0.4.5 (#297)
- Fix clippy. (#287)

### Version "0.1.1"

- Fix history tests (#252)
- Add query() method (#215)
- Fix failing history tests (#219)
- Fix links in gloo-history README (#210)

### Version "0.1.0"

- Fix `utils` crate and `history` docs. (#189)
- 0.5.0
- Memory-based History (#178)
- Hash-based History type & Unified Location. (#177)

## `net`

### Version "0.4.0"

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)

### Version "0.3.1"

- export RequestBuilder and ResponseBuilder as public

### Version "0.3.0"

- Seanaye/feat/serverside http (#312)

### Version "0.2.6"

- Add `PartialEq, Eq, PartialOrd, Ord, Hash` for eventsource `State` (#336)
- Seanaye/feat/serverside http (#312)
- Fix clippy for Rust 1.67 (#302)

### Version "0.2.5"

- Fix clippy. (#287)
- Prevent send from hanging if connection fails. (#280)

### Version "0.2.4"

- fix(ws): calling close event with destroyed close callback (#264)
- fix: cyclic dependency for gloo-net websocket feature (#260)
- Gloo net fetch in worker (#253)
- fix: remove unused import in gloo-net::http (#257)
- Fix Request.json(): Use Rust Serde Serialization instead of Javascript Evaluator. Avoids Big Integer serialization issues. (#256)
- Add std::error::Error impl for WebSocketError (#250)
- Provides an EventSource implementation (#246)
- Release new gloo versions

### Version "0.2.3"

- feat(gloo-utils): Lift serde-serialization from wasm-bindgen (#242)
- Fix feature soundness issues with gloo-net (#243)
- fix: Break dependency cycle by not using serde-serialize (#239)
- gloo-net v0.2.3

### Version "0.2.2"

- Add missing feature flags to gloo-net (#230)
- gloo-net v0.2.2

### Version "0.2.1"

- Feature soundness of gloo-http (#228)
- Release v0.8.0

### Version "0.2.0"

- Added support for specifying Websocket Protocols (#222)
- Add query() method (#215)
- Move UncheckedIter to gloo-utils (#217)
- docs: revise docs for gloo_net::http::Request.method (#212)

### Version "0.1.0"

- add `json()` impl to `Request` (#204)
- Improve the Fetch API (#188)

## `render`

### Version "0.2.0"

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)

### Version "0.1.0"

## `storage`

### Version "0.3.0"

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)

### Version "0.2.1"

- fix: Break dependency cycle by not using serde-serialize (#239)

### Version "0.2.0"

- Fix up gloo-storage for release
- Prepare for 0.4 release (#156)

### Version "0.1.0"

- Utility crate for common `web_sys`/`js_sys` features (#155)

## `timers`

### Version "0.3.0"

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)

### Version "0.2.5"

- fix: `clearTimeout` illegal invocation with bundler (#187) (#283)

### Version "0.2.4"

- gloo_timers, ambiguous verbage (#255)

### Version "0.2.3"

- New patch versions

### Version "0.2.2"

- Remove `web-sys` dependency (#186)
- Add node.js support for timers (#185)
- 0.5.0

### Version "0.2.1"

- Hash-based History type & Unified Location. (#177)
- Add BrowserHistory and BrowserLocation (#171)
- Add sleep. (#163)
- Prepare v0.3.0 release (#148)
- Make docs.rs include futures functionality (#116)
- gloo-timers 0.2.1

### Version "0.2.0"

- Add implementation for rfc-1 (files)
- Change implementation of getting new global.
- Change macro implementation.
- Replace macro by enum `WindowOrWorker`.
- Fix timers to work in workers too.
- Preparing for release

### Version "0.1.0"

- Changing to use web-sys; this fixes a Webpack error
- Fix missing wasm_bindgen import.
- Address some mistakes and nits.
- Use futures_channel for faster and simpler code
- Remove unused dependency
- Address code review.
- Migrate to futures 0.3
- Adding in crate metadata for timers
- Annotate trait objects ready for when they become mandatory.
- Fix link to CI build in README template
- Generate `README.md`s for each crate from its top-level docs
- Derive Debug impls
- Merge pull request #57 from samcday/fix-timers-interval
- gloo-timers: move the Javascript API bindings into a sys module
- gloo-timers: rework interval tests a little bit and ensure that intervals fire more than once
- gloo-timers: Use raw bindings to (set|clear)(Timeout|Interval) instead of the Window API, since window isn't always present (e.g Web Workers)
- gloo-timers: don't consume callback in Interval closure - otherwise interval only works on first callback and fails after that
- timers: Split the submodules out into their own files
- Fix the gloo_timers browser tests
- timers: use Closure::once for timeouts
- Add note about feature requirement to docs.
- Change feature name to "futures".
- Fix tests to accommodate new submodules.
- Split up callback and future/stream APIs.
- timers: Fix author line in Cargo.toml

## `utils`

### Version "0.2.0"

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)

### Version "0.1.6"

- Update json.rs fix typo (#338)
- docs: correct format examples Fixes #276 (#278)

### Version "0.1.5"

- refactor: typo fix (#262)
- Release new gloo versions

### Version "0.1.4"

- feat(gloo-utils): Lift serde-serialization from wasm-bindgen (#242)
- Release v0.8.0

### Version "0.1.3"

- Implement std Error trait for JsError (#225)
- Move UncheckedIter to gloo-utils (#217)

### Version "0.1.2"

- Fix `utils` crate and `history` docs. (#189)
- Gloo v0.6.0

### Version "0.1.1"

- Html head access (#179)
- 0.4.2

### Version "0.1.0"

- utils: Add body() and document_element() getters (#161)

## `worker`

### Version 0.4.0

- Migrate to Edition 2021 and Apply MSRV in Cargo.toml (#360)
- Add Worker Loader (#349)

### Version 0.3.0

- Function Worker (#267)

### Version "0.2.0"

- Release v0.8.0

### Version "0.1.1"

- Worker v2 (#200)
- Remove the private worker refcell (#223)
