pub const fn next_pow2(n: usize) -> usize
{
    if n <= 1 {
        1
    } else {
        (usize::MAX >> (n - 1).leading_zeros()) + 1
    }
}

pub fn find_pow2_bits(n: usize) -> Option<usize>
{
    if n == 0 || (n & (n - 1)) != 0 {
        None
    } else {
        Some(n.trailing_zeros() as usize)
    }
}
