use crate::math::muldiv::muldiv;
use crate::math::twamm::exp2::exp2;
use crate::math::uint::U256;
use num_traits::Zero;

const TWO_POW_64: U256 = U256([0, 1, 0, 0]);

fn compute_sqrt_sale_ratio(sale_rate_token0: u128, sale_rate_token1: u128) -> U256 {
    let sale_ratio: U256 = (U256::from(sale_rate_token1) << 128) / sale_rate_token0;

    if sale_ratio >= U256([0, 0, 0, 1]) {
        (sale_ratio << 16).integer_sqrt() << 56
    } else if sale_ratio >= U256([0, 0, 1, 0]) {
        // we know it only has 192 bits, so we can shift it 64 before rooting to get more precision
        (sale_ratio << 64).integer_sqrt() << 32
    } else {
        // full precision
        (sale_ratio << 128).integer_sqrt()
    }
}

fn compute_c(sqrt_ratio: U256, sqrt_sale_ratio: U256) -> (U256, bool) {
    if sqrt_sale_ratio >= sqrt_ratio {
        (
            muldiv(
                sqrt_sale_ratio - sqrt_ratio,
                U256([0, 0, 1, 0]),
                sqrt_sale_ratio + sqrt_ratio,
                false,
            )
            .unwrap(),
            false,
        )
    } else {
        (
            muldiv(
                sqrt_ratio - sqrt_sale_ratio,
                U256([0, 0, 1, 0]),
                sqrt_sale_ratio + sqrt_ratio,
                false,
            )
            .unwrap(),
            true,
        )
    }
}

pub fn calculate_next_sqrt_ratio(
    sqrt_ratio: U256,
    liquidity: u128,
    sale_rate_token0: u128,
    sale_rate_token1: u128,
    time_elapsed: u32,
    fee: u64,
) -> U256 {
    let sqrt_sale_ratio = compute_sqrt_sale_ratio(sale_rate_token0, sale_rate_token1);

    if liquidity.is_zero() {
        return sqrt_sale_ratio;
    }

    let (c, negative) = compute_c(sqrt_sale_ratio, sqrt_ratio);

    if c.is_zero() || liquidity == 0 {
        sqrt_sale_ratio
    } else {
        let sale_rate = ((U256::from(sale_rate_token1) * U256::from(sale_rate_token0))
            .integer_sqrt()
            * (TWO_POW_64 - fee))
            / TWO_POW_64;

        let round_up = sqrt_ratio > sqrt_sale_ratio;

        let exponent: U256 =
            (sale_rate * U256::from(time_elapsed) * U256([12392656037, 0, 0, 0])) / liquidity;

        if exponent >= U256::from(0x400000000000000000_u128) {
            return sqrt_sale_ratio;
        }

        let e_pow_exponent = U256::from(exp2(exponent.low_u128())) << 64;

        let mut sqrt_ratio_next = if negative {
            muldiv(
                sqrt_sale_ratio,
                e_pow_exponent + c,
                e_pow_exponent.abs_diff(c),
                round_up,
            )
            .unwrap_or(sqrt_sale_ratio)
        } else {
            muldiv(
                sqrt_sale_ratio,
                e_pow_exponent + c,
                e_pow_exponent.abs_diff(c),
                round_up,
            )
            .unwrap_or(sqrt_sale_ratio)
        };

        // we should never exceed the sale ratio
        if round_up {
            sqrt_ratio_next = sqrt_ratio_next.max(sqrt_sale_ratio);
        } else {
            sqrt_ratio_next = sqrt_ratio_next.min(sqrt_sale_ratio);
        }

        sqrt_ratio_next
    }
}

#[cfg(test)]
mod tests {
    use crate::math::tick::{MAX_SQRT_RATIO, MIN_SQRT_RATIO};
    use crate::math::twamm::sqrt_ratio::{calculate_next_sqrt_ratio, compute_c};
    use crate::math::uint::U256;
    use alloc::vec;
    use insta::assert_debug_snapshot;

    const ONE_E18: u128 = 1_000_000_000_000_000_000; // 10^18
    const SHIFT_32: u128 = 1u128 << 32; // 2^32 = 4294967296
    const TOKEN_SALE_RATE: u128 = ONE_E18 * SHIFT_32; // 10^18 * 2^32

    struct TestCase {
        description: &'static str,
        sqrt_ratio: U256,
        liquidity: u128,
        token0_sale_rate: u128,
        token1_sale_rate: u128,
        time_elapsed: u32,
        fee: u64,
    }

    #[test]
    fn test_compute_c() {
        assert_eq!(
            compute_c(U256::from(0), U256::from(1)),
            (U256::from(1) << 128, false)
        );
        assert_eq!(
            compute_c(U256::from(1), U256::from(0)),
            (U256::from(1) << 128, true)
        );

        assert_eq!(
            compute_c(U256([0, 0, 1, 0]), U256([0, 0, 2, 0])),
            (
                U256::from_dec_str("113427455640312821154458202477256070485").unwrap(),
                false
            )
        );
        assert_eq!(
            compute_c(U256([0, 0, 2, 0]), U256([0, 0, 1, 0])),
            (
                U256::from_dec_str("113427455640312821154458202477256070485").unwrap(),
                true
            )
        );
        assert_eq!(
            compute_c(U256([0, 0, 1, 0]), U256([0, 0, 1, 0])),
            (U256::from(0), false)
        );

        assert_eq!(
            compute_c(MIN_SQRT_RATIO, MAX_SQRT_RATIO),
            (
                U256::from_dec_str("340282366920938463463374607431768211453").unwrap(),
                false
            )
        );
        assert_eq!(
            compute_c(MAX_SQRT_RATIO, MIN_SQRT_RATIO),
            (
                U256::from_dec_str("340282366920938463463374607431768211453").unwrap(),
                true
            )
        );
    }

    #[test]
    fn test_calculate_next_sqrt_ratio() {
        let test_cases = vec![
            TestCase {
                description: "zero_liquidity_price_eq_sale_ratio",
                sqrt_ratio: U256::zero(),
                liquidity: 0,
                token0_sale_rate: TOKEN_SALE_RATE,
                token1_sale_rate: TOKEN_SALE_RATE,
                time_elapsed: 0,
                fee: 0,
            },
            TestCase {
                description: "large_exponent_price_sqrt_ratio",
                sqrt_ratio: U256::one() << 128,
                liquidity: 1,
                token0_sale_rate: TOKEN_SALE_RATE,
                token1_sale_rate: 1980 * ONE_E18 * SHIFT_32,
                time_elapsed: 1,
                fee: 0,
            },
            TestCase {
                description: "low_liquiidty_same_sale_ratio",
                sqrt_ratio: U256::from(2u128) << 128,
                liquidity: 1,
                token0_sale_rate: TOKEN_SALE_RATE,
                token1_sale_rate: TOKEN_SALE_RATE,
                time_elapsed: 1,
                fee: 0,
            },
            TestCase {
                description: "low_liquidity_token0_gt_token1",
                sqrt_ratio: U256::one() << 128,
                liquidity: 1,
                token0_sale_rate: 2 * TOKEN_SALE_RATE,
                token1_sale_rate: TOKEN_SALE_RATE,
                time_elapsed: 16,
                fee: 0,
            },
            TestCase {
                description: "low_liquidity_token1_gt_token0",
                sqrt_ratio: U256::one() << 128,
                liquidity: 1,
                token0_sale_rate: TOKEN_SALE_RATE,
                token1_sale_rate: 2 * TOKEN_SALE_RATE,
                time_elapsed: 16,
                fee: 0,
            },
            TestCase {
                description: "high_liquidity_same_sale_rate",
                sqrt_ratio: U256::from(2u128) << 128,
                liquidity: 1_000_000 * ONE_E18,
                token0_sale_rate: TOKEN_SALE_RATE,
                token1_sale_rate: TOKEN_SALE_RATE,
                time_elapsed: 1,
                fee: 0,
            },
            TestCase {
                description: "high_liquidity_token0_gt_token1",
                sqrt_ratio: U256::one() << 128,
                liquidity: 1_000_000 * ONE_E18,
                token0_sale_rate: 2 * TOKEN_SALE_RATE,
                token1_sale_rate: TOKEN_SALE_RATE,
                time_elapsed: 1,
                fee: 0,
            },
            TestCase {
                description: "high_liquidity_token1_gt_token0",
                sqrt_ratio: U256::one() << 128,
                liquidity: 1_000_000 * ONE_E18,
                token0_sale_rate: TOKEN_SALE_RATE,
                token1_sale_rate: 2 * TOKEN_SALE_RATE,
                time_elapsed: 1,
                fee: 0,
            },
            TestCase {
                description: "round_in_direction_of_price",
                sqrt_ratio: U256::from_dec_str("481231811499356508086519009265716982182").unwrap(),
                liquidity: 70_710_696_755_630_728_101_718_334,
                token0_sale_rate: 10_526_880_627_450_980_392_156_862_745,
                token1_sale_rate: 10_526_880_627_450_980_392_156_862_745,
                time_elapsed: 2040,
                fee: 0,
            },
            TestCase {
                description: "example0_solidity",
                sqrt_ratio: U256::from_dec_str("3402823669209384634633746074317682114560000")
                    .unwrap(),
                liquidity: 10_000,
                token0_sale_rate: 458864027,
                token1_sale_rate: 280824784,
                time_elapsed: 46_800,
                fee: 0,
            },
        ];

        for test_case in test_cases {
            assert_debug_snapshot!(
                test_case.description,
                calculate_next_sqrt_ratio(
                    test_case.sqrt_ratio,
                    test_case.liquidity,
                    test_case.token0_sale_rate,
                    test_case.token1_sale_rate,
                    test_case.time_elapsed,
                    test_case.fee,
                )
            );
        }
    }
}
