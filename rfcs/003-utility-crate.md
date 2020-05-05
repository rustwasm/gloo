# Utility crate for gloo

## Summary

Create a utility crate for various small miscellaneous APIs that don't fit into other crates.

## Motivation

Having things cleanly separated into different crates is good. But not all APIs fit into that model.

For example, having convenience accessors for `window` and `document` don't really belong in their own crate.

Similarly, one-off functions like `document_ready` or `is_window_loaded` don't belong in their own crate.

## Proposal

Large features will continue to go into separate crates (like `gloo-file`, `gloo-events`, etc.)

Small features will go into the `gloo` crate.

The `gloo` crate will no longer re-export the other crates (so `gloo::events`, `gloo::file` no longer work).

The `gloo` crate's version will now only be dependent upon its own code, it won't be tied to the other `gloo-` crates.

The reason for this breaking change is that I don't think the "umbrella crate" concept offers us much advantage.
From the user's perspective, the only benefit is that they only need to manage 1 version rather than multiple.

But that's not really a big deal: pretty much any crate is going to have multiple dependencies.

## Alternatives

We could instead create a new `gloo-utils` crate. However, this doesn't seem to offer much advantage compared to putting things into `gloo`.
