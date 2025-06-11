<h1 align="center">Lattice.rust</h1>

<p align="center">
    <img alt="Static Badge" src="https://img.shields.io/badge/rust-v1.80.1-blue?logo=rust">
    <img alt="Static Badge" src="https://img.shields.io/badge/build-passing-green?logo=github">
    <img alt="Static Badge" src="https://img.shields.io/badge/release-v1.0.0-blue?logo=adguard">
    <img alt="Static Badge" src="https://img.shields.io/badge/Evm-support-orange?logo=ethereum">
</p>

## 📖 Intro

Rust language implementation of blockchain SDK.

## 🧰 WASM

[WASM](https://rustwasm.github.io/wasm-pack/book/)

## 🕸️ Website

local docs:

```shell
cargo doc --open
```

## ⏰ Plan in 2025 Year

- [ ] Optimize error handling, avoid abuse unwrap and expect.
- [ ] Website
- [ ] Examples
- [ ] Complete the [retry-rust](https://github.com/wylu1037/retry-rust)
- [ ] WASM

## Misc

How to add a submodule?

```shell
cargo new --lib your_module
```

## TODO

1. 不要使用`unwrap`或`expect`，使用`Result<T, E>`或`Option<T>`；
2. 定义自己的错误类型