//! Predefined expected-case inventory printed by the current scaffold command.
//! It does not execute conformance or produce observed results.

/// Expected-result label for a descriptive case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultKind {
    Accepted,
    Rejected,
    Indeterminate,
}

impl ResultKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Indeterminate => "indeterminate",
        }
    }
}

/// A public, value-free expected case. Its fields are descriptive labels, not
/// executable inputs or observed results.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Case {
    pub id: &'static str,
    pub description: &'static str,
    pub expected: ResultKind,
}

pub const CASES: &[Case] = &[
    Case {
        id: "release/valid",
        description: "exact bytes and active grant",
        expected: ResultKind::Accepted,
    },
    Case {
        id: "release/rollback",
        description: "older sequence is refused",
        expected: ResultKind::Rejected,
    },
    Case {
        id: "status/suspended",
        description: "blocking status removes currentness",
        expected: ResultKind::Rejected,
    },
    Case {
        id: "history/unknown",
        description: "incomplete history cannot be guessed",
        expected: ResultKind::Indeterminate,
    },
];

/// Returns the predefined expected-case inventory.
pub fn cases() -> &'static [Case] {
    CASES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_fields_are_nonempty_and_identifiers_are_unique() {
        for (index, case) in CASES.iter().enumerate() {
            assert!(!case.id.is_empty());
            assert!(!case.description.is_empty());
            assert!(CASES[index + 1..].iter().all(|other| other.id != case.id));
        }
    }
}
