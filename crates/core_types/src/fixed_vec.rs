/// Stack-allocated fixed-capacity vector. No heap allocation needed.
pub struct FixedVec<T, const N: usize> {
    buf: [core::mem::MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub const fn new() -> Self {
        Self {
            // SAFETY: MaybeUninit does not require initialization
            buf: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len >= N
    }

    pub fn push(&mut self, val: T) -> Result<(), T> {
        if self.len >= N {
            return Err(val);
        }
        self.buf[self.len] = core::mem::MaybeUninit::new(val);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: element at self.len was initialized via push()
        Some(unsafe { self.buf[self.len].assume_init_read() })
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: first self.len elements are initialized
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr() as *const T, self.len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: first self.len elements are initialized
        unsafe { core::slice::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut T, self.len) }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: index < self.len, element is initialized
        Some(unsafe { self.buf[index].assume_init_ref() })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: index < self.len, element is initialized
        Some(unsafe { self.buf[index].assume_init_mut() })
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: element is initialized
        let val = unsafe { self.buf[index].assume_init_read() };
        // Shift remaining elements left
        for i in index..self.len - 1 {
            // SAFETY: elements [index+1..len] are initialized
            unsafe {
                let next = self.buf[i + 1].assume_init_read();
                self.buf[i] = core::mem::MaybeUninit::new(next);
            }
        }
        self.len -= 1;
        Some(val)
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }
}

impl<T, const N: usize> Drop for FixedVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T: core::fmt::Debug, const N: usize> core::fmt::Debug for FixedVec<T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: Clone, const N: usize> Clone for FixedVec<T, N> {
    fn clone(&self) -> Self {
        let mut v = Self::new();
        for item in self.as_slice() {
            let _ = v.push(item.clone());
        }
        v
    }
}

impl<T: PartialEq, const N: usize> PartialEq for FixedVec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
