use std::{arch::x86_64::__m256i, slice::from_raw_parts};

use crate::LocationMap;

#[inline]
#[target_feature(enable = "avx,avx2")]
/// WARNING: This function assumes that `s` is of length 32 (i.e. contains 32 u8).
pub unsafe fn find_all_matches_m256(s: &[u8], b: u8) -> LocationMap<32> {
    let bag = s.as_ptr() as *const __m256i;
    let are_eq = unsafe {
        // feature: avx
        let nl_reg = std::arch::x86_64::_mm256_set1_epi8(b as i8);
        // feature: avx
        let loaded_slice = std::arch::x86_64::_mm256_loadu_si256(bag);
        // feature: avx2
        std::arch::x86_64::_mm256_cmpeq_epi8(nl_reg, loaded_slice)
    };
    let bob: [u8; 32] = unsafe { std::mem::transmute(are_eq) }; // BAG OF BYTES (BOB)
    let mut buf = [0usize; 32];
    let mut count = 0;
    (0..32).for_each(|idx| {
        if bob[idx] != 0 {
            buf[count] = idx;
            count += 1;
        }
    });
    LocationMap {
        map: buf,
        len: count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_all_matches_new_line_size_16_only_one_match() {
        let s = [
            0, 0, 0, 0, 0, 0, 0, b'\n', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        assert!(s.len() == 32);
        let matches = unsafe { find_all_matches_m256(&s, b'\n') };
        let mut inner = [0usize; 32];
        inner[0] = 7;
        let expected = LocationMap { map: inner, len: 1 };
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_all_matches_new_line_size_16_only_one_match_end() {
        let s = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, b'\n', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        assert!(s.len() == 32);
        let matches = unsafe { find_all_matches_m256(&s, b'\n') };
        let mut inner = [0usize; 32];
        inner[0] = 15;
        let expected = LocationMap { map: inner, len: 1 };
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_all_matches_new_line_size_16_only_one_match_start() {
        let s = [
            b'\n', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        assert!(s.len() == 32);
        let matches = unsafe { find_all_matches_m256(&s, b'\n') };
        let mut inner = [0usize; 32];
        inner[0] = 0;
        let expected = LocationMap { map: inner, len: 1 };
        assert_eq!(matches, expected);
    }
}
