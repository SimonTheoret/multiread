use crate::LocationMap;

#[inline]
pub unsafe fn find_all_matches_m512(s: &[u8], b: u8) -> LocationMap<32> {
    let addr = s.as_ptr() as *const i8;
    let mut mask = unsafe {
        let nl_reg = std::arch::x86_64::_mm512_set1_epi8(b as i8);
        let loaded_slice = std::arch::x86_64::_mm512_loadu_epi8(addr);
        std::arch::x86_64::_mm512_cmpeq_epi8_mask(nl_reg, loaded_slice)
    };
    let mut buf = [0usize; 32];
    let mut count = 0;
    while mask != 0 {
        let pos = mask.trailing_zeros();
        buf[count] = pos as usize;
        mask &= mask - 1; // Clear lowest set bit
        count += 1;
    }
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
        let matches = unsafe { find_all_matches_m512(&s, b'\n') };
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
        let matches = unsafe { find_all_matches_m512(&s, b'\n') };
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
        let matches = unsafe { find_all_matches_m512(&s, b'\n') };
        let mut inner = [0usize; 32];
        inner[0] = 0;
        let expected = LocationMap { map: inner, len: 1 };
        assert_eq!(matches, expected);
    }
}
