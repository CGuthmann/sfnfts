#! /bin/bash

cargo test nft_throughput --release -- --nocapture 20
cargo test nft_throughput --release -- --nocapture 50
cargo test nft_throughput --release -- --nocapture 100
cargo test nft_throughput --release -- --nocapture 500
cargo test nft_throughput --release -- --nocapture 1000