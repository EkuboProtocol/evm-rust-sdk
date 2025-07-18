use super::types::Config;
use crate::math::uint::U256;
use crate::quoting::constants::NATIVE_TOKEN_ADDRESS;
use crate::quoting::full_range_pool::{
    FullRangePool, FullRangePoolQuoteError, FullRangePoolResources, FullRangePoolState,
};
use crate::quoting::types::{BlockTimestamp, NodeKey, Pool, Quote, QuoteParams};
use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OraclePoolState {
    pub full_range_pool_state: FullRangePoolState,
    pub last_snapshot_time: u64,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OraclePoolResources {
    pub full_range_pool_resources: FullRangePoolResources,
    pub snapshots_written: u32,
}

impl AddAssign for OraclePoolResources {
    fn add_assign(&mut self, rhs: Self) {
        self.full_range_pool_resources += rhs.full_range_pool_resources;
        self.snapshots_written += rhs.snapshots_written;
    }
}

impl Add for OraclePoolResources {
    type Output = OraclePoolResources;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign for OraclePoolResources {
    fn sub_assign(&mut self, rhs: Self) {
        self.full_range_pool_resources -= rhs.full_range_pool_resources;
        self.snapshots_written -= rhs.snapshots_written;
    }
}

impl Sub for OraclePoolResources {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OraclePool {
    full_range_pool: FullRangePool,
    last_snapshot_time: u64,
}

/// Errors that can occur when constructing an OraclePool.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OraclePoolError {
    /// Errors from the underlying FullRangePool constructor.
    FullRangePoolError(FullRangePoolError),
}

use crate::quoting::full_range_pool::FullRangePoolError;

impl OraclePool {
    pub fn new(
        token1: U256,
        extension: U256,
        sqrt_ratio: U256,
        active_liquidity: u128,
        last_snapshot_time: u64,
    ) -> Result<Self, OraclePoolError> {
        let full_range_pool = FullRangePool::new(
            NodeKey {
                token0: NATIVE_TOKEN_ADDRESS,
                token1,
                config: Config {
                    fee: 0,
                    tick_spacing: 0,
                    extension,
                },
            },
            FullRangePoolState {
                sqrt_ratio,
                liquidity: active_liquidity,
            },
        )
        .map_err(OraclePoolError::FullRangePoolError)?;

        Ok(OraclePool {
            full_range_pool,
            last_snapshot_time,
        })
    }
}

impl Pool for OraclePool {
    type Resources = OraclePoolResources;
    type State = OraclePoolState;
    type QuoteError = FullRangePoolQuoteError;
    type Meta = BlockTimestamp;

    fn get_key(&self) -> &NodeKey {
        self.full_range_pool.get_key()
    }

    fn get_state(&self) -> Self::State {
        OraclePoolState {
            full_range_pool_state: self.full_range_pool.get_state(),
            last_snapshot_time: self.last_snapshot_time,
        }
    }

    fn quote(
        &self,
        params: QuoteParams<Self::State, Self::Meta>,
    ) -> Result<Quote<Self::Resources, Self::State>, Self::QuoteError> {
        let block_time = params.meta;
        let pool_time = params
            .override_state
            .map_or(self.last_snapshot_time, |os| os.last_snapshot_time);

        let result = self.full_range_pool.quote(QuoteParams {
            sqrt_ratio_limit: params.sqrt_ratio_limit,
            override_state: params.override_state.map(|s| s.full_range_pool_state),
            token_amount: params.token_amount,
            meta: (),
        })?;

        Ok(Quote {
            calculated_amount: result.calculated_amount,
            consumed_amount: result.consumed_amount,
            execution_resources: OraclePoolResources {
                snapshots_written: if pool_time != block_time { 1 } else { 0 },
                full_range_pool_resources: result.execution_resources,
            },
            fees_paid: result.fees_paid,
            is_price_increasing: result.is_price_increasing,
            state_after: OraclePoolState {
                full_range_pool_state: result.state_after,
                last_snapshot_time: block_time,
            },
        })
    }

    fn has_liquidity(&self) -> bool {
        self.full_range_pool.has_liquidity()
    }

    fn max_tick_with_liquidity(&self) -> Option<i32> {
        self.full_range_pool.max_tick_with_liquidity()
    }

    fn min_tick_with_liquidity(&self) -> Option<i32> {
        self.full_range_pool.min_tick_with_liquidity()
    }

    fn is_path_dependent(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::math::tick::to_sqrt_ratio;
    use crate::math::uint::U256;
    use crate::quoting::constants::NATIVE_TOKEN_ADDRESS;
    use crate::quoting::oracle_pool::OraclePool;
    use crate::quoting::types::{Pool, QuoteParams, TokenAmount};

    mod constructor_validation {
        use crate::math::tick::{MAX_SQRT_RATIO, MIN_SQRT_RATIO};
        use crate::math::uint::U256;
        use crate::quoting::oracle_pool::OraclePool;
        use crate::quoting::types::Pool;

        #[test]
        fn test_max_price_constructor() {
            assert_eq!(
                OraclePool::new(U256::one(), U256::zero(), MAX_SQRT_RATIO, 1, 0)
                    .expect("Pool creation should succeed")
                    .get_state()
                    .full_range_pool_state
                    .liquidity,
                1
            );
        }

        #[test]
        fn test_min_price_constructor() {
            assert_eq!(
                OraclePool::new(U256::one(), U256::zero(), MIN_SQRT_RATIO, 1, 0)
                    .expect("Pool creation should succeed")
                    .get_state()
                    .full_range_pool_state
                    .liquidity,
                1
            );
        }

        #[test]
        fn test_min_sqrt_ratio() {
            assert_eq!(
                OraclePool::new(U256::one(), U256::zero(), MIN_SQRT_RATIO, 1, 0)
                    .expect("Pool creation should succeed")
                    .get_state()
                    .full_range_pool_state
                    .liquidity,
                1
            );
        }

        #[test]
        fn test_max_sqrt_ratio() {
            assert_eq!(
                OraclePool::new(U256::one(), U256::zero(), MAX_SQRT_RATIO, 1, 0)
                    .expect("Pool creation should succeed")
                    .get_state()
                    .full_range_pool_state
                    .liquidity,
                1
            );
        }
    }

    const TOKEN: U256 = U256([1, 0, 0, 0]);
    const EXTENSION: U256 = U256([3, 0, 0, 0]);

    #[test]
    fn test_quote_token1_input_update() {
        let pool = OraclePool::new(
            TOKEN,
            EXTENSION,
            to_sqrt_ratio(0).unwrap(),
            1_000_000_000,
            1,
        )
        .expect("Pool creation should succeed");

        let params = QuoteParams {
            token_amount: TokenAmount {
                amount: 1000,
                token: TOKEN,
            },
            sqrt_ratio_limit: None,
            override_state: None,
            meta: 2,
        };

        let quote = pool.quote(params).expect("Failed to get quote");

        assert_eq!(quote.calculated_amount, 999);
        assert_eq!(quote.consumed_amount, 1000);
        assert_eq!(quote.execution_resources.snapshots_written, 1);
        assert_eq!(quote.state_after.last_snapshot_time, 2);
    }

    #[test]
    fn test_quote_token0_input() {
        let pool = OraclePool::new(
            TOKEN,
            EXTENSION,
            to_sqrt_ratio(0).unwrap(),
            1_000_000_000,
            1,
        )
        .expect("Pool creation should succeed");

        let params = QuoteParams {
            token_amount: TokenAmount {
                amount: 1000,
                token: NATIVE_TOKEN_ADDRESS,
            },
            sqrt_ratio_limit: None,
            override_state: None,
            meta: 2,
        };

        let quote = pool.quote(params).expect("Failed to get quote");

        assert_eq!(quote.calculated_amount, 999);
        assert_eq!(quote.consumed_amount, 1000);
        assert_eq!(quote.execution_resources.snapshots_written, 1);
        assert_eq!(quote.state_after.last_snapshot_time, 2);
    }
}
