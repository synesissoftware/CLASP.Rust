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
- [Conformance](#conformance)
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

**CLASP** stands for **C**ommand-**L**ine **A**rgument **S**orting and **P**arsing. **CLASP.Rust** parses a command line into flags, options, and values, using the same rules as the C/C++ [**CLASP**](https://github.com/synesissoftware/CLASP) library. The other implementations are listed under [Related projects](#related-projects).


## Developer Experience

A stable Rust toolchain at **MSRV 1.74** or newer is required. From the crate root:

```bash
cargo build
cargo test
./scripts/fmt
```

**scripts/fmt** runs the pinned nightly **rustfmt** described in **rustfmt.toml**. The stock example is **examples/versions/main.rs**. The smallest parse is **examples/minimal-usage/main.rs**:

```rust
use clasp_rs::{
    parse,
    ParseFlags,
    Specification,
};

fn main() {
    let specifications = [
        Specification::flag(Some("-v"), Some("--verbose"), None),
        Specification::option(Some("-n"), Some("--name"), Some("a name"), None),
    ];
    let argv = std::env::args().collect::<Vec<_>>();
    let parsed = parse(ParseFlags::empty(), &argv, Some(&specifications));

    println!("{}", parsed.program_name());

    for argument in parsed.flags() {
        println!("  {argument}");
    }
    for argument in parsed.options() {
        println!("  {argument}");
    }
    for argument in parsed.values() {
        println!("  {argument}");
    }
    for argument in parsed.unused_arguments() {
        println!("  {argument}");
    }
}
```

That call sorts flags, then options, then values. Marking queries such as `flag_is_specified` set the used bit; printing the partitions does not. See [Conformance](#conformance) for the C corpus.


## Installation

Reference in **Cargo.toml** in the usual way:

```toml
clasp-rs = { version = "0.1.0" }
```


## Components

### Constants

Parse flags, on [`ParseFlags`](https://docs.rs/clasp-rs/latest/clasp_rs/struct.ParseFlags.html). The values match the C header:

* `DONT_EXPAND_WILDCARDS_ON_WINDOWS` — accepted, and it has no effect (version 1 does not expand wildcards);
* `DONT_RECOGNISE_DOUBLEHYPHEN_TO_START_VALUES`;
* `DO_EXPAND_WILDCARDS_IN_APOSQUOTES_ON_WINDOWS` — accepted, and it has no effect;
* `PRESERVE_ORIGINAL_ARGUMENT_ORDER` — the result stays in `argv` order and the three partitions are empty;
* `TREAT_SINGLEHYPHEN_AS_VALUE`;


### Enumerations

* [`ArgumentKind`](https://docs.rs/clasp-rs/latest/clasp_rs/enum.ArgumentKind.html) — `Flag`, `Option`, `Value`;
* [`SpecificationType`](https://docs.rs/clasp-rs/latest/clasp_rs/enum.SpecificationType.html) — `Flag`, `Gap`, `Option`, `Section`, `Tacit`;


### Features

The following crate features are defined:

* `null-feature` — a feature that has no effect (useful for simplifying driver scripts);


### Functions

* [`parse`](https://docs.rs/clasp-rs/latest/clasp_rs/fn.parse.html) — sort `argv` into flags, options, and values;


### Macros

None defined at this time.


### Structures

* [`Argument`](https://docs.rs/clasp-rs/latest/clasp_rs/struct.Argument.html) — one parsed flag, option, or value, including the used bit;
* [`Arguments`](https://docs.rs/clasp-rs/latest/clasp_rs/struct.Arguments.html) — the program name, the sorted arguments, and the three partitions;
* [`ParseFlags`](https://docs.rs/clasp-rs/latest/clasp_rs/struct.ParseFlags.html) — the C parse-flag bit set;
* [`Report`](https://docs.rs/clasp-rs/latest/clasp_rs/struct.Report.html) — one hit from a nonmutating report query;
* [`Specification`](https://docs.rs/clasp-rs/latest/clasp_rs/struct.Specification.html) — one alias, mapped argument, help string, value set, bit value, or section;


### Traits

* [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) for [`Argument`](https://docs.rs/clasp-rs/latest/clasp_rs/struct.Argument.html) and [`ArgumentKind`](https://docs.rs/clasp-rs/latest/clasp_rs/enum.ArgumentKind.html);


## Examples

Examples can be found in `examples/`:

* `examples/minimal-usage/main.rs` — parses the process arguments and prints flags, options, values, and unused arguments;
* `examples/versions/main.rs` — prints the crate name and version;


## Conformance

The crate carries its own corpus and does not read a sibling checkout at test time.

* **test/conformance/shared/** is a byte-for-byte snapshot of the shared corpus (`cases/*.json`, **schema.md**, **schema.json**) plus **MANIFEST**. The current snapshot is revision `ba102d36eb29398a643002d57702e3d3729dcbc4`. Do not edit those files by hand;
* **test/conformance/local/** holds cases whose ids start with `rust.`. Each has `"verified": false`. They are Rust-authored, derived from named C lines, and candidates to contribute back to the shared corpus;
* **scripts/sync_conformance.sh** refreshes **shared/** from a committed revision of **CLASP-conformance** (`git archive`, default `HEAD`). It refuses a revision that does not resolve, rewrites **MANIFEST**, prints the revision, and warns when that working tree is dirty. It does not copy uncommitted files;

The runner skips a case when **CLASP.Rust** does not implement every listed capability, and it skips `expected.status` of `undefined` with the reason `undefined in C contract`. Implemented capabilities are `bit-flags`, `dont-recognise-doublehyphen`, and `single-hyphen-as-value`. `preserve-original-order` and the two wildcard capabilities are not claimed. The core (no capability tag) is always run.

`CLASP_CONFORMANCE_DIR` may point at a live corpus root for a developer check. The runner says so, and it still prints the in-tree snapshot revision. The test directories are listed in **Cargo.toml** `exclude`, so they are not part of the published package.

```bash
cargo test --test conformance -- --nocapture
cargo test --test unit -- --nocapture
```


## Project Information

### Where to get help

[GitHub Page](https://github.com/synesissoftware/CLASP.Rust "GitHub Page")


### Contribution guidelines

Defect reports, feature requests, and pull requests are welcome on https://github.com/synesissoftware/CLASP.Rust.


### Minimum Supported Rust Version (MSRV)

The declared Minimum Supported Rust Version (MSRV) for **CLASP.Rust** is **1.74**.

This MSRV guarantee applies to the library crate itself, its runtime dependencies (`[dependencies]`), and its build dependencies (`[build-dependencies]`). Downstream consumers compiling this crate as a dependency are guaranteed that it builds cleanly on the declared MSRV toolchain.

Development dependencies (`[dev-dependencies]`) may require newer Rust toolchains for local development or performance testing. **serde_json**, used only by the conformance runner, is in that class. These dev-dependencies are never fetched or compiled by downstream consumers and do not affect the library's MSRV guarantee.


### Dependencies

#### Efferent (fan-out)

Libraries upon which **CLASP.Rust** depends:

* None currently.


#### Build Dependencies

None currently.


#### Development Dependencies

* [**serde_json**](https://github.com/serde-rs/json) — loads conformance case files. `default-features = false`, feature `std` only. It is not a library dependency;
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
