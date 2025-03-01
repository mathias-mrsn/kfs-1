/// Align `value` up to the next multiple of `page_size`.
///
/// # Panics
///
/// Panics if `page_size` is not a power of two.
fn align_up(
    value: usize,
    page_size: usize,
) -> usize
{
    assert!(
        page_size.is_power_of_two(),
        "page_size must be a power of two"
    );
    (value + page_size - 1) & !(page_size - 1)
}

/// Align `value` down to the previous multiple of `page_size`.
///
/// # Panics
///
/// Panics if `page_size` is not a power of two.
fn align_down(
    value: usize,
    page_size: usize,
) -> usize
{
    assert!(
        page_size.is_power_of_two(),
        "page_size must be a power of two"
    );
    value & !(page_size - 1)
}
