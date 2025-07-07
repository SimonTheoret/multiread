use std::arch::x86_64::*;
use std::mem::transmute;

//TODO: Make an associated constant used to represent the number of bytes to get
// from a slice.
#[derive(Debug, Clone)]
#[repr(transparent)]
struct M256RegisterWrapper(__m256i);

impl M256RegisterWrapper {
    const NUM_OF_BYTES: usize = 32;
    /// Create a M256 register based on a slice `slice` and a starting
    /// index. The starting index is used to copy bytes from the slice. If
    /// `start_index` is `0`, we assume that it is the *first* iteration, and
    /// therefore there is no offset.
    fn from_bytes<AsSlice: AsRef<[u8]>>(slice: AsSlice, start_idx: usize) -> Self {
        // bob: Bag of bytes
        let slice = slice.as_ref();
        let offset = if start_idx == 0 { 0 } else { 1 };
        let mut bob = [0u8; Self::NUM_OF_BYTES as usize];
        let slice_range = (start_idx - offset)..(start_idx + Self::NUM_OF_BYTES as usize - offset);
        for (bob_index, slice_index) in slice_range.enumerate() {
            let val = slice.get(slice_index).cloned().unwrap_or(0u8);
            bob[bob_index] = val;
        }
        Self(unsafe { _mm256_loadu_si256(bob.as_ptr() as *const __m256i) })
    }
    fn contains_byte(self, byte: u8) -> __m256i {
        let const_reg = unsafe { _mm256_set1_epi8(transmute(byte)) };
        unsafe { _mm256_cmpeq_epi8(self.0, const_reg) }
    }
    fn contains_byte_cloned(&self, byte: u8) -> __m256i {
        let const_reg = unsafe { _mm256_set1_epi8(transmute(byte)) };
        unsafe { _mm256_cmpeq_epi8(self.0.clone(), const_reg) }
    }
    pub fn match_bytes(self, num_quotes: usize) -> Vec<u8> {
        let reg_contains_lf: [i8; Self::NUM_OF_BYTES] =
            Self(self.contains_byte_cloned(b'\n')).into();
        let reg_contains_quote: [i8; Self::NUM_OF_BYTES] = Self(self.contains_byte(b'"')).into();
        let mut new_lines_offsets: Vec<u8> = Vec::with_capacity(32);
        let mut num_quotes = num_quotes;
        for idx in 0..Self::NUM_OF_BYTES {
            let i8_lf = reg_contains_lf[idx];
            // means i8_lf is a new line
            if i8_lf < 0 && num_quotes % 2 == 0 {
                new_lines_offsets.push(idx as u8); // idx < 256
                continue;
            };
            let i8_quote = reg_contains_quote[idx];
            if i8_quote < 0 {
                num_quotes += 1;
            }
        }
        new_lines_offsets
    }
}

impl From<M256RegisterWrapper> for [u8; 32] {
    fn from(value: M256RegisterWrapper) -> Self {
        unsafe { transmute(value) }
    }
}
impl From<M256RegisterWrapper> for [i8; 32] {
    fn from(value: M256RegisterWrapper) -> Self {
        unsafe { transmute(value) }
    }
}

// #[repr(align(64))]
// struct M512RegisterWrapper([u8; 64]);

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;
    use pretty_assertions::assert_eq;

    fn byte_vec_m256(start: Option<u8>, end: Option<u8>) -> Vec<u8> {
        let range = Range {
            start: start.unwrap_or(0),
            end: end.unwrap_or(64),
        };
        Vec::from_iter(range.into_iter())
    }

    #[test]
    fn test_get_bytes_with_m256_register_index_32() {
        let bytes = byte_vec_m256(None, None);
        let reg = M256RegisterWrapper::from_bytes(&bytes, 32);
        let actual_slice: [u8; 32] = reg.into();
        assert_eq!(actual_slice.first().unwrap(), &31u8)
    }

    #[test]
    fn test_get_bytes_with_m256_register_index_0() {
        let bytes = byte_vec_m256(None, None);
        let reg = M256RegisterWrapper::from_bytes(&bytes, 0);
        let actual_slice: [u8; 32] = reg.into();
        assert_eq!(
            &bytes.as_slice()[0..32],
            &actual_slice[0..actual_slice.len()]
        )
    }
}
