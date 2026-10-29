// queries.rs : marking queries and nonmutating reports


use crate::{
    arguments::{
        Argument,
        Arguments,
    },
    specifications::{
        lookup,
        Specification,
        SpecificationType,
    },
};


/// One hit from a report query.
///
/// `remaining` is the C return: the number of matches after `skip`,
/// including `first` when it is `Some`.
pub struct Report<'a> {
    /// Matches remaining after the skip, including `first`.
    pub remaining : usize,
    /// The first match after the skip.
    pub first :     Option<&'a Argument>,
}


fn resolve_specs<'a>(
    arguments : &'a Arguments,
    specifications : Option<&'a [Specification]>,
) -> Option<&'a [Specification]> {
    match specifications {
        Some(specs) => Some(specs),
        None => arguments.specifications(),
    }
}


fn report_unused<'a>(
    items : &'a [Argument],
    skip : u32,
) -> Report<'a> {
    let mut seen = 0u32;
    let mut remaining = 0usize;
    let mut first = None;

    for argument in items {
        if argument.is_used() {
            continue;
        }
        if seen < skip {
            seen += 1;
            continue;
        }
        remaining += 1;
        if first.is_none() {
            first = Some(argument);
        }
    }

    Report {
        remaining,
        first,
    }
}


impl Arguments {
    /// Whether a flag with this resolved name is present.
    ///
    /// Marks the first match. This is `clasp_flagIsSpecified`.
    pub fn flag_is_specified(
        &self,
        mapped_argument_name : &str,
    ) -> bool {
        let mut bit_flags = 0;

        self.check_flag(mapped_argument_name, &mut bit_flags, 1)
    }

    /// Sets `flag` into `bit_flags` when a flag's resolved name matches.
    ///
    /// Marks that flag. This is `clasp_checkFlag`. Returns whether a
    /// flag matched.
    pub fn check_flag(
        &self,
        mapped_argument_name : &str,
        bit_flags : &mut i32,
        flag : i32,
    ) -> bool {
        for argument in self.flags() {
            if argument.resolved_name() == Some(mapped_argument_name) {
                *bit_flags |= flag;
                argument.mark_used();
                return true;
            }
        }

        false
    }

    /// ORs the bit values of unused flags and marks each matched flag.
    ///
    /// A flag whose bit value is 0 is marked and does not change the
    /// result. `None` uses the specifications passed to `parse`. When
    /// those were null, the result is `*bit_flags` and nothing is
    /// marked. This is `clasp_checkAllFlags`.
    pub fn check_all_flags(
        &self,
        specifications : Option<&[Specification]>,
        bit_flags : &mut i32,
    ) -> i32 {
        let Some(specifications) = resolve_specs(self, specifications) else {
            return *bit_flags;
        };

        for argument in self.flags() {
            if argument.is_used() {
                continue;
            }
            let Some(resolved) = argument.resolved_name() else {
                continue;
            };

            for spec in specifications {
                if spec.specification_type() != SpecificationType::Flag {
                    continue;
                }
                if spec.mapped_argument() != Some(resolved) {
                    continue;
                }

                argument.mark_used();
                if spec.bit_flags() != 0 {
                    *bit_flags |= spec.bit_flags();
                    break;
                }
            }
        }

        *bit_flags
    }

    /// ORs bit values of unused flags whose specification bits overlap
    /// `mask`, and marks each such flag.
    ///
    /// `None` uses the specifications passed to `parse`. This is
    /// `clasp_checkAllMatchingFlags`.
    pub fn check_all_matching_flags(
        &self,
        specifications : Option<&[Specification]>,
        mask : i32,
        bit_flags : &mut i32,
    ) -> i32 {
        let Some(specifications) = resolve_specs(self, specifications) else {
            return *bit_flags;
        };

        for argument in self.flags() {
            if argument.is_used() {
                continue;
            }
            let Some(resolved) = argument.resolved_name() else {
                continue;
            };

            for spec in specifications {
                if (spec.bit_flags() & mask) == 0 {
                    continue;
                }
                if spec.mapped_argument() != Some(resolved) {
                    continue;
                }

                *bit_flags |= spec.bit_flags();
                argument.mark_used();
                break;
            }
        }

        *bit_flags
    }

    /// The flag or option with this resolved name, after `skip` matches.
    ///
    /// Marks only the returned argument. This is
    /// `clasp_findFlagOrOption`.
    pub fn find_flag_or_option(
        &self,
        mapped_argument_name : &str,
        skip : u32,
    ) -> Option<&Argument> {
        let mut seen = 0u32;
        let end = self.flag_count() + self.option_count();

        for argument in &self.arguments()[..end] {
            if argument.resolved_name() != Some(mapped_argument_name) {
                continue;
            }
            if seen < skip {
                seen += 1;
                continue;
            }
            argument.mark_used();
            return Some(argument);
        }

        None
    }

    /// The value at `index` in the value partition.
    ///
    /// `Some` means the index exists, which is what the C body returns,
    /// and the value is marked. `None` means the index does not exist
    /// and nothing is marked. This is `clasp_checkValue`.
    pub fn check_value(
        &self,
        index : usize,
    ) -> Option<&Argument> {
        if index >= self.value_count() {
            return None;
        }

        let argument = self.values().get(index)?;

        argument.mark_used();
        Some(argument)
    }

    /// Unused flags after `skip`. Does not mark.
    pub fn report_unused_flags(
        &self,
        skip : u32,
    ) -> Report<'_> {
        report_unused(self.flags(), skip)
    }

    /// Unused options after `skip`. Does not mark.
    pub fn report_unused_options(
        &self,
        skip : u32,
    ) -> Report<'_> {
        report_unused(self.options(), skip)
    }

    /// Unused flags and options after `skip`. Does not mark.
    pub fn report_unused_flags_and_options(
        &self,
        skip : u32,
    ) -> Report<'_> {
        let end = self.flag_count() + self.option_count();

        report_unused(&self.arguments()[..end], skip)
    }

    /// Unused values after `skip`. Does not mark.
    pub fn report_unused_values(
        &self,
        skip : u32,
    ) -> Report<'_> {
        report_unused(self.values(), skip)
    }

    /// Unused arguments after `skip`. Does not mark.
    pub fn report_unused_arguments(
        &self,
        skip : u32,
    ) -> Report<'_> {
        report_unused(self.arguments(), skip)
    }

    /// Unrecognised flags and options after `skip`.
    ///
    /// `None` means the specification pointer is null, so the C
    /// function is not called. An empty slice is a real array: every
    /// flag and option is unrecognised. Does not mark.
    pub fn report_unrecognised_flags_and_options(
        &self,
        specifications : Option<&[Specification]>,
        skip : u32,
    ) -> Option<Report<'_>> {
        let specifications = resolve_specs(self, specifications)?;
        let end = self.flag_count() + self.option_count();
        let mut seen = 0u32;
        let mut remaining = 0usize;
        let mut first = None;

        for argument in &self.arguments()[..end] {
            let recognised = match argument.resolved_name() {
                Some(name) => lookup(Some(specifications), name).is_some(),
                None => false,
            };

            if recognised {
                continue;
            }
            if seen < skip {
                seen += 1;
                continue;
            }
            remaining += 1;
            if first.is_none() {
                first = Some(argument);
            }
        }

        Some(Report {
            remaining,
            first,
        })
    }
}


// ///////////////////////////// end of file //////////////////////////// //
