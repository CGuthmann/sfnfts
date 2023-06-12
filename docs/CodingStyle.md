# Coding Style Guide

## General Coding Guidelines
- Avoid using the global Namespace
- Avoid unnecessary complex constructs. Everybody should be able to understand the code.
- Be mindful of optimizations

## Documentation Guidelines
- Every code file should start with a short introduction to its purpose.
- Every function should be proceeded by a short description.
- Complex code constructs or algorithms should be proceeded by a short explanation or reference to a source.
- Use descriptive variable names.
- Write whole sentences and use punctuation.
- Don‘t pre-announce features.

## Limitations of this Guide
It is important to note that we only discuss guidelines here. If your code is significantly improved by breaking a rule, this is recommended.

## Rust coding Guide
- Use BigInt for generic Integer values.
- Always return Results.
- Handle Results and avoid using unwrap.
- Avoid using unsafe code wherever possible.
- Specify crate versions (or git commits) in the dependencies. This helps to reduce the risk of dependency mismatches
- Frequently update dependencies to take advantage of the newest features and bug fixes.
  

## Circom coding Guide
- Comment on every section of the code for its purpose.
- Number public signals in the ordern in which they are provided by the witness
- Comment all non-trivial assumptions over constraints.
