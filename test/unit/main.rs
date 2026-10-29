// test/unit/main.rs : environment-free parser checks

#![allow(non_snake_case)]


use clasp_rs::{
    parse,
    ArgumentKind,
    ParseFlags,
    Specification,
};


#[test]
fn TEST_parse_PROGRAM_NAME_ONLY() {
    let parsed = parse::<&str>(ParseFlags::empty(), &["tool"], None);

    assert_eq!("tool", parsed.program_name());
    assert!(parsed.arguments().is_empty());
    assert_eq!(0, parsed.flag_count());
    assert!(parsed.specifications().is_none());
}


#[test]
fn TEST_parse_SORTS_FLAG_THEN_OPTION_THEN_VALUE() {
    let specifications = [
        Specification::flag(Some("-v"), Some("--verbose"), None),
        Specification::option(Some("-n"), Some("--name"), None, None),
    ];
    let parsed = parse(
        ParseFlags::empty(),
        &["tool", "file", "-n", "out", "-v"],
        Some(&specifications),
    );

    assert_eq!(
        vec![ArgumentKind::Flag, ArgumentKind::Option, ArgumentKind::Value],
        parsed
            .arguments()
            .iter()
            .map(|argument| argument.kind())
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        Some("--verbose"),
        parsed.flags().first().and_then(|argument| argument.resolved_name())
    );
    assert_eq!(
        Some("out"),
        parsed.options().first().and_then(|argument| argument.value())
    );
    assert_eq!(
        Some("file"),
        parsed.values().first().and_then(|argument| argument.value())
    );
}


#[test]
fn TEST_parse_ALIAS_PREFIX_MATCHES_SHORTER_ARGUMENT() {
    let specifications = [Specification::flag(Some("-verbose"), Some("--verbose"), None)];
    let parsed = parse(ParseFlags::empty(), &["tool", "-v"], Some(&specifications));
    let argument = parsed.arguments().first().expect("one argument");

    assert_eq!(ArgumentKind::Flag, argument.kind());
    assert_eq!(Some("--verbose"), argument.resolved_name());
    assert_eq!(Some("-v"), argument.given_name());
    assert_eq!(0, argument.alias_index());
}


#[test]
fn TEST_parse_COMBINED_FLAGS_SPLIT_WHEN_WHOLE_TOKEN_IS_UNKNOWN() {
    let specifications = [
        Specification::flag(Some("-a"), Some("--alpha"), None),
        Specification::flag(Some("-c"), Some("--gamma"), None),
    ];
    let parsed = parse(ParseFlags::empty(), &["tool", "-abc"], Some(&specifications));
    let names = parsed
        .arguments()
        .iter()
        .map(|argument| argument.resolved_name().unwrap_or("").to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        vec!["--alpha".to_string(), "--gamma".to_string(), "-b".to_string()],
        names
    );
    assert!(parsed
        .arguments()
        .iter()
        .all(|argument| argument.given_name() == Some("-abc")));
}


#[test]
fn TEST_parse_EMPTY_ARGV_DOES_NOT_PANIC() {
    let parsed = parse::<&str>(ParseFlags::empty(), &[], None);

    assert_eq!("", parsed.program_name());
    assert!(parsed.arguments().is_empty());
}


#[test]
fn TEST_flag_is_specified_MARKS_USED_AND_REPORT_DOES_NOT() {
    let specifications = [Specification::flag(Some("-v"), Some("--verbose"), None)];
    let parsed = parse(ParseFlags::empty(), &["tool", "-v", "file"], Some(&specifications));
    let before = parsed.report_unused_flags(0);

    assert_eq!(1, before.remaining);
    assert!(!parsed.flags().first().expect("flag").is_used());
    assert!(parsed.flag_is_specified("--verbose"));
    assert!(parsed.flags().first().expect("flag").is_used());
    assert_eq!(0, parsed.report_unused_flags(0).remaining);
    assert!(!parsed.values().first().expect("value").is_used());
    assert_eq!(1, parsed.report_unused_values(0).remaining);
}


// ///////////////////////////// end of file //////////////////////////// //
