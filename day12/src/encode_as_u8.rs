use bitvec::{order::BitOrder, slice::BitSlice, store::BitStore};

/// It's convenient for this problem to treat some slices of booleans as bytes.
pub trait EncodeAsU8 {
    fn as_u8(&self) -> u8;
}

impl<BoolSlice> EncodeAsU8 for BoolSlice
where
    BoolSlice: AsRef<[bool]>,
{
    fn as_u8(&self) -> u8 {
        let mut out = 0;
        for (idx, bit) in self.as_ref().iter().copied().rev().enumerate().take(8) {
            if bit {
                out |= 1 << idx;
            }
        }
        out
    }
}

impl<O, T> EncodeAsU8 for BitSlice<O, T>
where
    O: BitOrder,
    T: BitStore,
{
    fn as_u8(&self) -> u8 {
        let mut out: u8 = 0;
        for (idx, bit) in self
            .iter()
            .map(|bit_ref| *bit_ref)
            .rev()
            .enumerate()
            .take(8)
        {
            if bit {
                out |= 1 << idx;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;
    use rstest::rstest;

    #[rstest]
    #[case([false, false, false, false, false], 0)]
    #[case([false, false, false, false, true], 1)]
    #[case([false, false, false, true, false], 2)]
    #[case([false, false, false, true, true], 3)]
    #[case([false, false, true, false, false], 4)]
    #[case([false, true, false, false, false], 8)]
    #[case([true, false, false, false, false], 16)]
    #[case([true, false, false, false, true], 17)]
    #[case([true, true, true, true, true], 31)]
    fn bool_array_as_u8(#[case] arr: [bool; 5], #[case] expect: u8) {
        assert_eq!(arr.as_u8(), expect);
    }

    #[rstest]
    #[case(bitvec![0, 0, 0], 0)]
    #[case(bitvec![0, 0, 1], 1)]
    #[case(bitvec![0, 1, 0], 2)]
    #[case(bitvec![0, 1, 1], 3)]
    #[case(bitvec![1, 0, 0], 4)]
    #[case(bitvec![0, 0, 0, 0], 0)]
    #[case(bitvec![0, 0, 0, 1], 1)]
    #[case(bitvec![0, 0, 1, 0], 2)]
    #[case(bitvec![0, 0, 1, 1], 3)]
    #[case(bitvec![0, 1, 0, 0], 4)]
    #[case(bitvec![1, 0, 0, 0], 8)]
    #[case(bitvec![0, 0, 0, 0, 0], 0)]
    #[case(bitvec![0, 0, 0, 0, 1], 1)]
    #[case(bitvec![0, 0, 0, 1, 0], 2)]
    #[case(bitvec![0, 0, 0, 1, 1], 3)]
    #[case(bitvec![0, 0, 1, 0, 0], 4)]
    #[case(bitvec![0, 1, 0, 0, 0], 8)]
    #[case(bitvec![1, 0, 0, 0, 0], 16)]
    #[case(bitvec![1, 0, 0, 0, 1], 17)]
    #[case(bitvec![1, 1, 1, 1, 1], 31)]
    #[case(bitvec![1, 0, 0, 0, 0, 0, 0, 0, 0], 0)]
    fn bit_vec_as_u8(#[case] arr: BitVec, #[case] expect: u8) {
        assert_eq!(arr.as_u8(), expect);
    }
}
