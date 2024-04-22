use crate::errors::MinerError;
// use uint::construct_uint;

// construct_uint! {
//     pub struct U256(4);
// }

// type InnerUint = U256;
// pub const ONE: u128 = 1_000_000_000_000;

// #[derive(Clone, Debug, PartialEq)]
// pub struct PreciseNumber {
//     /// Wrapper over the inner value, which is multiplied by ONE
//     pub value: InnerUint,
// }

// /// The precise-number 1 as a InnerUint
// fn one() -> InnerUint {
//     InnerUint::from(ONE)
// }

// /// The number 0 as a PreciseNumber, used for easier calculations.
// fn zero() -> InnerUint {
//     InnerUint::from(0)
// }

// impl PreciseNumber {
//     /// Create a precise number from an imprecise u128, should always succeed
//     pub fn new(value: u128) -> Option<Self> {
//         let value = InnerUint::from(value).checked_mul(one())?;
//         Some(Self { value })
//     }

//     fn rounding_correction() -> InnerUint {
//         InnerUint::from(ONE / 2)
//     }

//     fn zero() -> Self {
//         Self { value: zero() }
//     }

//     /// Convert a precise number back to u128
//     pub fn to_imprecise(&self) -> Option<u128> {
//         self.value
//             .checked_add(Self::rounding_correction())?
//             .checked_div(one())
//             .map(|v| v.as_u128())
//     }

//     /// Checks that two PreciseNumbers are equal within some tolerance
//     pub fn almost_eq(&self, rhs: &Self, precision: InnerUint) -> bool {
//         let (difference, _) = self.unsigned_sub(rhs);
//         difference.value < precision
//     }

//     /// Checks that a number is less than another
//     pub fn less_than(&self, rhs: &Self) -> bool {
//         self.value < rhs.value
//     }

//     /// Checks that a number is greater than another
//     pub fn greater_than(&self, rhs: &Self) -> bool {
//         self.value > rhs.value
//     }

//     /// Checks that a number is less than another
//     pub fn less_than_or_equal(&self, rhs: &Self) -> bool {
//         self.value <= rhs.value
//     }

//     /// Checks that a number is greater than another
//     pub fn greater_than_or_equal(&self, rhs: &Self) -> bool {
//         self.value >= rhs.value
//     }

//     /// Floors a precise value to a precision of ONE
//     pub fn floor(&self) -> Option<Self> {
//         let value = self.value.checked_div(one())?.checked_mul(one())?;
//         Some(Self { value })
//     }

//     /// Ceiling a precise value to a precision of ONE
//     pub fn ceiling(&self) -> Option<Self> {
//         let value = self
//             .value
//             .checked_add(one().checked_sub(InnerUint::from(1))?)?
//             .checked_div(one())?
//             .checked_mul(one())?;
//         Some(Self { value })
//     }

//     /// Performs a checked division on two precise numbers
//     pub fn checked_div(&self, rhs: &Self) -> Option<Self> {
//         if *rhs == Self::zero() {
//             return None;
//         }
//         match self.value.checked_mul(one()) {
//             Some(v) => {
//                 let value = v
//                     .checked_add(Self::rounding_correction())?
//                     .checked_div(rhs.value)?;
//                 Some(Self { value })
//             }
//             None => {
//                 let value = self
//                     .value
//                     .checked_add(Self::rounding_correction())?
//                     .checked_div(rhs.value)?
//                     .checked_mul(one())?;
//                 Some(Self { value })
//             }
//         }
//     }

//     /// Performs a multiplication on two precise numbers
//     pub fn checked_mul(&self, rhs: &Self) -> Option<Self> {
//         match self.value.checked_mul(rhs.value) {
//             Some(v) => {
//                 let value = v
//                     .checked_add(Self::rounding_correction())?
//                     .checked_div(one())?;
//                 Some(Self { value })
//             }
//             None => {
//                 let value = if self.value >= rhs.value {
//                     self.value.checked_div(one())?.checked_mul(rhs.value)?
//                 } else {
//                     rhs.value.checked_div(one())?.checked_mul(self.value)?
//                 };
//                 Some(Self { value })
//             }
//         }
//     }

//     /// Performs addition of two precise numbers
//     pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
//         let value = self.value.checked_add(rhs.value)?;
//         Some(Self { value })
//     }

//     /// Subtracts the argument from self
//     pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
//         let value = self.value.checked_sub(rhs.value)?;
//         Some(Self { value })
//     }

//     /// Performs a subtraction, returning the result and whether the result is
//     /// negative
//     pub fn unsigned_sub(&self, rhs: &Self) -> (Self, bool) {
//         match self.value.checked_sub(rhs.value) {
//             None => {
//                 let value = rhs.value.checked_sub(self.value).unwrap();
//                 (Self { value }, true)
//             }
//             Some(value) => (Self { value }, false),
//         }
//     }
// }

pub fn muldiv(a: u128, b: u128, c: u128) -> Option<u128> {
    a.checked_mul(b)?.checked_div(c)
}

pub fn to_u128(val: u64) -> Result<u128, MinerError> {
    val.try_into().map_err(|_| MinerError::ConversionFailure)
}

pub fn to_u64(val: u128) -> Result<u64, MinerError> {
    val.try_into().map_err(|_| MinerError::ConversionFailure)
}

pub fn calculate_fee(amount: u128, fee: u128) -> Result<u64, MinerError> {
    match muldiv(amount, fee, 
        to_u128(10000)?
    ) {
      Some(val) => to_u64(val),
      None => Err(MinerError::MathsError)
    }
}

pub fn calculate_interest(amount: u128, apy: u128, interval: u128) -> Result<u64, MinerError> {
    match muldiv(apy, amount, to_u128(315_360_000_000)?) {
        Some(interest_per_unit_period) => match interest_per_unit_period.checked_mul(interval) {
            Some(val) => to_u64(val),
            None => Err(MinerError::MathsError)
        },
        None => Err(MinerError::MathsError)
    }
}