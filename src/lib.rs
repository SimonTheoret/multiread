// PROTOTYPE
use std::fmt::Debug;
mod avx2;
mod avx512;

#[cfg(target_feature = "avx2")]
use avx2::find_all_matches_m256;
#[cfg(target_feature = "avx512bw")]
use avx512::find_all_matches_m512;

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

#[derive(Debug, Clone)]
pub struct MultiJsonlByteParser<'a, AsSlice>
where
    AsSlice: AsRef<[u8]>,
{
    counter: usize,
    quote_counter: usize,
    slice: &'a AsSlice,
    slice_len: usize,
    last: &'a u8,
}

impl<'a, AsSlice> MultiJsonlByteParser<'a, AsSlice>
where
    AsSlice: AsRef<[u8]>,
{
    pub fn new(mmap: &'a AsSlice) -> Self {
        Self {
            counter: 0,
            quote_counter: 0,
            slice_len: mmap.as_ref().len(),
            slice: mmap,
            last: &b'a',
        }
    }
}

#[allow(unreachable_code)]
fn parse_slice_lf<'a, AsSlice>(s: &'a [u8]) -> LocationMap<32> {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx512bw"
    ))]
    return unsafe { find_all_matches_m512(s, b'\n') };

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    return unsafe { find_all_matches_m256(s, b'\n') };
    #[cfg(not(any(target_feature = "avx512bw", target_feature = "avx2")))]
    return find_all_matches_fallback(s, b'\n');
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;
}
