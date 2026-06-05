//! Reed-Solomon codes operate over finite fields. QR codes use GF(2^8) with
//! the primitive polynomial 0x11D (285 in decimal).
//! Division/multiplication are defined via log/exp lookup tables.
//!
//! ## Log/Exp tables
//!
//! GF multiplication: exp[log[a] + log[b]] - O(1).
//! alpha = 2 (generator), exp[i] = 2^i mod 0x11D, log[exp[i]] = i


/// GF(2^8) with primitive polynomial 0x11D.
/// This is the field QR codes use.
pub struct GF256 {
    /// exp[i] = alpha^i in the field
    exp: [u8; 256],
    /// log[value] = i where alpha^i = value. log[0] is unused (set to 0xFF as sentinel)
    log: [u8; 256],
    /// Inverse table: inv[i] = i^-1 in the field. inv[0] unused.
    inv: [u8; 256],
}

impl GF256 {
    /// The primitive polynomial used by QR codes: x^8 + x^4 + x^3 + x^2 + 1
    /// In hex: 0x11D = binary 1_0001_1101
    const PRIM_POLY: u16 = 0x11D;

    /// Create a new GF(2^8) instance. Uses alpha = 2 as the generator.
    pub fn new() -> Self {
        let mut exp = [0u8; 256];
        let mut log = [0u8; 256];
        let mut inv = [0u8; 256];

        let mut value: u16 = 1;

        for i in 0..255 {
            exp[i] = value as u8;
            log[value as usize] = i as u8;
            // alpha^i has inverse alpha^(255-i), so inv[alpha^i] = alpha^(255-i)
            if i > 0 {
                inv[value as usize] = exp[(255 - i) as usize];
            }

            // Multiply by alpha (2) with reduction mod PRIM_POLY
            value <<= 1;
            if value & 0x100 != 0 {
                value ^= Self::PRIM_POLY;
            }
        }

        // exp[255] wraps back to 1 (field order is 255)
        exp[255] = exp[0];
        // log[0] = 255 (sentinel - 0 has no log)
        log[0] = 0xFF;
        // inv[0] unused, set to 0
        inv[0] = 0;
        // inv[1] = 1 (1 is its own inverse)
        inv[1] = 1;

        Self { exp, log, inv }
    }

    /// Multiply two field elements: a × b
    /// Returns 0 if either input is 0.
    ///
    /// Logic: exp[log[a] + log[b]] where + is integer addition mod 255.
    /// Since exp and log are inverses, this gives us field multiplication.
    #[inline]
    pub fn mul(&self, a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }
        let log_sum = (self.log[a as usize] as usize) + (self.log[b as usize] as usize);
        self.exp[log_sum % 255]
    }

    /// Divide two field elements: a / b
    /// Returns 0 if a is 0.
    /// Panics if b is 0 (division by zero).
    ///
    /// Logic: exp[log[a] - log[b]] where - is integer subtraction mod 255.
    /// Since inv[b] = exp[255 - log[b]], we can also compute: a × inv[b]
    #[inline]
    pub fn div(&self, a: u8, b: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        if b == 0 {
            panic!("GF(2^8) division by zero");
        }
        let log_diff = (self.log[a as usize] as isize) - (self.log[b as usize] as isize);
        let idx = ((log_diff % 255) + 255) as usize % 255;
        self.exp[idx]
    }

    /// Inverse of a field element: a^-1
    /// Returns 0 if a is 0.
    ///
    /// Using the property: a = alpha^k ⇒ a^-1 = alpha^(255-k)
    #[inline]
    pub fn inv(&self, a: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        // a = exp[k] ⇒ inv[a] = exp[255 - k]
        let k = self.log[a as usize] as usize;
        self.exp[(255 - k) % 255]
    }

    /// Exponentiate: alpha^k in the field
    #[inline]
    pub fn pow(&self, k: u8) -> u8 {
        let k = k % 255;
        self.exp[k as usize]
    }
}

impl Default for GF256 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_identity() {
        let gf = GF256::new();

        // alpha^0 = 1
        assert_eq!(gf.pow(0), 1);
        // alpha^1 = 2 (generator)
        assert_eq!(gf.pow(1), 2);
        // alpha^255 = 1 (field order)
        assert_eq!(gf.pow(255), 1);
    }

    #[test]
    fn test_multiplication() {
        let gf = GF256::new();

        // 1 × 1 = 1
        assert_eq!(gf.mul(1, 1), 1);
        // 2 × 2 = 4 (alpha × alpha = alpha^2)
        assert_eq!(gf.mul(2, 2), 4);
        // 3 × 5 = 15 (cross-check)
        assert_eq!(gf.mul(3, 5), 15);
    }

    #[test]
    fn test_inverse() {
        let gf = GF256::new();

        // 1^-1 = 1
        assert_eq!(gf.inv(1), 1);
        // 2 * 2^-1 = 1 (verifies the inverse is correct)
        assert_eq!(gf.mul(2, gf.inv(2)), 1);
        // a × a^-1 = 1 for a != 0
        for a in 1..=255 {
            assert_eq!(gf.mul(a, gf.inv(a)), 1);
        }
    }

    #[test]
    fn test_division() {
        let gf = GF256::new();

        // 4 / 2 = 2
        assert_eq!(gf.div(4, 2), 2);
        // a / a = 1 for a != 0
        for a in 1..=255 {
            assert_eq!(gf.div(a, a), 1);
        }
    }
}
