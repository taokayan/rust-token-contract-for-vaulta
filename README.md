# Simple token contract using Rust for Vaulta (prev. EOS) blockchain

## Build Step:

What you need:
- *Rust version 1.79.0* (or any version less than 1.82, because Vaulta does not support wasm bulk memory ops)
- *wasm-bindgen 0.2* (However, you need `#[no_mangle]` in your source code for exporting the `apply` function which is the entry point for Vaulta smart contract)
- *wasm-pack*

See https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm for installation

Steps to build:
```
clean_build.sh
```

You will get the wasm file in
```
./target/wasm32-unknown-unknown/release/hello_wasm.wasm
```

## Deploy to Vaulta Blockchain
In this example, the test account is `a123`
```
cleos set code a123 ./target/wasm32-unknown-unknown/release/hello_wasm.wasm
cleos set abi a123 ./token.abi
```

## Create your own token
```
cleos push action a123 create '{"issuer":"a123","maximum_supply":"1000000.0000 RUST"}' -p a123
```

## issue token
```
cleos push action a123 issue '{"to":"a124","quantity":"100.0000 RUST","memo":"hihi"}' -p a123
executed transaction: 5586add21a12371660df3d24ea1542d94ec79db8c086cd01e7d2280788567470  128 bytes  954 us
#          a123 <= a123::issue                  {"to":"a124","quantity":"100.0000 RUST","memo":"hihi"}
warning: transaction executed locally, but may not be confirmed by the network yetult         ]
```

## transfer token
```
cleos transfer a124 a125 "1.0 RUST" -c a123
executed transaction: 924977560c3aca6c01ba9fe4636a948978765076df2014caba9ca1ec574f35a9  128 bytes  968 us
#          a123 <= a123::transfer               {"from":"a124","to":"a125","quantity":"1.0000 RUST","memo":""}
warning: transaction executed locally, but may not be confirmed by the network yetult         ]
```

## get token balance of account a124
```
cleos get currency balance a123 a124
99.7000 RUST
```

## get currenty stats
```
cleos get currency stats a123 RUST
{
  "RUST": {
    "supply": "301.0000 RUST",
    "max_supply": "1000000.0000 RUST",
    "issuer": "a123"
  }
}
```


