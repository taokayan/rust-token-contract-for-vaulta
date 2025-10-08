# notes:

# Because wasm bulk memory ops is not supported in Vaulta
# We must use Rust version earlier than 1.82
# e.g. rustc 1.79.0 (129f3b996 2024-06-10)

export WASM_INTERFACE_TYPES=1

rm -rf ./target/*
RUSTFLAGS="-C link-arg=-zstack-size=32768 -C target-feature=-mutable-globals,-sign-ext,-multivalue,-simd128" wasm-pack build --target web $@

