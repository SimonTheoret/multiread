use std::{array::IntoIter, ptr::addr_of};

#[derive(Debug, Clone, PartialEq)]
struct LocationMap<const N: usize> {
    map: [usize; N],
    len: usize,
}

struct LocationMapIter<const N: usize> {
    lm: LocationMap<N>,
    counter: usize,
}

impl<const N: usize> Iterator for LocationMapIter<N> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.lm.len {
            let res = self.lm.map[self.counter];
            self.counter += 1;
            Some(res)
        } else {
            None
        }
    }
}

impl<const N: usize> IntoIterator for LocationMap<N> {
    type IntoIter = LocationMapIter<N>;
    type Item = usize;
    fn into_iter(self) -> Self::IntoIter {
        LocationMapIter {
            lm: self,
            counter: 0,
        }
    }
}

// #[inline]
// unsafe fn find_all_matches_m256(s: &[u8], b: u8) -> LocationMap<16> {
//     let addr = s.as_ptr() as *const i8;
//     let mut mask = unsafe {
//         let nl_reg = std::arch::x86_64::_mm256_set1_epi8(b as i8);
//         let loaded_slice = std::arch::x86_64::_mm256_loadu_epi8(addr);
//         std::arch::x86_64::_mm256_cmpeq_epi8_mask(nl_reg, loaded_slice)
//     };
//     let mut buf = [0usize; 16];
//     let mut count = 0;
//     while mask != 0 {
//         let pos = mask.trailing_zeros();
//         buf[count] = pos as usize;
//         mask &= mask - 1; // Clear lowest set bit
//         count += 1;
//     }
//     LocationMap {
//         map: buf,
//         len: count,
//     }
// }


#[inline]
unsafe fn find_all_matches_m256(s: &[u8], b: u8) -> LocationMap<16> {
    let addr = s.as_ptr() as *const i8;
    let mut mask = unsafe {
        let nl_reg = std::arch::x86_64::_mm256_set1_epi8(b as i8);
        let loaded_slice = std::arch::x86_64::_mm256_loadu_epi8(addr);
        std::arch::x86_64::_mm256_cmpeq_epi8_mask(nl_reg, loaded_slice)
    };
    let mut buf = [0usize; 16];
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
        let s = [0, 0, 0, 0, 0, 0, 0, b'\n', 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(s.len() == 16);
        let matches = unsafe { find_all_matches_m256(&s, b'\n') };
        let mut inner = [0usize; 16];
        inner[0] = 7;
        let expected = LocationMap { map: inner, len: 1 };
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_all_matches_new_line_size_16_only_one_match_end() {
        let s = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, b'\n'];
        assert!(s.len() == 16);
        let matches = unsafe { find_all_matches_m256(&s, b'\n') };
        let mut inner = [0usize; 16];
        inner[0] = 15;
        let expected = LocationMap { map: inner, len: 1 };
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_all_matches_new_line_size_16_only_one_match_start() {
        let s = [b'\n', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(s.len() == 16);
        let matches = unsafe { find_all_matches_m256(&s, b'\n') };
        let mut inner = [0usize; 16];
        inner[0] = 0;
        let expected = LocationMap { map: inner, len: 1 };
        assert_eq!(matches, expected);
    }
}
