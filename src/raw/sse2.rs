use super::bitmask::BitMask;
use super::EMPTY;
use core::mem;

#[cfg(target_arch = "x86")]
use core::arch::x86;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as x86;

pub type BitMaskWord = u16;
pub const BITMASK_STRIDE: usize = 1;
pub const BITMASK_MASK: BitMaskWord = 0xffff;

/// Abstraction over a group of control bytes which can be scanned in
/// parallel.
///
/// This implementation uses a 128-bit SSE value.
#[derive(Copy, Clone)]
pub struct Group(x86::__m128i);

// FIXME: https://github.com/rust-lang/rust-clippy/issues/3859
#[allow(clippy::use_self)]
impl Group {
    /// Number of bytes in the group.
    pub const WIDTH: usize = mem::size_of::<Self>();

    // 添加注释: 返回完整的一组空字节, 适合用作空哈希表的初始值
    /// Returns a full group of empty bytes, suitable for use as the initial
    /// value for an empty hash table.
    ///
    // 添加注释: 确保与Group大小保持一致
    /// This is guaranteed to be aligned to the group size.
    #[allow(clippy::items_after_statements)]
    pub const fn static_empty() -> &'static [u8; Group::WIDTH] {
        #[repr(C)]
        struct AlignedBytes {
            _align: [Group; 0],
            bytes: [u8; Group::WIDTH],
        };
        const ALIGNED_BYTES: AlignedBytes = AlignedBytes {
            _align: [],
            bytes: [EMPTY; Group::WIDTH],
        };
        &ALIGNED_BYTES.bytes
    }

    // 添加注释: 从给定地址开始加载一组字节
    /// Loads a group of bytes starting at the given address.
    #[inline]
    #[allow(clippy::cast_ptr_alignment)] // unaligned load
    pub unsafe fn load(ptr: *const u8) -> Self {
        Group(x86::_mm_loadu_si128(ptr as *const _))
    }

    /// Loads a group of bytes starting at the given address, which must be
    /// aligned to `mem::align_of::<Group>()`.
    #[inline]
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn load_aligned(ptr: *const u8) -> Self {
        // FIXME: use align_offset once it stabilizes
        debug_assert_eq!(ptr as usize & (mem::align_of::<Self>() - 1), 0);
        // 添加注释: 一次性读取16个字节进入mmx寄存器
        Group(x86::_mm_load_si128(ptr as *const _))
    }

    /// Stores the group of bytes to the given address, which must be
    /// aligned to `mem::align_of::<Group>()`.
    #[inline]
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn store_aligned(self, ptr: *mut u8) {
        // FIXME: use align_offset once it stabilizes
        debug_assert_eq!(ptr as usize & (mem::align_of::<Self>() - 1), 0);
        x86::_mm_store_si128(ptr as *mut _, self.0);
    }

    // 添加注释: 返回一个BitMask, 指示组中具有给定值的所有字节
    /// Returns a `BitMask` indicating all bytes in the group which have
    /// the given value.
    #[inline]
    pub fn match_byte(self, byte: u8) -> BitMask {
        #[allow(
            clippy::cast_possible_wrap, // byte: u8 as i8
            // byte: i32 as u16
            //   note: _mm_movemask_epi8 returns a 16-bit mask in a i32, the
            //   upper 16-bits of the i32 are zeroed:
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        unsafe {
            // 添加注释: 比较a和b中的压缩8位整数是否相等

            // __m128i _mm_cmpeq_epi8 (__m128i a, __m128i b)
            // Operation
            // FOR j := 0 to 15
            // 	i := j*8
            // 	dst[i+7:i] := ( a[i+7:i] == b[i+7:i] ) ? 0xFF : 0
            // ENDFOR
            let cmp = x86::_mm_cmpeq_epi8(self.0, x86::_mm_set1_epi8(byte as i8));
            // 添加注释: 返回a中每个元素的最高有效位的掩码

            // int _mm_movemask_epi8 (__m128i a)
            // Operation
            // FOR j := 0 to 15
            // 	i := j*8
            // 	dst[j] := a[i+7]
            // ENDFOR
            // dst[MAX:16] := 0
            BitMask(x86::_mm_movemask_epi8(cmp) as u16)
        }
    }

    // 添加注释: 返回一个`BitMask`, 指示组中所有为`EMPTY`的字节
    /// Returns a `BitMask` indicating all bytes in the group which are
    /// `EMPTY`.
    #[inline]
    pub fn match_empty(self) -> BitMask {
        self.match_byte(EMPTY)
    }

    // 添加注释: 返回一个`BitMask`, 指示组中的所有字节为`EMPTY`或`DELETED`
    /// Returns a `BitMask` indicating all bytes in the group which are
    /// `EMPTY` or `DELETED`.
    #[inline]
    pub fn match_empty_or_deleted(self) -> BitMask {
        // 添加注释: `_mm_movemask_epi8`在i32中返回一个16位掩码, 该i32的高16位被清零
        #[allow(
            // byte: i32 as u16
            //   note: _mm_movemask_epi8 returns a 16-bit mask in a i32, the
            //   upper 16-bits of the i32 are zeroed:
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        unsafe {
            // 添加注释: `_mm_movemask_epi8`函数将返回传入变量中每个元素的最高有效位掩码
            // 添加注释: x86::_mm_movemask_epi8会将传入的16字节的数据先转为16个1位的,
            // 添加注释: `pmovmskb`会将mm寄存器中组合的字节整型数的最高比特位按顺序保存到指定的寄存器中
            //          rax[0] <== XMM[7], rax[1] <== XMM[15]
            // A byte is EMPTY or DELETED iff the high bit is set
            BitMask(x86::_mm_movemask_epi8(self.0) as u16)
        }
    }

    // 添加注释: 返回一个`BitMask`, 指示Group中所有已满的字节
    /// Returns a `BitMask` indicating all bytes in the group which are full.
    #[inline]
    pub fn match_full(&self) -> BitMask {
        self.match_empty_or_deleted().invert()
    }

    /// Performs the following transformation on all bytes in the group:
    /// - `EMPTY => EMPTY`
    /// - `DELETED => EMPTY`
    /// - `FULL => DELETED`
    #[inline]
    pub fn convert_special_to_empty_and_full_to_deleted(self) -> Self {
        // Map high_bit = 1 (EMPTY or DELETED) to 1111_1111
        // and high_bit = 0 (FULL) to 1000_0000
        //
        // Here's this logic expanded to concrete values:
        //   let special = 0 > byte = 1111_1111 (true) or 0000_0000 (false)
        //   1111_1111 | 1000_0000 = 1111_1111
        //   0000_0000 | 1000_0000 = 1000_0000
        #[allow(
            clippy::cast_possible_wrap, // byte: 0x80_u8 as i8
        )]
        unsafe {
            let zero = x86::_mm_setzero_si128();
            let special = x86::_mm_cmpgt_epi8(zero, self.0);
            Group(x86::_mm_or_si128(
                special,
                x86::_mm_set1_epi8(0x80_u8 as i8),
            ))
        }
    }
}
