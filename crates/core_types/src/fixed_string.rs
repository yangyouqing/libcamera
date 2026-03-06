/// Stack-allocated fixed-capacity string. UTF-8 validated on construction.
#[derive(Clone)]
pub struct FixedString<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> FixedString<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let mut fs = Self::new();
        let copy_len = s.len().min(N);
        fs.buf[..copy_len].copy_from_slice(&s.as_bytes()[..copy_len]);
        // Ensure we don't split a UTF-8 multi-byte char
        while copy_len > 0 && !core::str::from_utf8(&fs.buf[..copy_len]).is_ok() {
            // This path handles edge truncation of multi-byte chars
            break;
        }
        fs.len = match core::str::from_utf8(&fs.buf[..copy_len]) {
            Ok(_) => copy_len,
            Err(e) => e.valid_up_to(),
        };
        fs
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        core::str::from_utf8(bytes).ok()?;
        let mut fs = Self::new();
        let copy_len = bytes.len().min(N);
        let valid = match core::str::from_utf8(&bytes[..copy_len]) {
            Ok(_) => copy_len,
            Err(e) => e.valid_up_to(),
        };
        fs.buf[..valid].copy_from_slice(&bytes[..valid]);
        fs.len = valid;
        Some(fs)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: we validate UTF-8 on construction
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
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

    pub fn push_str(&mut self, s: &str) -> bool {
        let remaining = N - self.len;
        if s.len() > remaining {
            return false;
        }
        self.buf[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        true
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Truncate to the given byte length, adjusting down to a valid UTF-8 boundary.
    pub fn truncate(&mut self, max_len: usize) {
        if max_len >= self.len {
            return;
        }
        let target = max_len.min(self.len);
        self.len = match core::str::from_utf8(&self.buf[..target]) {
            Ok(_) => target,
            Err(e) => e.valid_up_to(),
        };
    }
}

impl<const N: usize> core::fmt::Debug for FixedString<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\"{}\"", self.as_str())
    }
}

impl<const N: usize> core::fmt::Display for FixedString<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const N: usize> PartialEq for FixedString<N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> PartialEq<str> for FixedString<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> core::fmt::Write for FixedString<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.push_str(s) {
            Ok(())
        } else {
            // Silently truncate rather than error, for logging use
            let remaining = N - self.len;
            if remaining > 0 {
                let valid = match core::str::from_utf8(&s.as_bytes()[..remaining]) {
                    Ok(_) => remaining,
                    Err(e) => e.valid_up_to(),
                };
                if valid > 0 {
                    self.buf[self.len..self.len + valid]
                        .copy_from_slice(&s.as_bytes()[..valid]);
                    self.len += valid;
                }
            }
            Ok(())
        }
    }
}
