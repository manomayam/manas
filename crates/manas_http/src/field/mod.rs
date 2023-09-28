//! This module implements types for representing http fields.
//!
//!  > HTTP uses "fields" to provide data in the form of
//! extensible name/value pairs with a registered key namespace.
//! Fields are sent and received within the header and trailer
//! sections of messages.
//!

pub mod name;
pub mod pvalue;

pub mod rules;

// ABNF NOTES:

// Variable Repetition:  *Rule
// <a>*<b>element
// <a> and <b> are optional decimal values, indicating at least
//  <a> and at most <b> occurrences of the element.
// Default values are 0 and infinity

// Specific Repetition:  nRule
// <n>element is equivaent to <n>*<n>element

// Optional Sequence:  [RULE]
// Equivalent to *1(RULE)

// Comment:  ; Comment

//  Lists (#rule ABNF Extension)
// Extension for list of comma delimeted list of elements.
// The full form is "<n>#<m>element" indicating at least <n> and
// at most <m> elements, each separated by a single comma (",")
// and optional whitespace (OWS, defined in Section 5.6.3).
