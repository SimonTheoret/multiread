use std::fmt::Debug;
mod avx2;
mod avx512;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(target_feature = "avx512bw", target_feature = "avx512f"))] {
        use avx512::find_all_matches_m512;
    } else if #[cfg(target_feature = "avx2")] {
        use avx2::find_all_matches_m256;
    } else {}
}

fn find_all_matches_fallback<const N: usize>(s: &[u8], b: u8) -> LocationMap<N> {
    let mut buf = [0usize; N];
    let mut counter = 0;
    let mut buf_len = 0;
    while counter <= s.len() {
        if s[counter] == b {
            buf[buf_len] = counter;
            buf_len += 1;
        }
        counter += 1;
    }
    LocationMap {
        len: buf_len,
        map: buf,
    }
}

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

//NOTE: Change the location map 32 to 16 for m256 ?
fn parse_slice_lf(s: &[u8]) -> LocationMap<32> {
    cfg_if! {
        if #[cfg(all(target_feature = "avx512bw", target_feature = "avx512f"))] {
            unsafe { find_all_matches_m512(s, b'\n') }
        } else if #[cfg(target_feature = "avx2")] {
             unsafe { find_all_matches_m256(s, b'\n') }
        } else {
            find_all_matches_fallback(s, b'\n')

        }
    }
}

pub struct LineParser<'a>(&'a [u8]);

impl<'a> LineParser<'a> {
    const OFFSET_MULTIPLIER: usize = 32;
}

impl<'a> Iterator for LineParser<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        parse_slice_lf()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_newline_finder() {
        let exemple = r#"First sentence
            Second sentence
            Third"#
            .as_bytes();
        let expected_content = vec![
            "First sentence".as_bytes(),
            "Second sentence".as_bytes(),
            "Third".as_bytes(),
        ];
        let iter = LineParser(&exemple);
        assert_eq!(iter.collect(), expected_content);
    }
}
