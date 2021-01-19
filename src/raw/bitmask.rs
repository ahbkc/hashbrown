use super::imp::{BitMaskWord, BITMASK_MASK, BITMASK_STRIDE};
#[cfg(feature = "nightly")]
use core::intrinsics;

/// A bit mask which contains the result of a `Match` operation on a `Group` and
/// allows iterating through them.
///
/// The bit mask is arranged so that low-order bits represent lower memory
/// addresses for group match results.
///
/// For implementation reasons, the bits in the set may be sparsely packed, so
/// that there is only one bit-per-byte used (the high bit, 7). If this is the
/// case, `BITMASK_STRIDE` will be 8 to indicate a divide-by-8 should be
/// performed on counts/indices to normalize this difference. `BITMASK_MASK` is
/// similarly a mask of all the actually-used bits.
#[derive(Copy, Clone)]
pub struct BitMask(pub BitMaskWord);

#[allow(clippy::use_self)]
impl BitMask {
    /// Returns a new `BitMask` with all bits inverted.
    #[inline]
    #[must_use]
    pub fn invert(self) -> Self {
        BitMask(self.0 ^ BITMASK_MASK)
    }

    /// Flip the bit in the mask for the entry at the given index.
    ///
    /// Returns the bit's previous state.
    #[inline]
    #[allow(clippy::cast_ptr_alignment)]
    #[cfg(feature = "raw")]
    pub unsafe fn flip(&mut self, index: usize) -> bool {
        // NOTE: The + BITMASK_STRIDE - 1 is to set the high bit.
        let mask = 1 << (index * BITMASK_STRIDE + BITMASK_STRIDE - 1);
        self.0 ^= mask;
        // The bit was set if the bit is now 0.
        self.0 & mask == 0
    }

    // 添加注释: 返回删除了最低位的新`BitMask`
    /// Returns a new `BitMask` with the lowest bit removed.
    #[inline]
    #[must_use]
    pub fn remove_lowest_bit(self) -> Self {
        // 添加注释: 取出self.0 和 self.0 - 1 进行&位运算
        BitMask(self.0 & (self.0 - 1))
    }

    // 添加注释: 返回`BitMask`是否至少有一个设置位
    /// Returns whether the `BitMask` has at least one set bit.
    #[inline]
    pub fn any_bit_set(self) -> bool {
        self.0 != 0
    }

    // 添加注释: 如果存在, 将返回BitMask中的第一个置位
    /// Returns the first set bit in the `BitMask`, if there is one.
    #[inline]
    pub fn lowest_set_bit(self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            Some(unsafe { self.lowest_set_bit_nonzero() })
        }
    }

    // 添加注释: 如果存在, 返回BitMask中的第一个置位. 位掩码不能为空
    /// Returns the first set bit in the `BitMask`, if there is one. The
    /// bitmask must not be empty.
    #[inline]
    #[cfg(feature = "nightly")]
    pub unsafe fn lowest_set_bit_nonzero(self) -> usize {
        // 添加注释: `intrinsics::cttz_nonzero`函数将会返回从右起直至遇到1时的0的个数
        // 添加注释: 传入`0b0011_1100_u8`时, 将会返回2, 因为从右起直至遇到1时的0的个数为2
        // 添加注释: 获取`self.0`变量从右起直至遇到1时的0的数量除以1
        intrinsics::cttz_nonzero(self.0) as usize / BITMASK_STRIDE
    }
    #[inline]
    #[cfg(not(feature = "nightly"))]
    pub unsafe fn lowest_set_bit_nonzero(self) -> usize {
        self.trailing_zeros()
    }

    /// Returns the number of trailing zeroes in the `BitMask`.
    #[inline]
    pub fn trailing_zeros(self) -> usize {
        // ARM doesn't have a trailing_zeroes instruction, and instead uses
        // reverse_bits (RBIT) + leading_zeroes (CLZ). However older ARM
        // versions (pre-ARMv7) don't have RBIT and need to emulate it
        // instead. Since we only have 1 bit set in each byte on ARM, we can
        // use swap_bytes (REV) + leading_zeroes instead.
        if cfg!(target_arch = "arm") && BITMASK_STRIDE % 8 == 0 {
            self.0.swap_bytes().leading_zeros() as usize / BITMASK_STRIDE
        } else {
            self.0.trailing_zeros() as usize / BITMASK_STRIDE
        }
    }

    /// Returns the number of leading zeroes in the `BitMask`.
    #[inline]
    pub fn leading_zeros(self) -> usize {
        self.0.leading_zeros() as usize / BITMASK_STRIDE
    }
}

impl IntoIterator for BitMask {
    type Item = usize;
    type IntoIter = BitMaskIter;

    #[inline]
    fn into_iter(self) -> BitMaskIter {
        BitMaskIter(self)
    }
}

// 添加注释: 对BitMask的内容进行迭代, 返回设置位的索引
/// Iterator over the contents of a `BitMask`, returning the indicies of set
/// bits.
pub struct BitMaskIter(BitMask);

impl Iterator for BitMaskIter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        // ---------
        // 0111 1111
        // 0/1=0                 ; lowest_set_bit
        // 0111 1111
        // &
        // 0111 1110 = 0111 1110 ; remove_lowest_bit
        //
        // ---------
        // 0111 1110
        // 1/1=1
        // 0111 1110
        // &
        // 0111 1101 = 0111 1100
        //
        // ---------
        // 0111 1100
        // 2/1=2
        // 0111 1100
        // &
        // 0111 1011 = 0111 1000

        let bit = self.0.lowest_set_bit()?;
        self.0 = self.0.remove_lowest_bit();
        Some(bit)
    }
}
