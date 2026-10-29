// arguments.rs : parsed arguments and the sorted result


use crate::specifications::Specification;

use std::{
    cell::Cell,
    fmt::{
        self as std_fmt,
    },
};


/// Kind of one parsed argument.
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum ArgumentKind {
    /// A flag.
    Flag,
    /// An option.
    Option,
    /// A value.
    Value,
}


impl std_fmt::Display for ArgumentKind {
    fn fmt(
        &self,
        f : &mut std_fmt::Formatter<'_>,
    ) -> std_fmt::Result {
        f.write_str(match self {
            Self::Flag => "flag",
            Self::Option => "option",
            Self::Value => "value",
        })
    }
}


/// One parsed argument.
///
/// The used bit lives in a `Cell`, so a marking query can take `&self`.
/// See the crate documentation for that choice.
#[derive(Clone)]
#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Argument {
    kind :           ArgumentKind,
    resolved_name :  Option<String>,
    given_name :     Option<String>,
    value :          Option<String>,
    original_index : i32,
    given_hyphens :  u32,
    alias_index :    i32,
    used :           Cell<bool>,
}


impl Argument {
    pub(crate) fn new(
        kind : ArgumentKind,
        resolved_name : Option<String>,
        given_name : Option<String>,
        value : Option<String>,
        original_index : i32,
        given_hyphens : u32,
        alias_index : i32,
    ) -> Self {
        Self {
            kind,
            resolved_name,
            given_name,
            value,
            original_index,
            given_hyphens,
            alias_index,
            used : Cell::new(false),
        }
    }

    pub(crate) fn set_value(
        &mut self,
        value : Option<String>,
    ) {
        self.value = value;
    }

    /// Argument kind.
    pub fn kind(&self) -> ArgumentKind {
        self.kind
    }

    /// Name after specification lookup.
    ///
    /// `None` is a null C pointer.
    pub fn resolved_name(&self) -> Option<&str> {
        self.resolved_name.as_deref()
    }

    /// Name as written, without an attached value.
    ///
    /// `None` is a null C pointer.
    pub fn given_name(&self) -> Option<&str> {
        self.given_name.as_deref()
    }

    /// Attached value.
    ///
    /// `None` is a null C pointer (absent). `Some("")` is a present empty
    /// string.
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    /// Index in `argv`. The program name is 0.
    pub fn original_index(&self) -> i32 {
        self.original_index
    }

    /// Hyphens recorded for the original token.
    ///
    /// A double-hyphen token stores 2, including a token that begins
    /// with three hyphens.
    pub fn given_hyphens(&self) -> u32 {
        self.given_hyphens
    }

    /// Specification index, or -1 when none matched.
    ///
    /// A gap, a tacit marker, and a section occupy an index.
    pub fn alias_index(&self) -> i32 {
        self.alias_index
    }

    /// Whether a marking query has marked this argument.
    ///
    /// This does not itself mark the argument.
    pub fn is_used(&self) -> bool {
        self.used.get()
    }

    /// Marks this argument used.
    ///
    /// This is `clasp_useArgument`.
    pub fn mark_used(&self) {
        self.used.set(true);
    }
}


impl std_fmt::Display for Argument {
    fn fmt(
        &self,
        f : &mut std_fmt::Formatter<'_>,
    ) -> std_fmt::Result {
        let resolved = self.resolved_name.as_deref().unwrap_or("");

        match self.kind {
            ArgumentKind::Flag => {
                write!(f, "flag {resolved}")
            },
            ArgumentKind::Option => {
                match self.value.as_deref() {
                    Some(text) => write!(f, "option {resolved}={text}"),
                    None => write!(f, "option {resolved}"),
                }
            },
            ArgumentKind::Value => {
                write!(f, "value {}", self.value.as_deref().unwrap_or(""))
            },
        }
    }
}


/// Sorted parse result.
///
/// Partitions are slices of `arguments`: flags, then options, then
/// values. When `PRESERVE_ORIGINAL_ARGUMENT_ORDER` is set, `arguments`
/// stays in `argv` order and the three partitions are empty, which is
/// the C layout. No version-1 case sets that flag.
pub struct Arguments {
    pub(crate) program_name :   String,
    pub(crate) arguments :      Vec<Argument>,
    pub(crate) specifications : Option<Vec<Specification>>,
    pub(crate) flag_len :       usize,
    pub(crate) option_len :     usize,
    pub(crate) value_len :      usize,
}


impl Arguments {
    /// Program name: the last `'/'`-separated component of `argv[0]`.
    ///
    /// The extension is not removed. That is the non-Windows C result.
    /// An empty `argv` yields an empty name.
    pub fn program_name(&self) -> &str {
        &self.program_name
    }

    /// Arguments in result order.
    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    /// Flag partition.
    pub fn flags(&self) -> &[Argument] {
        let end = self.flag_len;

        &self.arguments[..end]
    }

    /// Option partition.
    pub fn options(&self) -> &[Argument] {
        let start = self.flag_len;
        let end = start + self.option_len;

        &self.arguments[start..end]
    }

    /// Value partition.
    pub fn values(&self) -> &[Argument] {
        let start = self.flag_len + self.option_len;
        let end = start + self.value_len;

        &self.arguments[start..end]
    }

    /// Number of flags in the flag partition.
    pub fn flag_count(&self) -> usize {
        self.flag_len
    }

    /// Number of options in the option partition.
    pub fn option_count(&self) -> usize {
        self.option_len
    }

    /// Number of values in the value partition.
    pub fn value_count(&self) -> usize {
        self.value_len
    }

    /// Specifications supplied to `parse`, when the pointer was not null.
    pub fn specifications(&self) -> Option<&[Specification]> {
        self.specifications.as_deref()
    }

    /// Index of `argument` in `arguments`.
    pub fn index_of(
        &self,
        argument : &Argument,
    ) -> Option<usize> {
        self.arguments.iter().position(|item| std::ptr::eq(item, argument))
    }

    /// Arguments whose used bit is clear, in result order.
    ///
    /// This does not mark any argument.
    pub fn unused_arguments(&self) -> impl Iterator<Item = &Argument> {
        self.arguments.iter().filter(|argument| !argument.is_used())
    }
}


// ///////////////////////////// end of file //////////////////////////// //
