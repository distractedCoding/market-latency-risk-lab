pub fn calculate_orders_per_sec(processed_orders: u64, elapsed_nanos: u128) -> u64 {
    if elapsed_nanos == 0 {
        return 0;
    }

    let scaled_orders = (processed_orders as u128).saturating_mul(1_000_000_000);
    let achieved = scaled_orders / elapsed_nanos;
    u64::try_from(achieved).unwrap_or(u64::MAX)
}

pub fn meets_target_orders_per_sec(
    achieved_orders_per_sec: u64,
    target_orders_per_sec: u64,
) -> bool {
    achieved_orders_per_sec >= target_orders_per_sec
}

#[cfg(test)]
mod tests {
    use super::{calculate_orders_per_sec, meets_target_orders_per_sec};

    #[test]
    fn calculates_orders_per_second_from_elapsed_nanos() {
        let achieved = calculate_orders_per_sec(1_500, 1_000_000_000);
        assert_eq!(achieved, 1_500);
    }

    #[test]
    fn handles_sub_second_windows() {
        let achieved = calculate_orders_per_sec(500, 250_000_000);
        assert_eq!(achieved, 2_000);
    }

    #[test]
    fn zero_elapsed_nanos_returns_zero() {
        let achieved = calculate_orders_per_sec(500, 0);
        assert_eq!(achieved, 0);
    }

    #[test]
    fn meets_target_when_achieved_is_equal_or_greater() {
        assert!(meets_target_orders_per_sec(1_000, 1_000));
        assert!(meets_target_orders_per_sec(1_001, 1_000));
        assert!(!meets_target_orders_per_sec(999, 1_000));
    }
}
