# CLASP.Rust <!-- omit in toc -->

Command-Line Argument Sorting and Parsing (for Rust)

![Language](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
[![Crates.io](https://img.shields.io/crates/v/clasp-rs.svg)](https://crates.io/crates/clasp-rs)
[![GitHub release](https://img.shields.io/github/v/release/synesissoftware/CLASP.Rust.svg)](https://github.com/synesissoftware/CLASP.Rust/releases/latest)
![MSRV](https://img.shields.io/badge/MSRV-1.74-lightgrey)
[![CI](https://github.com/synesissoftware/CLASP.Rust/actions/workflows/ci.yml/badge.svg)](https://github.com/synesissoftware/CLASP.Rust/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/clasp-rs/badge.svg)](https://docs.rs/clasp-rs)


## Table of Contents <!-- omit in toc -->

- [Introduction](#introduction)
- [Developer Experience](#developer-experience)
- [Installation](#installation)
- [Components](#components)
  - [Constants](#constants)
  - [Enumerations](#enumerations)
  - [Features](#features)
  - [Functions](#functions)
  - [Macros](#macros)
  - [Structures](#structures)
  - [Traits](#traits)
- [Examples](#examples)
- [Project Information](#project-information)
  - [Where to get help](#where-to-get-help)
  - [Contribution guidelines](#contribution-guidelines)
  - [Minimum Supported Rust Version (MSRV)](#minimum-supported-rust-version-msrv)
  - [Dependencies](#dependencies)
    - [Efferent (fan-out)](#efferent-fan-out)
    - [Build Dependencies](#build-dependencies)
    - [Development Dependencies](#development-dependencies)
    - [Afferent (fan-in)](#afferent-fan-in)
  - [Related projects](#related-projects)
  - [License](#license)


## Introduction

**CLASP** stands for **C**ommand-**L**ine **A**rgument **S**orting and **P**arsing. **CLASP.Rust** is an initial published scaffold with no supported public API. The reference implementation is the C/C++ [**CLASP**](https://github.com/synesissoftware/CLASP) library. The other implementations are listed under [Related projects](#related-projects).


## Developer Experience

A stable Rust toolchain at **MSRV 1.74** or newer is required. From the crate root:

```bash
cargo build
cargo test
./scripts/fmt
```

**scripts/fmt** runs the pinned nightly **rustfmt** described in **rustfmt.toml**. The stock example is **examples/versions/main.rs**.


## Installation

Reference in **Cargo.toml** in the usual way:

```toml
clasp-rs = { version = "0.0.1" }
```


## Components

### Constants

None defined at this time.


### Enumerations

None defined at this time.


### Features

The following crate features are defined:

* `null-feature` — a feature that has no effect (useful for simplifying driver scripts);


### Functions

None defined at this time.


### Macros

None defined at this time.


### Structures

None defined at this time.


### Traits

None defined at this time.


## Examples

Examples can be found in `examples/`:

* `examples/versions/main.rs` — prints the crate name and version;


## Project Information

### Where to get help

[GitHub Page](https://github.com/synesissoftware/CLASP.Rust "GitHub Page")


### Contribution guidelines

Defect reports, feature requests, and pull requests are welcome on https://github.com/synesissoftware/CLASP.Rust.


### Minimum Supported Rust Version (MSRV)

The declared Minimum Supported Rust Version (MSRV) for **CLASP.Rust** is **1.74**.

This MSRV guarantee applies to the library crate itself, its runtime dependencies (`[dependencies]`), and its build dependencies (`[build-dependencies]`). Downstream consumers compiling this crate as a dependency are guaranteed that it builds cleanly on the declared MSRV toolchain.

Development dependencies (`[dev-dependencies]`, such as benchmarking frameworks like **criterion**) may require newer Rust toolchains for local development or performance testing. These dev-dependencies are never fetched or compiled by downstream consumers and do not affect the library's MSRV guarantee.


### Dependencies

#### Efferent (fan-out)

Libraries upon which **CLASP.Rust** depends:

* None currently.


#### Build Dependencies

None currently.


#### Development Dependencies

* [**test_help-rs**](https://github.com/synesissoftware/test_help-rs) — test helpers for unit tests;


#### Afferent (fan-in)

None currently.


### Related projects

* [**CLASP**](https://github.com/synesissoftware/CLASP) — the C/C++ reference implementation;
* [**CLASP.Go**](https://github.com/synesissoftware/CLASP.Go);
* [**CLASP.js**](https://github.com/synesissoftware/CLASP.js);
* [**CLASP.NET**](https://github.com/synesissoftware/CLASP.NET);
* [**CLASP.Python**](https://github.com/synesissoftware/CLASP.Python);
* [**CLASP.Ruby**](https://github.com/synesissoftware/CLASP.Ruby);
* [**libCLImate.Rust**](https://github.com/synesissoftware/libCLImate.Rust);


### License

**CLASP.Rust** is released under the 3-clause BSD license. See [LICENSE](./LICENSE) for details.


<!-- ########################### end of file ########################### -->
