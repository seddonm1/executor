use crate::bindings::{
    self,
    component::workflow::abi::{Content, GuestToHost, HostToGuest, Types},
};

/// Generates a random value.
///
/// This function makes a call to the host environment to generate a random
/// numver using the thread-local random number generator.
///
/// This function is generic over types that implement the `Rand` trait.
///
/// # Examples
///
/// ```
/// let random_bool = rand::<bool>();
/// let random_i32 = rand::<i32>();
/// let random_f32 = rand::<f32>();
/// ```
pub fn rand<A: Rand>() -> A {
    A::rand()
}

/// A trait for types that can be randomly generated.
#[allow(unused)]
pub trait Rand {
    /// Generates a random value of the implementing type.
    fn rand() -> Self;
}

impl Rand for bool {
    /// Generates a random boolean value.
    fn rand() -> Self {
        match bindings::call(&GuestToHost::Random(Types::BoolType(bool::default()))) {
            HostToGuest {
                content: Content::Value(Types::BoolType(value)),
                ..
            } => value,
            m => {
                log::error!("expected Content::Random(Types::BoolType got {:?}", m);
                unreachable!()
            }
        }
    }
}

impl Rand for i32 {
    /// Generates a random 32-bit signed integer.
    fn rand() -> Self {
        match bindings::call(&GuestToHost::Random(Types::I32Type(u32::default()))) {
            HostToGuest {
                content: Content::Value(Types::I32Type(value)),
                ..
            } => value as i32,
            m => {
                log::error!("expected Content::Random(Types::I32Type got {:?}", m);
                unreachable!()
            }
        }
    }
}

impl Rand for f32 {
    /// Generates a random 32-bit floating-point number.
    fn rand() -> Self {
        match bindings::call(&GuestToHost::Random(Types::F32Type(0.0))) {
            HostToGuest {
                content: Content::Value(Types::F32Type(value)),
                ..
            } => value,
            m => {
                log::error!("expected Content::Random(Types::F32Type got {:?}", m);
                unreachable!()
            }
        }
    }
}
