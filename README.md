# `cozy-chess`

## Rust Chess and Chess960 move generation library
`cozy-chess` is a Chess and Chess960 move generation library written in Rust. It is largely inspired by Jordan Bray's neat [`chess`](https://github.com/jordanbray/chess) move generation library. Compared to `chess`, it provides a more ergonomic but less featureful interface. It also uses a significantly lower (currently zero) amount of `unsafe`, which has contributed to numerous unsoundness bugs in `chess`. This was the primary reason for its creation. Basic perft testing seems to indicate similar performance to `chess`.
