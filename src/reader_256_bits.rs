#[derive(Debug, Clone)]
pub struct MultiJsonl256BitsParser<'a, AsSlice>
where
    AsSlice: AsRef<[u8]>,
{
    counter: usize,
    quote_counter: usize,
    slice: &'a AsSlice,
    slice_len: usize,
    last: &'a u8,
}

impl<'a, AsSlice> MultiJsonl256BitsParser<'a, AsSlice>
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

impl<'a, AsSlice> Iterator for MultiJsonl256BitsParser<'a, AsSlice>
where
    AsSlice: AsRef<[u8]>,
{
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_iter_counter = self.counter;
        loop {
            if current_iter_counter >= self.slice_len {
                return None;
            }
            let b = unsafe { self.slice.as_ref().get_unchecked(current_iter_counter) };
            match b {
                // End of line and inside a string
                b'\n' if self.quote_counter % 2 == 1 => self.last = b,

                // End of line and NOT inside a string
                b'\n' if self.quote_counter % 2 == 0 => {
                    self.last = b;
                    current_iter_counter += 1;
                    break;
                }
                //
                // Quote inside a string
                b'"' if self.last == &b'\\' => {
                    self.last = b;
                }

                // Quote, but not inside a string
                b'"' if self.last != &b'\\' => {
                    self.last = b;
                    self.quote_counter += 1;
                }
                _ => self.last = b,
            };
            current_iter_counter += 1;
        }
        if current_iter_counter == 0 {
            None
        } else {
            let lower_bound = self.counter;
            self.counter = current_iter_counter;
            let out = &self.slice.as_ref()[lower_bound..self.counter];
            Some(out)
        }
    }
}
