# Minimal Example

This repo is a minimal example of how I'm trying to start a Hyper
server on a hand assembled Tokio runtime.

The tokio runtime setup comes from [Building a
Runtime](https://tokio.rs/docs/going-deeper/building-runtime/) in the
tokio docs, so it should be correct.

The Hyper server setup uses the `conn:Http` type and is based on what
I did to set this same thing up in 0.11, but adapted for the changed
API in Hyper 0.12.


## Expected

I expected every HTTP request to this server to get the response
"HELLO WORLD!".

## Actual

Instead, the connection is reset/closed immediately after it is
opened.
