---
name: Propose Design
about: Propose an API or crate design for Gloo
title: ''
labels: ''
assignees: ''
---

## Summary

Short overview of the proposal.

## Motivation

Why are we doing this? What problems does it solve?

## Detailed Explanation

Introduce and explain the new APIs and concepts. How will this proposal be
implemented? Provide representative and edge-case examples.

Provide a skeleton of the proposed API by writing out types (don't need their
members or full implementation) as well as function and method signatures
(again, just the signature, don't need the function body):

```rust
pub struct Whatever { ... }

impl Whatever {
    pub fn new(raw: &web_sys::RawWhatever) -> Self { ... }
    pub fn another(&self) -> Another { ... }
}

pub struct Another { ... }

// Does X, Y, and Z when dropped.
impl Drop for Another {}
```

## Drawbacks, Rationale, and Alternatives

Does this design have drawbacks? Are there alternative approaches? Why is this
design the best of all designs available?

What prior art exists? There are many good sources of inspiration: Ember, React,
Angular, Vue, Knockout, jQuery, Closure, Elm, Emscripten, ClojureScript,
Polymer, etc..

## Unresolved Questions

What is not clear yet? What do we expect to clarify through implementation
and/or usage experience?
