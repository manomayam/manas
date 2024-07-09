//! This example demonstrates on creating a refinement type satisfying a single predicate.
//!

use gdp_rs::{
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
    Proven,
};

/// A struct for representing a password, that allows arbitrary string value.
struct Password {
    value: String,
}

/// A predicate type overt [`Password`], that checks if it is strong.
#[derive(Debug)]
struct IsStrong {}

// Implement predicate contract over subject values of [``Password`] type.
impl Predicate<Password> for IsStrong {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsStrong".into()
    }
}

impl SyncEvaluablePredicate<Password> for IsStrong {
    type EvalError = IsNotStrong;

    fn evaluate_for(sub: &Password) -> Result<(), Self::EvalError> {
        let p = &sub.value;
        // We have simple check of length for password strength.
        if p.len() >= 8 {
            Ok(())
        } else {
            Err(IsNotStrong)
        }
    }
}

/// Mark predicate as pure.
impl PurePredicate<Password> for IsStrong {}

/// Define alias for refined new type.
type StrongPassword = Proven<Password, IsStrong>;

/// Now use the newtype. No need of further validation inside the function.
fn set_password(_username: String, _password: StrongPassword) {
    // ...
}

/// Error type for predicate.
#[derive(Debug, thiserror::Error)]
#[error("Given password is not a strong password")]
struct IsNotStrong;

fn main() {
    let password_input = Password {
        value: "anc1234f".into(),
    };

    let validated_password = StrongPassword::try_new(password_input).unwrap_or_else(|e| {
        let sub = e.subject;
        panic!("Entered password {} is not strong", sub.value);
    });

    set_password("username".into(), validated_password);
}
