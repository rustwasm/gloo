# TodoMVC example

This is an example to show the use of the IndexedDB wrapper in `gloo-storage`.

First, [install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) if needed.

Then build the example by running `wasm-pack build --target web` and open your browser to load `index.html`.

This example is uses [`dominator`](https://crates.io/crates/dominator) to update the DOM. It's a fairly low-level library to help with keeping the DOM and our app state in sync. You don't need to understand it to see how IndexedDB is used in this library (just search for `idb` to see the library in action).
