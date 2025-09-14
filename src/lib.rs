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

// //NOTE: Change the location map 32 to 16 for m256 ?
// fn parse_slice_lf_m512(s: &[u8]) -> LocationMap<32> {
//     cfg_if! {
//         if #[cfg(all(target_feature = "avx512bw", target_feature = "avx512f"))] {
//             unsafe { find_all_matches_m512(s, b'\n') }
//         } else if #[cfg(target_feature = "avx2")] {
//              unsafe { find_all_matches_m256(s, b'\n') }
//         } else {
//             find_all_matches_fallback(s, b'\n')
//
//         }
//     }
// }

struct LocMapIter<'a> {
    data: &'a [u8],
    counter: usize,
}

impl<'a> Iterator for LocMapIter<'a> {
    type Item = LocationMap<32>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.counter;
            if start >= self.data.len() {
                return None;
            }
            let end = start + 32;
            let mut extended: [u8; 32];
            let slice = if end <= self.data.len() {
                &self.data[start..end]
            } else {
                let short_slice = &self.data[start..];
                extended = [0; 32];
                extended[..short_slice.len()].copy_from_slice(short_slice);
                &extended
            };
            let loc_map = unsafe { find_all_matches_m512(slice, b'\n') };
            if loc_map.len == 0 {
                continue;
            }
            self.counter += 32;
            return Some(loc_map);
        }
    }
}

pub struct NewLinePositionIter<'a> {
    data: &'a [u8],
    iter: LocMapIter<'a>,
    buf: LocationMap<32>,
    buf_index: usize,
}
impl<'a> NewLinePositionIter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let buf = LocationMap {
            map: [0; 32],
            len: 0,
        };
        let iter = LocMapIter { data, counter: 0 };
        Self {
            data,
            iter,
            buf,
            buf_index: 0,
        }
    }
}

impl<'a> Iterator for NewLinePositionIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buf_index < self.buf.len {
            let res = Some(self.buf.map[self.buf_index]);
            self.buf_index += 1;
            return res;
        }
        // Non-emtpy (len >= 1) location map
        // See LocMapIter.next()
        if let Some(loc_map) = self.iter.next() {
            self.buf_index = 0;
            self.buf = loc_map;
            let res = self.buf.map[self.buf_index];
            Some(res)
        } else {
            None
        }
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
        let iter = NewLinePositionIter::new(exemple);
        let actual_newlines: Vec<_> = iter.collect();
        dbg!(&actual_newlines);
        for pos in actual_newlines {
            assert_eq!(exemple[pos], b'\n')
        }
    }
}
