// specifications.rs : flag, option, gap, tacit, and section specifications


/// Kind of one specification entry.
///
/// `Section` carries the positive section number that the C array stores
/// in the specification `type` field.
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum SpecificationType {
    /// A flag specification.
    Flag,
    /// A gap. It is not an argument, and it still occupies an index.
    Gap,
    /// An option specification.
    Option,
    /// A numbered section. It is not an argument, and it still occupies
    /// an index.
    Section(i32),
    /// A tacit marker. It is not an argument, and it still occupies an
    /// index.
    Tacit,
}


/// One CLASP specification.
///
/// A null C string is `None`. A present empty C string is `Some("")`.
#[derive(Clone)]
#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Specification {
    specification_type : SpecificationType,
    alias :              Option<String>,
    mapped_argument :    Option<String>,
    help :               Option<String>,
    value_set :          Option<String>,
    bit_flags :          i32,
    section :            Option<i32>,
}


impl Specification {
    /// Flag specification. `bit_flags` is 0.
    pub fn flag(
        alias : Option<&str>,
        mapped_argument : Option<&str>,
        help : Option<&str>,
    ) -> Self {
        Self::bit_flag(alias, mapped_argument, 0, help)
    }

    /// Flag specification with a bit value.
    pub fn bit_flag(
        alias : Option<&str>,
        mapped_argument : Option<&str>,
        bit_flags : i32,
        help : Option<&str>,
    ) -> Self {
        Self {
            specification_type : SpecificationType::Flag,
            alias : owned(alias),
            mapped_argument : owned(mapped_argument),
            help : owned(help),
            value_set : None,
            bit_flags,
            section : None,
        }
    }

    /// Option specification.
    pub fn option(
        alias : Option<&str>,
        mapped_argument : Option<&str>,
        help : Option<&str>,
        value_set : Option<&str>,
    ) -> Self {
        Self {
            specification_type : SpecificationType::Option,
            alias :              owned(alias),
            mapped_argument :    owned(mapped_argument),
            help :               owned(help),
            value_set :          owned(value_set),
            bit_flags :          0,
            section :            None,
        }
    }

    /// Gap entry. `label` is stored as the help string, which is where
    /// the C `CLASP_GAP_SECTION` macro places it.
    pub fn gap(label : Option<&str>) -> Self {
        Self {
            specification_type : SpecificationType::Gap,
            alias :              None,
            mapped_argument :    None,
            help :               owned(label),
            value_set :          None,
            bit_flags :          0,
            section :            None,
        }
    }

    /// Tacit marker.
    pub fn tacit() -> Self {
        Self {
            specification_type : SpecificationType::Tacit,
            alias :              None,
            mapped_argument :    None,
            help :               None,
            value_set :          None,
            bit_flags :          0,
            section :            None,
        }
    }

    /// Numbered section. The number is stored as the section and as the
    /// specification type, matching the C array.
    pub fn section(number : i32) -> Self {
        Self {
            specification_type : SpecificationType::Section(number),
            alias :              None,
            mapped_argument :    None,
            help :               None,
            value_set :          None,
            bit_flags :          0,
            section :            Some(number),
        }
    }

    /// Specification type.
    pub fn specification_type(&self) -> SpecificationType {
        self.specification_type
    }

    /// Alias (`name` in the C structure). `None` is a null pointer.
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Mapped argument. `None` is a null pointer.
    pub fn mapped_argument(&self) -> Option<&str> {
        self.mapped_argument.as_deref()
    }

    /// Help string. `None` is a null pointer. `Some("")` is a present
    /// empty string.
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// Value set. Not used by version-1 parsing.
    pub fn value_set(&self) -> Option<&str> {
        self.value_set.as_deref()
    }

    /// Bit value of a flag specification.
    pub fn bit_flags(&self) -> i32 {
        self.bit_flags
    }

    /// Section number when the type is a section; otherwise `None`.
    pub fn section_number(&self) -> Option<i32> {
        self.section
    }
}


pub(crate) fn owned(text : Option<&str>) -> Option<String> {
    text.map(str::to_string)
}


pub(crate) fn separator_index(text : &str) -> Option<usize> {
    text.find(['=', ':'])
}


pub(crate) fn is_null_or_empty(text : Option<&str>) -> bool {
    match text {
        None => true,
        Some(value) => value.is_empty(),
    }
}


/// `strncmp` of `spec_name` against `arg` for `arg.len()` bytes.
///
/// A longer specification name matches a shorter argument that is its
/// prefix. The comparison does not require the specification name to
/// end. This is `clasp_lookup_spec_for_name_` in **src/clasp.api.c**.
pub(crate) fn name_prefix_eq(
    spec_name : &str,
    arg : &str,
) -> bool {
    let spec_bytes = spec_name.as_bytes();
    let arg_bytes = arg.as_bytes();

    spec_bytes.get(..arg_bytes.len()) == Some(arg_bytes)
}


/// First matching specification, in the C search order.
///
/// Name prefix, then a mapped argument with no default, then a mapped
/// argument whose default is separated by `'='` or `':'`.
pub(crate) fn lookup<'a>(
    specifications : Option<&'a [Specification]>,
    arg : &str,
) -> Option<(i32, &'a Specification)> {
    let specifications = specifications?;

    for (index, spec) in specifications.iter().enumerate() {
        if let Some(name) = spec.alias.as_deref() {
            if name_prefix_eq(name, arg) {
                return Some((index as i32, spec));
            }
        }
    }

    for (index, spec) in specifications.iter().enumerate() {
        if let Some(mapped) = spec.mapped_argument.as_deref() {
            if separator_index(mapped).is_none() && mapped == arg {
                return Some((index as i32, spec));
            }
        }
    }

    for (index, spec) in specifications.iter().enumerate() {
        if let Some(mapped) = spec.mapped_argument.as_deref() {
            if let Some(eq) = separator_index(mapped) {
                if &mapped[..eq] == arg {
                    return Some((index as i32, spec));
                }
            }
        }
    }

    None
}


// ///////////////////////////// end of file //////////////////////////// //
