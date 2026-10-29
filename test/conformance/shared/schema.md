# CLASP-conformance - Schema <!-- omit in toc -->

Version 1 records parsing and used-tracking. The C library in **CLASP/CLASP** is the oracle. **schema.json** is the machine-readable form of one case file.


## Table of Contents <!-- omit in toc -->

- [Case file](#case-file)
- [Capabilities](#capabilities)
- [Parse flags](#parse-flags)
- [Specifications](#specifications)
- [Arguments and result](#arguments-and-result)
- [Checks](#checks)
- [Language mapping](#language-mapping)
- [Driver protocol](#driver-protocol)
- [Provenance and verification](#provenance-and-verification)
- [Decisions](#decisions)
- [Future](#future)


## Case file

One JSON object per file. The file name is the identifier plus `.json`. Keys are lexicographic. Arrays and objects use one element per line. Indentation is two spaces.

| Field | Meaning |
| --- | --- |
| `id` | `topic.NNN`, three digits |
| `description` | What the case locks in |
| `capabilities` | Tags the port must implement; otherwise it skips the case |
| `flags` | C parse-flag names, OR-ed |
| `specifications` | Specification array, or `null` for a C `NULL` pointer |
| `argv` | C calling convention. `argv[0]` is the program name. An empty array means `argc` less than 1 |
| `checks` | Queries applied in order, after the parse |
| `expected` | Result after those checks |
| `provenance` | Where the expectation came from |
| `verified` | `true` only after an accepted compare has been recorded |
| `verifiedAgainst` | Present only when `verified` is `true`. C oracle identity |
| `notes` | Optional. Silent or conflicting C text is recorded here |

`expected.status` is `ok` or `undefined`. `undefined` is used when the C contract does not define a result. That object carries `note` and does not carry argument lists.


## Capabilities

A runner skips a case when it lacks any listed capability. It does not fail that case. The vocabulary is lexicographic, matching **schema.json**.

| Tag | C flag or behaviour |
| --- | --- |
| `bit-flags` | `clasp_checkAllFlags` / `clasp_checkAllMatchingFlags` and `bitFlags` on specifications |
| `dont-recognise-doublehyphen` | `CLASP_F_DONT_RECOGNISE_DOUBLEHYPHEN_TO_START_VALUES` |
| `preserve-original-order` | `CLASP_F_PRESERVE_ORIGINAL_ARGUMENT_ORDER`. No version-1 case |
| `single-hyphen-as-value` | `CLASP_F_TREAT_SINGLEHYPHEN_AS_VALUE` |
| `wildcards-apostrophe-quotes-windows` | `CLASP_F_DO_EXPAND_WILDCARDS_IN_APOSQUOTES_ON_WINDOWS`. No version-1 case |
| `wildcards-windows` | `CLASP_F_DONT_EXPAND_WILDCARDS_ON_WINDOWS`. No version-1 case |

Combined flags, valued aliases, and the `"--"` delimiter are core behaviour. They are not skip-tags. A port that lacks them fails those cases.


## Parse flags

Names are the C macros. Order in a case file is lexicographic. Values are the header values in **include/clasp/clasp.h**.

* `CLASP_F_DONT_EXPAND_WILDCARDS_ON_WINDOWS`;
* `CLASP_F_DONT_RECOGNISE_DOUBLEHYPHEN_TO_START_VALUES`;
* `CLASP_F_DO_EXPAND_WILDCARDS_IN_APOSQUOTES_ON_WINDOWS`;
* `CLASP_F_PRESERVE_ORIGINAL_ARGUMENT_ORDER`;
* `CLASP_F_TREAT_SINGLEHYPHEN_AS_VALUE`;

`CLASP_F_PRESERVE_ORIGINAL_ARGUMENT_ORDER` changes the layout of the result pointers. Version 1 does not exercise it. A later case must say what `arguments`, `flags`, `options`, and `values` mean when that flag is set.


## Specifications

`null` is a `NULL` pointer. `[]` is a non-NULL array whose only element is the terminator. Those two are different: a `NULL` pointer does not split a short token, and an empty array does. See [Decisions](#decisions).

Each element:

| Field | C member |
| --- | --- |
| `type` | `flag`, `option`, `gap`, `tacit`, or `section` |
| `alias` | `name`. JSON `null` is a `NULL` pointer |
| `mappedArgument` | `mappedArgument`. JSON `null` is a `NULL` pointer |
| `help` | `help`. JSON `null` is a `NULL` pointer. `""` is a present empty string |
| `valueSet` | `valueSet`. Same null rule. Not exercised by version 1 |
| `bitFlags` | `bitFlags` |
| `section` | When `type` is `section`, the positive section number stored in `type`. Otherwise `null` |

`gap` and `tacit` are not arguments. They still occupy a slot, so `aliasIndex` counts them.


## Arguments and result

On the normative path `expected.status` is `ok`. `programName` is the non-Windows form: the last slash-separated component of `argv[0]`, with no extension stripped.

`arguments` is the sorted array. Default order is flag, then option, then value. Within one type the order is `originalIndex`. The C comparator returns 0 when two resolved names share a prefix at the same index; version-1 cases avoid that pair, and the order is unspecified if it occurs.

Each argument:

| Field | Meaning |
| --- | --- |
| `type` | `flag`, `option`, or `value` |
| `resolvedName` | Name after specification lookup. JSON `null` if the C pointer is `NULL` |
| `givenName` | Name as written, without an attached value. JSON `null` if the C pointer is `NULL` |
| `givenHyphens` | `numGivenHyphens` |
| `value` | JSON `null` if the C pointer is `NULL` (absent). `""` if the pointer is non-`NULL` and the length is 0 |
| `originalIndex` | Index in `argv`, program name at 0 |
| `aliasIndex` | Specification index, or `-1` |
| `used` | Whether the used bit is set after `checks` |

`flags`, `options`, and `values` are indexes into `arguments`, in partition order.

`unusedFlags`, `unusedOptions`, `unusedFlagsAndOptions`, `unusedValues`, and `unusedArguments` are the `nSkip` 0 results of the corresponding `clasp_reportUnused*` functions, after `checks`. Indexes point into `arguments`.

`unrecognised` is the `nSkip` 0 result of `clasp_reportUnrecognisedFlagsAndOptions` when `specifications` is an array, including `[]`. It is JSON `null` when `specifications` is `null`, because that function's precondition rejects a `NULL` specification pointer.


## Checks

Checks run in order on one parse. A marking query sets the used bit. A nonmutating query does not. Report queries are nonmutating; version-1 cases record their results in `expected` rather than as check steps, and the names are still part of the vocabulary so a runner can implement one set of operations.

| Operation | C function | Effect |
| --- | --- | --- |
| `flag-is-specified` | `clasp_flagIsSpecified` | Marks the first matching flag. `result` is boolean |
| `check-all-flags` | `clasp_checkAllFlags` | Marks each matched flag, including a flag whose `bitFlags` is 0. `result` is the OR of the non-zero bit values |
| `check-all-matching-flags` | `clasp_checkAllMatchingFlags` | Marks each flag whose specification bits overlap `mask`. `result` is the OR written to the out-parameter |
| `find-flag-or-option` | `clasp_findFlagOrOption` | Marks only the returned argument. `skip` is `nSkip`. `result` is `null` or an object with `originalIndex`, `resolvedName`, `type`, and `value` |
| `check-value` | `clasp_checkValue` | Marks the value at the value-partition `index` when that index exists. `present` follows the function body: `true` means the index exists. `value` is the C value pointer in the same null-or-string form as an argument |
| `report-unused-flags` | `clasp_reportUnusedFlags` | Nonmutating. `skip` and `count` |
| `report-unused-options` | `clasp_reportUnusedOptions` | Nonmutating. `skip` and `count` |
| `report-unused-flags-and-options` | `clasp_reportUnusedFlagsAndOptions` | Nonmutating. `skip` and `count` |
| `report-unused-values` | `clasp_reportUnusedValues` | Nonmutating. `skip` and `count` |
| `report-unused-arguments` | `clasp_reportUnusedArguments` | Nonmutating. `skip` and `count` |
| `report-unrecognised-flags-and-options` | `clasp_reportUnrecognisedFlagsAndOptions` | Nonmutating. `skip` and `count`. Not called when the specification pointer is `NULL` |

`check-all-flags` and `check-all-matching-flags` take the case specifications unless the check object has its own `specifications` array. An empty array on the check is a non-`NULL` terminator, not a `NULL` pointer.

`clasp_flagIsSpecified` is the marking query. There is no separate nonmutating "is specified" in the C API that leaves the bit clear. A runner that needs a nonmutating probe must not call the marking function.


## Language mapping

A runner adapts the call, then performs the same query. Where a port has no function, the cell is `none`.

| Operation | Go | JavaScript | .NET | Python | Ruby |
| --- | --- | --- | --- | --- | --- |
| `flag-is-specified` | `FlagIsSpecified` | `flagIsSpecified` | `ArgumentUtil.FlagSpecified` | `flag_is_specified` | `none` |
| `check-all-flags` | `none` (bits are applied in `Parse`) | `none` | `none` | `none` | `none` |
| `check-all-matching-flags` | `none` | `none` | `none` | `none` | `none` |
| `find-flag-or-option` | `LookupFlag` or `LookupOption`, `nSkip` `none` | `lookupFlag` or `lookupOption`, `nSkip` `none` | `FlagSpecified` or `FindOption`, `nSkip` `none` | `lookup_flag` or `lookup_option`, `nSkip` `none` | `find_flag` or `find_option`, no used bit |
| `check-value` | `none` | `none` | `none` | `none` | `none` |
| `report-unused-flags` | `GetUnusedFlags` | `getFirstUnusedFlag` (first only) | `VerifyAllFlagsUsed` (throws) | `get_first_unused_flag` (first only) | `none` |
| `report-unused-options` | `GetUnusedOptions` | `getFirstUnusedOption` | `VerifyAllOptionsUsed` (throws) | `get_first_unused_option` | `none` |
| `report-unused-flags-and-options` | `GetUnusedFlagsAndOptions` | `getFirstUnusedFlagOrOption` | `VerifyAllFlagsAndOptionsUsed` | `get_first_unused_flag_or_option` | `none` |
| `report-unused-values` | `none` | `none` | `none` | `none` | `none` |
| `report-unused-arguments` | `none` | `none` | `none` | `none` | `none` |
| `report-unrecognised-flags-and-options` | `none` | `none` | `none` | `none` | `find_first_unknown` (first only; not the used bit) |

The idioms are not drop-in substitutes. **DIVERGENCES.md** records where the behaviour differs. Calling-convention adaptations, applied by the runner before comparison:

* C, Go, and Python treat `argv[0]` as the program name and parse the rest;
* Ruby parses every element. Default `ARGV` excludes the program. Indexes are indexes of the array that was passed;
* JavaScript `_parse` starts at index 2. The program name comes from `process.argv[1]` unless the caller passes one;
* .NET `Arguments` starts at index 0. `ProgramName` is `Process.GetCurrentProcess().ProcessName`.


## Driver protocol

**scripts/run_oracle.py** is the only program that reads case JSON. It speaks a line protocol to **oracle/main.c**. The C program's leading comment is the byte layout. This section records the split.

The request order is `flags`, `argv`, `specs`, `checks`, `end`. A string is the line `null`, or a decimal byte count, that many bytes, and a newline. A specification group is the line `null` or a count. `null` is a C `NULL` pointer. A count of 0 is a non-NULL array whose only element is the terminator. Inside a check, `null` means "use the request specifications".

The script does not start the C program when `argv` is empty. `--language` selects the driver. Only `c` is implemented. Ruby is the next language, and it is not in this version. The script rebuilds `checks` and `expected` from the C result. It copies each check's input fields, including a check-local `specifications` array, rather than asking C to echo them.


## Provenance and verification

`provenance.kind` is `unit-test`, `source`, `header`, or `readme`. `provenance.path` is the workspace path. `provenance.symbol` is the function or test when there is one.

`verified` is `false`, and `verifiedAgainst` is absent, until `python3 scripts/run_oracle.py --compare` has agreed for that file and the agreement has been recorded. The compare does not write either field. Recording sets `verified` to `true` and writes `verifiedAgainst`:

| Field | Meaning |
| --- | --- |
| `language` | `C`. This object is the C oracle. Port runs do not write it |
| `macro` | `CLASP_VER` |
| `packed` | `CLASP_VER` as `0x` and eight hex digits |
| `revision` | Git revision of **CLASP/CLASP** that was linked |
| `version` | Four dotted fields, one per byte of `CLASP_VER` |

A version macro can stay unchanged across a parser edit, so `revision` pins the tree. Changing `expected` removes `verifiedAgainst` and sets `verified` back to `false`.


## Decisions

These are places where the header or the README is silent or disagrees with the function body. The corpus follows the body.

* An absent value is a `NULL` pointer, JSON `null`. A present empty value is a non-`NULL` pointer of length 0, JSON `""`. **src/clasp.api.c** stores the pointer that way for `"--name="`;
* `clasp_findFlagOrOption` on a bare flag stores a `NULL` value pointer. **test/unit/parse/findflagoroption/entry.c** `test_1_1` compares that value with `""`. The case follows the pointer. See **find-flag-or-option.002**;
* The `clasp_checkValue` comment says a return of 0 means the value exists (**include/clasp/clasp.h** around the retval lines). The body returns 1 when the index exists (**src/clasp.api.c** `clasp_checkValue`). `present` follows the body;
* In the double-hyphen arm, `numGivenHyphens` is the constant 2, not a count of hyphen characters (**src/clasp.api.c** around the assignment `numGivenHyphens = 2`). `"---verbose"` is therefore 2. See **edges.006**;
* Name lookup uses `strncmp` of the argument length against `spec->name` and does not require the specification name to end there (**src/clasp.api.c** `clasp_lookup_spec_for_name_`). A specification named `"-verbose"` matches `"-v"`. See **aliases.004**;
* `clasp_strchreq_` searches both `'='` and `':'` (**src/clasp.string.c**). `"--name:file"` is an option. The README shows `'='` only. See **options.008**;
* A long option consumes the next argument only when that next argument is not itself a recognised argument (**src/clasp.api.c**, the block that requires `clasp_lookup_spec_for_name_` of the next argument to be `NULL`). A short option whose specification type is option sets the next-is-value flag unconditionally (**src/clasp.api.c**, the `CLASP_ARGTYPE_OPTION` assignment of `nextArgumentIsValue`). See **options.006** and **options.007**;
* `argc` less than 1 is undefined (**include/clasp/clasp.h** `clasp_parseArguments`). **edges.001** does not invent an empty result;
* `aliasIndex` is the specification pointer difference, so a gap occupies an index (**src/clasp.api.c** `clasp_evaluate_spec_index_`).


## Future

Not in version 1, and not represented by cases:

* usage text and version text;
* typed values, ranges, defaults, and required options;
* a result shape for `preserve-original-order` (the capability tag exists);
* Windows wildcard expansion, including apostrophe-quoted wildcards (the capability tags exist).

Bit flags are in version 1 as optional-capability cases under **cases/bit-flags.*.json**.


<!-- ########################### end of file ########################### -->
