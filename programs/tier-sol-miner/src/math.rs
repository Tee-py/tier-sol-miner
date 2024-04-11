pub fn muldiv(a: u64, b: u64, c: u64) -> u64 {
    ((a as u128 * b as u128)/c as u128) as u64
}

pub fn calculate_fee(amount: u64, fee: u64) -> u64 {
    muldiv(amount, fee, 10000)
}

pub fn calculate_interest(amount: u64, apy: u64, interval: u64) -> u64 {
    interval * muldiv(apy, amount, 315_360_000_000)
}