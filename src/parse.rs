// parse.rs : CLASP parse, following src/clasp.api.c


use crate::{
    arguments::{
        Argument,
        ArgumentKind,
        Arguments,
    },
    specifications::{
        is_null_or_empty,
        lookup,
        separator_index,
        Specification,
        SpecificationType,
    },
};

use std::cmp as std_fmt;


/// Flags that control parsing. Values match **include/clasp/clasp.h**.
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(Default)]
#[derive(Eq, PartialEq)]
pub struct ParseFlags {
    bits : u32,
}


impl ParseFlags {
    /// `CLASP_F_DONT_EXPAND_WILDCARDS_ON_WINDOWS`.
    ///
    /// No effect. Version 1 does not expand wildcards.
    pub const DONT_EXPAND_WILDCARDS_ON_WINDOWS : u32 = 0x0000_0100;
    /// `CLASP_F_DONT_RECOGNISE_DOUBLEHYPHEN_TO_START_VALUES`.
    pub const DONT_RECOGNISE_DOUBLEHYPHEN_TO_START_VALUES : u32 = 0x0000_0001;
    /// `CLASP_F_DO_EXPAND_WILDCARDS_IN_APOSQUOTES_ON_WINDOWS`.
    ///
    /// No effect. Version 1 does not expand wildcards.
    pub const DO_EXPAND_WILDCARDS_IN_APOSQUOTES_ON_WINDOWS : u32 = 0x0000_0200;
    /// `CLASP_F_PRESERVE_ORIGINAL_ARGUMENT_ORDER`.
    ///
    /// The result stays in `argv` order and the flag, option, and value
    /// partitions are empty. No version-1 case sets this flag.
    pub const PRESERVE_ORIGINAL_ARGUMENT_ORDER : u32 = 0x0000_0400;
    /// `CLASP_F_TREAT_SINGLEHYPHEN_AS_VALUE`.
    pub const TREAT_SINGLEHYPHEN_AS_VALUE : u32 = 0x0000_0002;

    /// No flags set.
    pub const fn empty() -> Self {
        Self {
            bits : 0
        }
    }

    /// Flags from the C bit set.
    pub const fn from_bits(bits : u32) -> Self {
        Self {
            bits,
        }
    }

    /// The stored bit set.
    pub const fn bits(self) -> u32 {
        self.bits
    }

    /// Whether `bit` is present.
    pub const fn contains(
        self,
        bit : u32,
    ) -> bool {
        self.bits & bit != 0
    }

    /// `self` with `bit` set.
    pub const fn union(
        self,
        bit : u32,
    ) -> Self {
        Self {
            bits : self.bits | bit
        }
    }
}


struct Built {
    kind :           ArgumentKind,
    resolved_name :  Option<String>,
    given_name :     Option<String>,
    value :          Option<String>,
    original_index : i32,
    given_hyphens :  u32,
    alias_index :    i32,
}


impl Built {
    fn argument(self) -> Argument {
        Argument::new(
            self.kind,
            self.resolved_name,
            self.given_name,
            self.value,
            self.original_index,
            self.given_hyphens,
            self.alias_index,
        )
    }
}


struct Classified {
    arguments :    Vec<Built>,
    await_value :  bool,
    start_values : bool,
}


fn done(argument : Built) -> Classified {
    Classified {
        arguments :    vec![argument],
        await_value :  false,
        start_values : false,
    }
}


fn await_value(argument : Built) -> Classified {
    Classified {
        arguments :    vec![argument],
        await_value :  true,
        start_values : false,
    }
}


fn same_name(text : &str) -> (Option<String>, Option<String>) {
    let owned = text.to_string();

    (Some(owned.clone()), Some(owned))
}


fn argument_kind(spec : &Specification) -> ArgumentKind {
    if spec.specification_type() == SpecificationType::Option {
        ArgumentKind::Option
    } else {
        ArgumentKind::Flag
    }
}


fn mapped_non_empty(spec : &Specification) -> Option<&str> {
    match spec.mapped_argument() {
        Some(text) if !text.is_empty() => Some(text),
        _ => None,
    }
}


fn alias_non_empty(spec : &Specification) -> bool {
    !is_null_or_empty(spec.alias())
}


/// Separator index in `arg` at or after byte `skip`.
fn separator_after(
    arg : &str,
    skip : usize,
) -> Option<usize> {
    let suffix = arg.get(skip..)?;

    suffix.find(['=', ':']).map(|offset| skip + offset)
}


fn resolved_from_mapped(mapped : &str) -> String {
    match separator_index(mapped) {
        Some(eq) => mapped[..eq].to_string(),
        None => mapped.to_string(),
    }
}


fn value_after_separator(text : &str) -> String {
    match separator_index(text) {
        Some(eq) => text[eq + 1..].to_string(),
        None => String::new(),
    }
}


fn next_is_free(
    specifications : Option<&[Specification]>,
    next : Option<&str>,
) -> bool {
    match next {
        Some(text) if !text.is_empty() => lookup(specifications, text).is_none(),
        _ => false,
    }
}


fn classify_double_with_separator(
    arg : &str,
    eq : usize,
    index : i32,
    specifications : Option<&[Specification]>,
) -> Classified {
    let name = &arg[..eq];
    let value = arg[eq + 1..].to_string();
    let found = lookup(specifications, name);
    let alias_index = found.map(|(alias_index, _)| alias_index).unwrap_or(-1);
    let (resolved_name, given_name) = if let Some(mapped) = found.and_then(|(_, spec)| mapped_non_empty(spec)) {
        (Some(resolved_from_mapped(mapped)), Some(name.to_string()))
    } else {
        same_name(name)
    };

    done(Built {
        kind : ArgumentKind::Option,
        resolved_name,
        given_name,
        value : Some(value),
        original_index : index,
        given_hyphens : 2,
        alias_index,
    })
}


fn classify_double_without_separator(
    arg : &str,
    index : i32,
    specifications : Option<&[Specification]>,
    next : Option<&str>,
) -> Classified {
    let found = lookup(specifications, arg);

    if let Some((alias_index, spec)) = found {
        if let Some(mapped) = mapped_non_empty(spec) {
            if separator_index(mapped).is_some() {
                return done(Built {
                    kind : ArgumentKind::Option,
                    resolved_name : Some(resolved_from_mapped(mapped)),
                    given_name : Some(arg.to_string()),
                    value : Some(value_after_separator(mapped)),
                    original_index : index,
                    given_hyphens : 2,
                    alias_index,
                });
            }

            let built = Built {
                kind : argument_kind(spec),
                resolved_name : Some(mapped.to_string()),
                given_name : Some(arg.to_string()),
                value : None,
                original_index : index,
                given_hyphens : 2,
                alias_index,
            };

            if spec.specification_type() == SpecificationType::Option
                && alias_non_empty(spec)
                && next_is_free(specifications, next)
            {
                return await_value(built);
            }

            return done(built);
        }
    }

    let alias_index = found.map(|(alias_index, _)| alias_index).unwrap_or(-1);
    let built = Built {
        kind : ArgumentKind::Flag,
        resolved_name : Some(arg.to_string()),
        given_name : Some(arg.to_string()),
        value : None,
        original_index : index,
        given_hyphens : 2,
        alias_index,
    };

    if let Some((_, spec)) = found {
        if spec.specification_type() == SpecificationType::Option {
            return await_value(Built {
                kind : ArgumentKind::Option,
                ..built
            });
        }
    }

    done(built)
}


fn classify_double(
    arg : &str,
    index : i32,
    flags : ParseFlags,
    specifications : Option<&[Specification]>,
    next : Option<&str>,
) -> Classified {
    let recognise = !flags.contains(ParseFlags::DONT_RECOGNISE_DOUBLEHYPHEN_TO_START_VALUES);

    if arg.len() == 2 && recognise {
        return Classified {
            arguments :    Vec::new(),
            await_value :  false,
            start_values : true,
        };
    }

    if let Some(eq) = separator_after(arg, 2) {
        classify_double_with_separator(arg, eq, index, specifications)
    } else {
        classify_double_without_separator(arg, index, specifications, next)
    }
}


fn classify_single_with_separator(
    arg : &str,
    eq : usize,
    index : i32,
    specifications : Option<&[Specification]>,
) -> Classified {
    let name = &arg[..eq];
    let value = arg[eq + 1..].to_string();
    let found = lookup(specifications, name);
    let alias_index = found.map(|(alias_index, _)| alias_index).unwrap_or(-1);
    let (resolved_name, given_name) = if let Some(mapped) = found.and_then(|(_, spec)| mapped_non_empty(spec)) {
        (Some(resolved_from_mapped(mapped)), Some(name.to_string()))
    } else {
        same_name(name)
    };

    done(Built {
        kind : ArgumentKind::Option,
        resolved_name,
        given_name,
        value : Some(value),
        original_index : index,
        given_hyphens : 1,
        alias_index,
    })
}


fn classify_known_short(
    arg : &str,
    index : i32,
    alias_index : i32,
    spec : &Specification,
) -> Classified {
    let mapped = spec.mapped_argument();
    let leading_hyphen = mapped.is_some_and(|text| text.starts_with('-'));

    if leading_hyphen {
        if let Some(text) = mapped {
            if separator_index(text).is_some() {
                return done(Built {
                    kind : ArgumentKind::Option,
                    resolved_name : Some(resolved_from_mapped(text)),
                    given_name : Some(arg.to_string()),
                    value : Some(value_after_separator(text)),
                    original_index : index,
                    given_hyphens : 1,
                    alias_index,
                });
            }
        }
    }

    let (resolved_name, given_name) = if leading_hyphen {
        (mapped.map(str::to_string), Some(arg.to_string()))
    } else {
        same_name(arg)
    };
    let await_next = spec.specification_type() == SpecificationType::Option;
    let kind = if await_next {
        ArgumentKind::Option
    } else {
        argument_kind(spec)
    };
    let built = Built {
        kind,
        resolved_name,
        given_name,
        value : None,
        original_index : index,
        given_hyphens : 1,
        alias_index,
    };

    if await_next {
        await_value(built)
    } else {
        done(built)
    }
}


fn flag_letter(byte : u8) -> String {
    let mut name = String::from("-");

    name.push(char::from(byte));
    name
}


fn classify_combined(
    arg : &str,
    index : i32,
    specifications : Option<&[Specification]>,
) -> Classified {
    let mut arguments = Vec::new();
    let bytes = arg.as_bytes();
    let mut offset = 1usize;

    while offset < bytes.len() {
        let letter = flag_letter(bytes[offset]);
        offset += 1;

        if let Some((alias_index, spec)) = lookup(specifications, &letter) {
            let mapped = spec.mapped_argument();
            let valued = mapped.is_some_and(|text| text.starts_with('-') && separator_index(text).is_some());

            if valued {
                if let Some(text) = mapped {
                    arguments.push(Built {
                        kind : ArgumentKind::Option,
                        resolved_name : Some(resolved_from_mapped(text)),
                        given_name : Some(arg.to_string()),
                        value : Some(value_after_separator(text)),
                        original_index : index,
                        given_hyphens : 1,
                        alias_index,
                    });
                }
            } else {
                let resolved_name = match mapped {
                    Some(text) => Some(text.to_string()),
                    None => Some(letter),
                };

                arguments.push(Built {
                    kind : argument_kind(spec),
                    resolved_name,
                    given_name : Some(arg.to_string()),
                    value : None,
                    original_index : index,
                    given_hyphens : 1,
                    alias_index,
                });
            }
        } else {
            arguments.push(Built {
                kind :           ArgumentKind::Flag,
                resolved_name :  Some(letter),
                given_name :     Some(arg.to_string()),
                value :          None,
                original_index : index,
                given_hyphens :  1,
                alias_index :    -1,
            });
        }
    }

    Classified {
        arguments,
        await_value : false,
        start_values : false,
    }
}


fn classify_single(
    arg : &str,
    index : i32,
    flags : ParseFlags,
    specifications : Option<&[Specification]>,
) -> Classified {
    if arg.len() == 1 {
        let (resolved_name, given_name) = same_name("-");
        let treat = flags.contains(ParseFlags::TREAT_SINGLEHYPHEN_AS_VALUE);

        return done(Built {
            kind : if treat { ArgumentKind::Value } else { ArgumentKind::Flag },
            resolved_name : resolved_name.clone(),
            given_name,
            value : if treat { resolved_name } else { None },
            original_index : index,
            given_hyphens : 1,
            alias_index : -1,
        });
    }

    if let Some(eq) = separator_after(arg, 1) {
        return classify_single_with_separator(arg, eq, index, specifications);
    }

    if specifications.is_none() {
        let (resolved_name, given_name) = same_name(arg);

        return done(Built {
            kind : ArgumentKind::Flag,
            resolved_name,
            given_name,
            value : None,
            original_index : index,
            given_hyphens : 1,
            alias_index : -1,
        });
    }

    if let Some((alias_index, spec)) = lookup(specifications, arg) {
        return classify_known_short(arg, index, alias_index, spec);
    }

    classify_combined(arg, index, specifications)
}


fn classify_value(
    arg : &str,
    index : i32,
) -> Classified {
    done(Built {
        kind :           ArgumentKind::Value,
        resolved_name :  None,
        given_name :     None,
        value :          Some(arg.to_string()),
        original_index : index,
        given_hyphens :  0,
        alias_index :    -1,
    })
}


fn classify(
    arg : &str,
    index : i32,
    flags : ParseFlags,
    specifications : Option<&[Specification]>,
    next : Option<&str>,
    treat_all_as_values : bool,
) -> Classified {
    if treat_all_as_values || !arg.starts_with('-') {
        return classify_value(arg, index);
    }

    if arg.as_bytes().get(1) == Some(&b'-') {
        classify_double(arg, index, flags, specifications, next)
    } else {
        classify_single(arg, index, flags, specifications)
    }
}


fn program_name_of(argv0 : &str) -> String {
    match argv0.rfind('/') {
        Some(slash) => argv0[slash + 1..].to_string(),
        None => argv0.to_string(),
    }
}


/// When the C comparator returns 0, the order is unspecified.
///
/// A length tie-break keeps the ordering total, so sorting does not
/// panic. Version-1 cases do not rely on that tie.
fn name_order(
    lhs : Option<&str>,
    rhs : Option<&str>,
) -> std_fmt::Ordering {
    let left = lhs.unwrap_or("");
    let right = rhs.unwrap_or("");
    let width = left.len().min(right.len());

    match left.as_bytes()[..width].cmp(&right.as_bytes()[..width]) {
        std_fmt::Ordering::Equal => left.len().cmp(&right.len()),
        other => other,
    }
}


fn kind_rank(kind : ArgumentKind) -> u8 {
    match kind {
        ArgumentKind::Flag => 0,
        ArgumentKind::Option => 1,
        ArgumentKind::Value => 2,
    }
}


fn compare_arguments(
    lhs : &Argument,
    rhs : &Argument,
) -> std_fmt::Ordering {
    if lhs.kind() == rhs.kind() {
        let by_index = lhs.original_index().cmp(&rhs.original_index());

        if by_index != std_fmt::Ordering::Equal {
            return by_index;
        }

        return name_order(lhs.resolved_name(), rhs.resolved_name());
    }

    kind_rank(lhs.kind()).cmp(&kind_rank(rhs.kind()))
}


fn partition_lengths(arguments : &[Argument]) -> (usize, usize, usize) {
    let flag_len = arguments
        .iter()
        .take_while(|argument| argument.kind() == ArgumentKind::Flag)
        .count();
    let option_len = arguments[flag_len..]
        .iter()
        .take_while(|argument| argument.kind() == ArgumentKind::Option)
        .count();
    let value_len = arguments.len() - flag_len - option_len;

    (flag_len, option_len, value_len)
}


/// Parses `argv` against `specifications`.
///
/// `argv[0]` is the program name. `None` is a null specification
/// pointer, and `Some(&[])` is an empty array. Those two are different:
/// a null pointer does not split a short token, and an empty array does.
///
/// An empty `argv` is undefined in C. This function returns an empty
/// program name and no arguments.
///
/// Duplicate aliases are a C `EINVAL`. This function still parses, and
/// the first matching specification wins.
pub fn parse<T>(
    flags : ParseFlags,
    argv : &[T],
    specifications : Option<&[Specification]>,
) -> Arguments
where
    T : AsRef<str>,
{
    let owned_specs = specifications.map(<[Specification]>::to_vec);

    if argv.is_empty() {
        return Arguments {
            program_name :   String::new(),
            arguments :      Vec::new(),
            specifications : owned_specs,
            flag_len :       0,
            option_len :     0,
            value_len :      0,
        };
    }

    let mut arguments : Vec<Argument> = Vec::new();
    let mut await_value = false;
    let mut treat_all_as_values = false;
    let mut index = 1usize;

    while index < argv.len() {
        let arg = argv[index].as_ref();

        if await_value {
            if let Some(current) = arguments.last_mut() {
                current.set_value(Some(arg.to_string()));
            }
            await_value = false;
            index += 1;
            continue;
        }

        let next = argv.get(index + 1).map(AsRef::as_ref);
        let classified = classify(arg, index as i32, flags, specifications, next, treat_all_as_values);

        if classified.start_values {
            treat_all_as_values = true;
        }
        await_value = classified.await_value;
        arguments.extend(classified.arguments.into_iter().map(Built::argument));
        index += 1;
    }

    if await_value {
        if let Some(current) = arguments.last_mut() {
            current.set_value(None);
        }
    }

    let preserve = flags.contains(ParseFlags::PRESERVE_ORIGINAL_ARGUMENT_ORDER);

    if !preserve {
        arguments.sort_by(compare_arguments);
    }

    let (flag_len, option_len, value_len) = if preserve {
        (0, 0, 0)
    } else {
        partition_lengths(&arguments)
    };

    Arguments {
        program_name : program_name_of(argv[0].as_ref()),
        arguments,
        specifications : owned_specs,
        flag_len,
        option_len,
        value_len,
    }
}


// ///////////////////////////// end of file //////////////////////////// //
