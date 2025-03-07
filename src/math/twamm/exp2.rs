use crate::math::uint::U256;

pub fn exp2(x: i128) -> u128 {
    // x must be less than 0x400000000000000000 (overflow check)
    assert!(x < 0x400000000000000000, "Overflow");
    // Underflow: if x is too negative, return 0.
    if x < -0x400000000000000000 {
        return 0;
    }

    // Start with 0x80000000000000000000000000000000 (i.e. 2^127)
    let mut result = U256::one() << 127;

    macro_rules! mul_shift {
        ($mask:expr, $factor:expr) => {
            if (x & $mask) != 0 {
                let factor = U256::from_str_radix($factor, 16).unwrap();
                result = (result * factor) >> 128;
            }
        };
    }

    // Each branch multiplies result by a precomputed constant if the corresponding bit is set.
    mul_shift!(0x8000000000000000_i128, "16A09E667F3BCC908B2FB1366EA957D3E");
    mul_shift!(0x4000000000000000, "1306FE0A31B7152DE8D5A46305C85EDEC");
    mul_shift!(0x2000000000000000, "1172B83C7D517ADCDF7C8C50EB14A791F");
    mul_shift!(0x1000000000000000, "10B5586CF9890F6298B92B71842A98363");
    mul_shift!(0x800000000000000, "1059B0D31585743AE7C548EB68CA417FD");
    mul_shift!(0x400000000000000, "102C9A3E778060EE6F7CACA4F7A29BDE8");
    mul_shift!(0x200000000000000, "10163DA9FB33356D84A66AE336DCDFA3F");
    mul_shift!(0x100000000000000, "100B1AFA5ABCBED6129AB13EC11DC9543");
    mul_shift!(0x80000000000000, "10058C86DA1C09EA1FF19D294CF2F679B");
    mul_shift!(0x40000000000000, "1002C605E2E8CEC506D21BFC89A23A00F");
    mul_shift!(0x20000000000000, "100162F3904051FA128BCA9C55C31E5DF");
    mul_shift!(0x10000000000000, "1000B175EFFDC76BA38E31671CA939725");
    mul_shift!(0x8000000000000, "100058BA01FB9F96D6CACD4B180917C3D");
    mul_shift!(0x4000000000000, "10002C5CC37DA9491D0985C348C68E7B3");
    mul_shift!(0x2000000000000, "1000162E525EE054754457D5995292026");
    mul_shift!(0x1000000000000, "10000B17255775C040618BF4A4ADE83FC");
    mul_shift!(0x800000000000, "1000058B91B5BC9AE2EED81E9B7D4CFAB");
    mul_shift!(0x400000000000, "100002C5C89D5EC6CA4D7C8ACC017B7C9");
    mul_shift!(0x200000000000, "10000162E43F4F831060E02D839A9D16D");
    mul_shift!(0x100000000000, "100000B1721BCFC99D9F890EA06911763");
    mul_shift!(0x80000000000, "10000058B90CF1E6D97F9CA14DBCC1628");
    mul_shift!(0x40000000000, "1000002C5C863B73F016468F6BAC5CA2B");
    mul_shift!(0x20000000000, "100000162E430E5A18F6119E3C02282A5");
    mul_shift!(0x10000000000, "1000000B1721835514B86E6D96EFD1BFE");
    mul_shift!(0x8000000000, "100000058B90C0B48C6BE5DF846C5B2EF");
    mul_shift!(0x4000000000, "10000002C5C8601CC6B9E94213C72737A");
    mul_shift!(0x2000000000, "1000000162E42FFF037DF38AA2B219F06");
    mul_shift!(0x1000000000, "10000000B17217FBA9C739AA5819F44F9");
    mul_shift!(0x800000000, "1000000058B90BFCDEE5ACD3C1CEDC823");
    mul_shift!(0x400000000, "100000002C5C85FE31F35A6A30DA1BE50");
    mul_shift!(0x200000000, "10000000162E42FF0999CE3541B9FFFCF");
    mul_shift!(0x100000000, "100000000B17217F80F4EF5AADDA45554");
    mul_shift!(0x80000000, "10000000058B90BFBF8479BD5A81B51AD");
    mul_shift!(0x40000000, "1000000002C5C85FDF84BD62AE30A74CC");
    mul_shift!(0x20000000, "100000000162E42FEFB2FED257559BDAA");
    mul_shift!(0x10000000, "1000000000B17217F7D5A7716BBA4A9AE");
    mul_shift!(0x8000000, "100000000058B90BFBE9DDBAC5E109CCE");
    mul_shift!(0x4000000, "10000000002C5C85FDF4B15DE6F17EB0D");
    mul_shift!(0x2000000, "1000000000162E42FEFA494F1478FDE05");
    mul_shift!(0x1000000, "10000000000B17217F7D20CF927C8E94C");
    mul_shift!(0x800000, "1000000000058B90BFBE8F71CB4E4B33D");
    mul_shift!(0x400000, "100000000002C5C85FDF477B662B26945");
    mul_shift!(0x200000, "10000000000162E42FEFA3AE53369388C");
    mul_shift!(0x100000, "100000000000B17217F7D1D351A389D40");
    mul_shift!(0x80000, "10000000000058B90BFBE8E8B2D3D4EDE");
    mul_shift!(0x40000, "1000000000002C5C85FDF4741BEA6E77E");
    mul_shift!(0x20000, "100000000000162E42FEFA39FE95583C2");
    mul_shift!(0x10000, "1000000000000B17217F7D1CFB72B45E1");
    mul_shift!(0x8000, "100000000000058B90BFBE8E7CC35C3F0");
    mul_shift!(0x4000, "10000000000002C5C85FDF473E242EA38");
    mul_shift!(0x2000, "1000000000000162E42FEFA39F02B772C");
    mul_shift!(0x1000, "10000000000000B17217F7D1CF7D83C1A");
    mul_shift!(0x800, "1000000000000058B90BFBE8E7BDCBE2E");
    mul_shift!(0x400, "100000000000002C5C85FDF473DEA871F");
    mul_shift!(0x200, "10000000000000162E42FEFA39EF44D91");
    mul_shift!(0x100, "100000000000000B17217F7D1CF79E949");
    mul_shift!(0x80, "10000000000000058B90BFBE8E7BCE544");
    mul_shift!(0x40, "1000000000000002C5C85FDF473DE6ECA");
    mul_shift!(0x20, "100000000000000162E42FEFA39EF366F");
    mul_shift!(0x10, "1000000000000000B17217F7D1CF79AFA");
    mul_shift!(0x8, "100000000000000058B90BFBE8E7BCD6D");
    mul_shift!(0x4, "10000000000000002C5C85FDF473DE6B2");
    mul_shift!(0x2, "1000000000000000162E42FEFA39EF358");
    mul_shift!(0x1, "10000000000000000B17217F7D1CF79AB");

    // Final adjustment: shift right by 63 - (x >> 64). (x >> 64) is the integer part.
    let shift = (63 - (x >> 64)) as u32;
    result >>= shift;
    // Ensure the final result fits in u128.
    assert!(result <= U256::from(u128::MAX));
    result.as_u128()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exp2_cases() {
        assert_eq!(exp2(0), 1 << 64);
    }
}
