// test/conformance/main.rs : component runner for the in-tree corpus

#![allow(non_snake_case)]


use clasp_rs::{
    Argument,
    ArgumentKind,
    Arguments,
    ParseFlags,
    Specification,
};

use serde_json::{
    Map,
    Value,
};

use std::{
    env as std_env,
    fs as std_fs,
    path::{
        Path,
        PathBuf,
    },
};


const IMPLEMENTED : &[&str] = &["bit-flags", "dont-recognise-doublehyphen", "single-hyphen-as-value"];


struct CaseFile {
    id :             String,
    capabilities :   Vec<String>,
    flags :          ParseFlags,
    specifications : Option<Vec<Specification>>,
    argv :           Vec<String>,
    checks :         Vec<Check>,
    expected :       Expected,
}


struct Check {
    operation :      String,
    name :           Option<String>,
    skip :           u32,
    index :          usize,
    mask :           i32,
    specifications : Option<Vec<Specification>>,
    result :         Value,
    present :        Option<bool>,
    value :          Value,
}


enum Expected {
    Undefined,
    Ok(Box<ExpectedOk>),
}


struct ExpectedOk {
    program_name :             String,
    arguments :                Vec<ExpectedArgument>,
    flags :                    Vec<i64>,
    options :                  Vec<i64>,
    values :                   Vec<i64>,
    unrecognised :             Option<Vec<i64>>,
    unused_arguments :         Vec<i64>,
    unused_flags :             Vec<i64>,
    unused_flags_and_options : Vec<i64>,
    unused_options :           Vec<i64>,
    unused_values :            Vec<i64>,
}


struct ExpectedArgument {
    kind :           ArgumentKind,
    resolved_name :  Option<String>,
    given_name :     Option<String>,
    value :          Option<String>,
    original_index : i64,
    given_hyphens :  i64,
    alias_index :    i64,
    used :           bool,
}


struct RunTotals {
    found :    usize,
    run :      usize,
    passed :   usize,
    failed :   usize,
    skipped :  usize,
    skips :    Vec<(String, usize)>,
    failures : Vec<String>,
}


#[test]
fn TEST_CONFORMANCE_CORPUS() {
    let totals = execute();
    print_totals(&totals);

    assert!(totals.failures.is_empty(), "{}", totals.failures.join("\n"),);
}


fn execute() -> RunTotals {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let local_dir = root.join("test/conformance/local");
    let (shared_dir, _) = shared_directory(&root);
    let mut failures = Vec::new();
    let mut cases = Vec::new();

    match load_directory(&shared_dir) {
        Ok(mut loaded) => cases.append(&mut loaded),
        Err(error) => failures.push(format!("shared\n  expected: readable cases\n  actual:   {error}")),
    }
    match load_directory(&local_dir) {
        Ok(mut loaded) => cases.append(&mut loaded),
        Err(error) => failures.push(format!("local\n  expected: readable cases\n  actual:   {error}")),
    }

    let mut skips : Vec<(String, usize)> = Vec::new();
    let mut run = 0usize;
    let mut passed = 0usize;
    let found = cases.len();

    for case in &cases {
        if let Some(reason) = skip_reason(case) {
            bump_skip(&mut skips, reason);
            continue;
        }

        run += 1;
        let mut case_failures = Vec::new();
        compare_case(case, &mut case_failures);
        if case_failures.is_empty() {
            passed += 1;
        } else {
            failures.extend(case_failures);
        }
    }

    skips.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));

    RunTotals {
        found,
        run,
        passed,
        failed : run - passed,
        skipped : found - run,
        skips,
        failures,
    }
}


fn print_totals(totals : &RunTotals) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = root.join("test/conformance/shared/MANIFEST");
    let snapshot = read_snapshot(&manifest_path);
    let local_dir = root.join("test/conformance/local");
    let (shared_dir, live) = shared_directory(&root);

    println!("CLASP.Rust conformance");
    if live {
        println!(
            "shared: {} (CLASP_CONFORMANCE_DIR; the in-tree snapshot was not used)",
            shared_dir.display(),
        );
    } else {
        println!("shared: {}", shared_dir.display());
    }
    println!("local: {}", local_dir.display());
    println!("snapshot: {snapshot}");
    println!("capabilities: {}", IMPLEMENTED.join(", "));
    println!(
        "found: {}  run: {}  passed: {}  failed: {}  skipped: {}",
        totals.found, totals.run, totals.passed, totals.failed, totals.skipped,
    );
    if totals.skips.is_empty() {
        println!("skips: none");
    } else {
        println!("skips:");
        for (reason, count) in &totals.skips {
            println!("  {reason}: {count}");
        }
    }
    if !totals.failures.is_empty() {
        println!("failures:");
        for failure in &totals.failures {
            println!("{failure}");
        }
    }
}


fn shared_directory(root : &Path) -> (PathBuf, bool) {
    if let Some(live) = std_env::var_os("CLASP_CONFORMANCE_DIR") {
        let live = PathBuf::from(live);
        let cases = live.join("cases");

        if cases.is_dir() {
            return (cases, true);
        }
        return (live, true);
    }

    (root.join("test/conformance/shared/cases"), false)
}


fn read_snapshot(path : &Path) -> String {
    let Ok(text) = std_fs::read_to_string(path) else {
        return "missing".to_string();
    };

    for line in text.lines() {
        if let Some(revision) = line.strip_prefix("revision: ") {
            return revision.to_string();
        }
    }

    "missing".to_string()
}


fn bump_skip(
    skips : &mut Vec<(String, usize)>,
    reason : String,
) {
    if let Some((_, count)) = skips.iter_mut().find(|(name, _)| name == &reason) {
        *count += 1;
    } else {
        skips.push((reason, 1));
    }
}


fn skip_reason(case : &CaseFile) -> Option<String> {
    if matches!(case.expected, Expected::Undefined) {
        return Some("undefined in C contract".to_string());
    }

    let mut missing = Vec::new();

    for capability in &case.capabilities {
        if !IMPLEMENTED.contains(&capability.as_str()) {
            missing.push(capability.clone());
        }
    }
    if missing.is_empty() {
        return None;
    }
    missing.sort();
    Some(format!("capability: {}", missing.join(", ")))
}


fn load_directory(dir : &Path) -> Result<Vec<CaseFile>, String> {
    if !dir.is_dir() {
        return Err(format!("not a directory: {}", dir.display()));
    }

    let mut names = Vec::new();
    for entry in std_fs::read_dir(dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            names.push(path);
        }
    }
    names.sort();

    let mut cases = Vec::new();

    for path in names {
        let text = std_fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let value : Value = serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        cases.push(parse_case(&path, &value)?);
    }

    Ok(cases)
}


fn parse_case(
    path : &Path,
    value : &Value,
) -> Result<CaseFile, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{}: case is not an object", path.display()))?;
    let id = required_string(object, "id", path)?;
    let capabilities = required_string_array(object, "capabilities", path)?;
    let flag_names = required_string_array(object, "flags", path)?;
    let flags = parse_flag_names(&flag_names).map_err(|error| format!("{id}: {error}"))?;
    let specifications = parse_specifications(object.get("specifications"), path)?;
    let argv = required_string_array(object, "argv", path)?;
    let checks = parse_checks(object.get("checks"), path)?;
    let expected = parse_expected(object.get("expected"), path)?;

    Ok(CaseFile {
        id,
        capabilities,
        flags,
        specifications,
        argv,
        checks,
        expected,
    })
}


fn parse_flag_names(names : &[String]) -> Result<ParseFlags, String> {
    let mut bits = 0u32;

    for name in names {
        let bit = match name.as_str() {
            "CLASP_F_DONT_EXPAND_WILDCARDS_ON_WINDOWS" => ParseFlags::DONT_EXPAND_WILDCARDS_ON_WINDOWS,
            "CLASP_F_DONT_RECOGNISE_DOUBLEHYPHEN_TO_START_VALUES" => {
                ParseFlags::DONT_RECOGNISE_DOUBLEHYPHEN_TO_START_VALUES
            },
            "CLASP_F_DO_EXPAND_WILDCARDS_IN_APOSQUOTES_ON_WINDOWS" => {
                ParseFlags::DO_EXPAND_WILDCARDS_IN_APOSQUOTES_ON_WINDOWS
            },
            "CLASP_F_PRESERVE_ORIGINAL_ARGUMENT_ORDER" => ParseFlags::PRESERVE_ORIGINAL_ARGUMENT_ORDER,
            "CLASP_F_TREAT_SINGLEHYPHEN_AS_VALUE" => ParseFlags::TREAT_SINGLEHYPHEN_AS_VALUE,
            other => return Err(format!("unknown parse flag {other}")),
        };

        bits |= bit;
    }

    Ok(ParseFlags::from_bits(bits))
}


fn parse_specifications(
    value : Option<&Value>,
    path : &Path,
) -> Result<Option<Vec<Specification>>, String> {
    let Some(value) = value else {
        return Err(format!("{}: missing specifications", path.display()));
    };
    if value.is_null() {
        return Ok(None);
    }

    let array = value
        .as_array()
        .ok_or_else(|| format!("{}: specifications is not an array", path.display()))?;
    let mut specifications = Vec::new();

    for (index, item) in array.iter().enumerate() {
        specifications.push(parse_specification(item, path, index)?);
    }

    Ok(Some(specifications))
}


fn parse_specification(
    value : &Value,
    path : &Path,
    index : usize,
) -> Result<Specification, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{}: specification {index} is not an object", path.display()))?;
    let kind = required_string(object, "type", path)?;
    let alias = optional_string(object, "alias")?;
    let mapped = optional_string(object, "mappedArgument")?;
    let help = optional_string(object, "help")?;
    let value_set = optional_string(object, "valueSet")?;
    let bit_flags = required_i32(object, "bitFlags", path)?;
    let section = match object.get("section") {
        None | Some(Value::Null) => None,
        Some(value) => {
            Some(
                value
                    .as_i64()
                    .ok_or_else(|| format!("{}: section is not an integer", path.display()))? as i32,
            )
        },
    };

    let spec = match kind.as_str() {
        "flag" => Specification::bit_flag(alias.as_deref(), mapped.as_deref(), bit_flags, help.as_deref()),
        "option" => {
            Specification::option(
                alias.as_deref(),
                mapped.as_deref(),
                help.as_deref(),
                value_set.as_deref(),
            )
        },
        "gap" => Specification::gap(help.as_deref()),
        "tacit" => Specification::tacit(),
        "section" => Specification::section(section.unwrap_or(0)),
        other => return Err(format!("{}: unknown specification type {other}", path.display())),
    };

    Ok(spec)
}


fn parse_checks(
    value : Option<&Value>,
    path : &Path,
) -> Result<Vec<Check>, String> {
    let Some(value) = value else {
        return Err(format!("{}: missing checks", path.display()));
    };
    let array = value
        .as_array()
        .ok_or_else(|| format!("{}: checks is not an array", path.display()))?;
    let mut checks = Vec::new();

    for item in array {
        let object = item
            .as_object()
            .ok_or_else(|| format!("{}: check is not an object", path.display()))?;
        let specifications = if object.contains_key("specifications") {
            parse_specifications(object.get("specifications"), path)?
        } else {
            None
        };

        checks.push(Check {
            operation : required_string(object, "operation", path)?,
            name : match object.get("name") {
                Some(Value::String(text)) => Some(text.clone()),
                _ => None,
            },
            skip : match object.get("skip") {
                Some(value) => {
                    value
                        .as_u64()
                        .ok_or_else(|| format!("{}: skip is not an integer", path.display()))?
                        as u32
                },
                None => 0,
            },
            index : match object.get("index") {
                Some(value) => {
                    value
                        .as_u64()
                        .ok_or_else(|| format!("{}: index is not an integer", path.display()))?
                        as usize
                },
                None => 0,
            },
            mask : match object.get("mask") {
                Some(value) => {
                    value
                        .as_i64()
                        .ok_or_else(|| format!("{}: mask is not an integer", path.display()))?
                        as i32
                },
                None => 0,
            },
            specifications,
            result : object.get("result").cloned().unwrap_or(Value::Null),
            present : match object.get("present") {
                Some(Value::Bool(flag)) => Some(*flag),
                _ => None,
            },
            value : object.get("value").cloned().unwrap_or(Value::Null),
        });
    }

    Ok(checks)
}


fn parse_expected(
    value : Option<&Value>,
    path : &Path,
) -> Result<Expected, String> {
    let object = value
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}: expected is not an object", path.display()))?;
    let status = required_string(object, "status", path)?;

    if status == "undefined" {
        return Ok(Expected::Undefined);
    }
    if status != "ok" {
        return Err(format!("{}: unknown status {status}", path.display()));
    }

    Ok(Expected::Ok(Box::new(ExpectedOk {
        program_name :             required_string(object, "programName", path)?,
        arguments :                parse_expected_arguments(object.get("arguments"), path)?,
        flags :                    required_i64_array(object, "flags", path)?,
        options :                  required_i64_array(object, "options", path)?,
        values :                   required_i64_array(object, "values", path)?,
        unrecognised :             parse_optional_indexes(object.get("unrecognised"), path)?,
        unused_arguments :         required_i64_array(object, "unusedArguments", path)?,
        unused_flags :             required_i64_array(object, "unusedFlags", path)?,
        unused_flags_and_options : required_i64_array(object, "unusedFlagsAndOptions", path)?,
        unused_options :           required_i64_array(object, "unusedOptions", path)?,
        unused_values :            required_i64_array(object, "unusedValues", path)?,
    })))
}


fn parse_expected_arguments(
    value : Option<&Value>,
    path : &Path,
) -> Result<Vec<ExpectedArgument>, String> {
    let array = value
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}: arguments is not an array", path.display()))?;
    let mut arguments = Vec::new();

    for item in array {
        let object = item
            .as_object()
            .ok_or_else(|| format!("{}: argument is not an object", path.display()))?;
        let kind = match required_string(object, "type", path)?.as_str() {
            "flag" => ArgumentKind::Flag,
            "option" => ArgumentKind::Option,
            "value" => ArgumentKind::Value,
            other => return Err(format!("{}: unknown argument type {other}", path.display())),
        };

        arguments.push(ExpectedArgument {
            kind,
            resolved_name : optional_string(object, "resolvedName")?,
            given_name : optional_string(object, "givenName")?,
            value : optional_string(object, "value")?,
            original_index : required_i64(object, "originalIndex", path)?,
            given_hyphens : required_i64(object, "givenHyphens", path)?,
            alias_index : required_i64(object, "aliasIndex", path)?,
            used : object
                .get("used")
                .and_then(Value::as_bool)
                .ok_or_else(|| format!("{}: used is not a boolean", path.display()))?,
        });
    }

    Ok(arguments)
}


fn parse_optional_indexes(
    value : Option<&Value>,
    path : &Path,
) -> Result<Option<Vec<i64>>, String> {
    match value {
        None => Err(format!("{}: missing unrecognised", path.display())),
        Some(Value::Null) => Ok(None),
        Some(value) => Ok(Some(i64_array(value, path)?)),
    }
}


fn compare_case(
    case : &CaseFile,
    failures : &mut Vec<String>,
) {
    let Expected::Ok(expected) = &case.expected else {
        return;
    };
    let parsed = clasp_rs::parse(case.flags, &case.argv, case.specifications.as_deref());

    run_checks(case, &parsed, failures);
    compare_arguments(case, expected, &parsed, failures);
    compare_indexes(
        case,
        "flags",
        &expected.flags,
        partition_indexes(parsed.flag_count(), 0),
        failures,
    );
    compare_indexes(
        case,
        "options",
        &expected.options,
        partition_indexes(parsed.option_count(), parsed.flag_count()),
        failures,
    );
    compare_indexes(
        case,
        "values",
        &expected.values,
        partition_indexes(parsed.value_count(), parsed.flag_count() + parsed.option_count()),
        failures,
    );
    compare_unrecognised(case, expected, &parsed, failures);
    compare_unused(case, expected, &parsed, failures);

    if parsed.program_name() != expected.program_name {
        push_diff(
            failures,
            &case.id,
            "programName",
            &show_text(&expected.program_name),
            &show_text(parsed.program_name()),
        );
    }
}


fn run_checks(
    case : &CaseFile,
    parsed : &Arguments,
    failures : &mut Vec<String>,
) {
    for (index, check) in case.checks.iter().enumerate() {
        let label = format!("checks[{index}].{}", check.operation);

        match check.operation.as_str() {
            "flag-is-specified" => {
                let name = check.name.as_deref().unwrap_or("");
                let actual = parsed.flag_is_specified(name);

                compare_bool(failures, &case.id, &format!("{label}.result"), &check.result, actual);
            },
            "check-all-flags" => {
                let mut bits = 0i32;
                let actual = parsed.check_all_flags(check.specifications.as_deref(), &mut bits);

                compare_i64(
                    failures,
                    &case.id,
                    &format!("{label}.result"),
                    &check.result,
                    i64::from(actual),
                );
            },
            "check-all-matching-flags" => {
                let mut bits = 0i32;
                let actual = parsed.check_all_matching_flags(check.specifications.as_deref(), check.mask, &mut bits);

                compare_i64(
                    failures,
                    &case.id,
                    &format!("{label}.result"),
                    &check.result,
                    i64::from(actual),
                );
            },
            "find-flag-or-option" => {
                let name = check.name.as_deref().unwrap_or("");
                let actual = parsed.find_flag_or_option(name, check.skip);

                compare_find(failures, &case.id, &label, &check.result, actual);
            },
            "check-value" => {
                let actual = parsed.check_value(check.index);
                let present = actual.is_some();

                if let Some(expected_present) = check.present {
                    if expected_present != present {
                        push_diff(
                            failures,
                            &case.id,
                            &format!("{label}.present"),
                            &expected_present.to_string(),
                            &present.to_string(),
                        );
                    }
                }
                let actual_value = actual.and_then(Argument::value);
                compare_optional_text(
                    failures,
                    &case.id,
                    &format!("{label}.value"),
                    &check.value,
                    actual_value,
                );
            },
            other => {
                push_diff(failures, &case.id, &label, "a known operation", other);
            },
        }
    }
}


fn compare_arguments(
    case : &CaseFile,
    expected : &ExpectedOk,
    parsed : &Arguments,
    failures : &mut Vec<String>,
) {
    if expected.arguments.len() != parsed.arguments().len() {
        push_diff(
            failures,
            &case.id,
            "arguments.len",
            &expected.arguments.len().to_string(),
            &parsed.arguments().len().to_string(),
        );
    }

    let count = expected.arguments.len().min(parsed.arguments().len());

    for index in 0..count {
        let want = &expected.arguments[index];
        let got = &parsed.arguments()[index];
        let prefix = format!("arguments[{index}]");

        if want.kind != got.kind() {
            push_diff(
                failures,
                &case.id,
                &format!("{prefix}.type"),
                &want.kind.to_string(),
                &got.kind().to_string(),
            );
        }
        compare_opt_field(
            failures,
            &case.id,
            &format!("{prefix}.resolvedName"),
            want.resolved_name.as_deref(),
            got.resolved_name(),
        );
        compare_opt_field(
            failures,
            &case.id,
            &format!("{prefix}.givenName"),
            want.given_name.as_deref(),
            got.given_name(),
        );
        compare_opt_field(
            failures,
            &case.id,
            &format!("{prefix}.value"),
            want.value.as_deref(),
            got.value(),
        );
        if want.original_index != i64::from(got.original_index()) {
            push_diff(
                failures,
                &case.id,
                &format!("{prefix}.originalIndex"),
                &want.original_index.to_string(),
                &got.original_index().to_string(),
            );
        }
        if want.given_hyphens != i64::from(got.given_hyphens()) {
            push_diff(
                failures,
                &case.id,
                &format!("{prefix}.givenHyphens"),
                &want.given_hyphens.to_string(),
                &got.given_hyphens().to_string(),
            );
        }
        if want.alias_index != i64::from(got.alias_index()) {
            push_diff(
                failures,
                &case.id,
                &format!("{prefix}.aliasIndex"),
                &want.alias_index.to_string(),
                &got.alias_index().to_string(),
            );
        }
        if want.used != got.is_used() {
            push_diff(
                failures,
                &case.id,
                &format!("{prefix}.used"),
                &want.used.to_string(),
                &got.is_used().to_string(),
            );
        }
    }
}


fn compare_unrecognised(
    case : &CaseFile,
    expected : &ExpectedOk,
    parsed : &Arguments,
    failures : &mut Vec<String>,
) {
    if parsed.report_unrecognised_flags_and_options(None, 0).is_none() {
        if expected.unrecognised.is_some() {
            push_diff(failures, &case.id, "unrecognised", "an index list", "null");
        }
        return;
    }

    let actual = gather_reports(parsed, ReportKind::Unrecognised);

    match &expected.unrecognised {
        None => push_diff(failures, &case.id, "unrecognised", "null", &format!("{actual:?}")),
        Some(want) => compare_indexes(case, "unrecognised", want, actual, failures),
    }
}


fn compare_unused(
    case : &CaseFile,
    expected : &ExpectedOk,
    parsed : &Arguments,
    failures : &mut Vec<String>,
) {
    compare_indexes(
        case,
        "unusedArguments",
        &expected.unused_arguments,
        gather_reports(parsed, ReportKind::Arguments),
        failures,
    );
    compare_indexes(
        case,
        "unusedFlags",
        &expected.unused_flags,
        gather_reports(parsed, ReportKind::Flags),
        failures,
    );
    compare_indexes(
        case,
        "unusedFlagsAndOptions",
        &expected.unused_flags_and_options,
        gather_reports(parsed, ReportKind::FlagsAndOptions),
        failures,
    );
    compare_indexes(
        case,
        "unusedOptions",
        &expected.unused_options,
        gather_reports(parsed, ReportKind::Options),
        failures,
    );
    compare_indexes(
        case,
        "unusedValues",
        &expected.unused_values,
        gather_reports(parsed, ReportKind::Values),
        failures,
    );
}


enum ReportKind {
    Arguments,
    Flags,
    FlagsAndOptions,
    Options,
    Unrecognised,
    Values,
}


fn gather_reports(
    parsed : &Arguments,
    kind : ReportKind,
) -> Vec<i64> {
    let mut indexes = Vec::new();
    let mut skip = 0u32;

    loop {
        let report = match kind {
            ReportKind::Arguments => Some(parsed.report_unused_arguments(skip)),
            ReportKind::Flags => Some(parsed.report_unused_flags(skip)),
            ReportKind::FlagsAndOptions => Some(parsed.report_unused_flags_and_options(skip)),
            ReportKind::Options => Some(parsed.report_unused_options(skip)),
            ReportKind::Unrecognised => parsed.report_unrecognised_flags_and_options(None, skip),
            ReportKind::Values => Some(parsed.report_unused_values(skip)),
        };
        let Some(report) = report else {
            break;
        };

        if report.remaining == 0 {
            break;
        }
        let Some(argument) = report.first else {
            break;
        };
        let Some(index) = parsed.index_of(argument) else {
            break;
        };

        indexes.push(index as i64);
        skip = skip.saturating_add(1);
        if indexes.len() > parsed.arguments().len() {
            break;
        }
    }

    indexes
}


fn partition_indexes(
    count : usize,
    start : usize,
) -> Vec<i64> {
    (0..count).map(|offset| (start + offset) as i64).collect()
}


fn compare_indexes(
    case : &CaseFile,
    field : &str,
    expected : &[i64],
    actual : Vec<i64>,
    failures : &mut Vec<String>,
) {
    if expected != actual.as_slice() {
        push_diff(
            failures,
            &case.id,
            field,
            &format!("{expected:?}"),
            &format!("{actual:?}"),
        );
    }
}


fn compare_bool(
    failures : &mut Vec<String>,
    id : &str,
    field : &str,
    expected : &Value,
    actual : bool,
) {
    match expected.as_bool() {
        Some(want) if want == actual => {},
        Some(want) => push_diff(failures, id, field, &want.to_string(), &actual.to_string()),
        None => push_diff(failures, id, field, "a boolean", &actual.to_string()),
    }
}


fn compare_i64(
    failures : &mut Vec<String>,
    id : &str,
    field : &str,
    expected : &Value,
    actual : i64,
) {
    match expected.as_i64() {
        Some(want) if want == actual => {},
        Some(want) => push_diff(failures, id, field, &want.to_string(), &actual.to_string()),
        None => push_diff(failures, id, field, "an integer", &actual.to_string()),
    }
}


fn compare_find(
    failures : &mut Vec<String>,
    id : &str,
    field : &str,
    expected : &Value,
    actual : Option<&Argument>,
) {
    if expected.is_null() {
        if actual.is_some() {
            push_diff(failures, id, &format!("{field}.result"), "null", "an argument");
        }
        return;
    }

    let Some(object) = expected.as_object() else {
        push_diff(
            failures,
            id,
            &format!("{field}.result"),
            "an object or null",
            &expected.to_string(),
        );
        return;
    };
    let Some(argument) = actual else {
        push_diff(failures, id, &format!("{field}.result"), "an argument", "null");
        return;
    };

    if let Some(want) = object.get("originalIndex").and_then(Value::as_i64) {
        if want != i64::from(argument.original_index()) {
            push_diff(
                failures,
                id,
                &format!("{field}.result.originalIndex"),
                &want.to_string(),
                &argument.original_index().to_string(),
            );
        }
    }
    if let Some(want) = object.get("type").and_then(Value::as_str) {
        if want != argument.kind().to_string() {
            push_diff(
                failures,
                id,
                &format!("{field}.result.type"),
                want,
                &argument.kind().to_string(),
            );
        }
    }
    compare_optional_text(
        failures,
        id,
        &format!("{field}.result.resolvedName"),
        object.get("resolvedName").unwrap_or(&Value::Null),
        argument.resolved_name(),
    );
    compare_optional_text(
        failures,
        id,
        &format!("{field}.result.value"),
        object.get("value").unwrap_or(&Value::Null),
        argument.value(),
    );
}


fn compare_optional_text(
    failures : &mut Vec<String>,
    id : &str,
    field : &str,
    expected : &Value,
    actual : Option<&str>,
) {
    let want = match expected {
        Value::Null => None,
        Value::String(text) => Some(text.as_str()),
        _ => {
            push_diff(failures, id, field, "a string or null", &expected.to_string());
            return;
        },
    };

    compare_opt_field(failures, id, field, want, actual);
}


fn compare_opt_field(
    failures : &mut Vec<String>,
    id : &str,
    field : &str,
    expected : Option<&str>,
    actual : Option<&str>,
) {
    if expected != actual {
        push_diff(failures, id, field, &show_opt(expected), &show_opt(actual));
    }
}


fn push_diff(
    failures : &mut Vec<String>,
    id : &str,
    field : &str,
    expected : &str,
    actual : &str,
) {
    failures.push(format!("{id} {field}\n  expected: {expected}\n  actual:   {actual}"));
}


fn show_opt(value : Option<&str>) -> String {
    match value {
        None => "null".to_string(),
        Some(text) => format!("{text:?}"),
    }
}


fn show_text(value : &str) -> String {
    format!("{value:?}")
}


fn required_string(
    object : &Map<String, Value>,
    key : &str,
    path : &Path,
) -> Result<String, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{}: missing string {key}", path.display()))
}


fn required_string_array(
    object : &Map<String, Value>,
    key : &str,
    path : &Path,
) -> Result<Vec<String>, String> {
    let array = object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}: missing array {key}", path.display()))?;
    let mut values = Vec::new();

    for item in array {
        let text = item
            .as_str()
            .ok_or_else(|| format!("{}: {key} item is not a string", path.display()))?;

        values.push(text.to_string());
    }

    Ok(values)
}


fn optional_string(
    object : &Map<String, Value>,
    key : &str,
) -> Result<Option<String>, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(format!("{key} is not a string or null")),
    }
}


fn required_i64(
    object : &Map<String, Value>,
    key : &str,
    path : &Path,
) -> Result<i64, String> {
    object
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{}: missing integer {key}", path.display()))
}


fn required_i32(
    object : &Map<String, Value>,
    key : &str,
    path : &Path,
) -> Result<i32, String> {
    Ok(required_i64(object, key, path)? as i32)
}


fn required_i64_array(
    object : &Map<String, Value>,
    key : &str,
    path : &Path,
) -> Result<Vec<i64>, String> {
    let value = object
        .get(key)
        .ok_or_else(|| format!("{}: missing {key}", path.display()))?;

    i64_array(value, path)
}


fn i64_array(
    value : &Value,
    path : &Path,
) -> Result<Vec<i64>, String> {
    let array = value
        .as_array()
        .ok_or_else(|| format!("{}: expected an index list", path.display()))?;
    let mut values = Vec::new();

    for item in array {
        values.push(
            item.as_i64()
                .ok_or_else(|| format!("{}: index is not an integer", path.display()))?,
        );
    }

    Ok(values)
}


// ///////////////////////////// end of file //////////////////////////// //
