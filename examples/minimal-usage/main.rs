use clasp_rs::{
    parse,
    ParseFlags,
    Specification,
};

use std::env as std_env;


fn main() {
    let specifications = [
        Specification::flag(Some("-v"), Some("--verbose"), None),
        Specification::option(Some("-n"), Some("--name"), Some("a name"), None),
    ];
    let argv = std_env::args().collect::<Vec<_>>();
    let parsed = parse(ParseFlags::empty(), &argv, Some(&specifications));

    println!("{}", parsed.program_name());
    println!("flags:");

    for argument in parsed.flags() {
        println!("  {argument}");
    }

    println!("options:");

    for argument in parsed.options() {
        println!("  {argument}");
    }

    println!("values:");

    for argument in parsed.values() {
        println!("  {argument}");
    }

    println!("unused:");

    for argument in parsed.unused_arguments() {
        println!("  {argument}");
    }
}
