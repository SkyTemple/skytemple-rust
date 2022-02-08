/*
 * Copyright 2021-2022 Capypara and the SkyTemple Contributors
 *
 * This file is part of SkyTemple.
 *
 * SkyTemple is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * SkyTemple is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with SkyTemple.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::cmp::{max, min};

#[inline]
#[allow(unused)]
pub(crate)  fn slice_to_array<const N: usize>(slice: &[u8]) -> [u8; N] {
    let mut arr: [u8; N] = [0; N];
    arr.copy_from_slice(slice);
    arr
}

#[allow(unused)]
pub(crate)  fn init_default_vec<U, T>(size: usize) -> U where U: FromIterator<T>, T: Default {
    (0..size).into_iter().map(|_| Default::default()).collect()
}

#[allow(unused)]
pub(crate) fn gcd(a: usize, b: usize) -> usize {
    match ((a, b), (a & 1, b & 1)) {
        ((x, y), _) if x == y => y,
        ((0, x), _) | ((x, 0), _) => x,
        ((x, y), (0, 1)) | ((y, x), (1, 0)) => gcd(x >> 1, y),
        ((x, y), (0, 0)) => gcd(x >> 1, y >> 1) << 1,
        ((x, y), (1, 1)) => {
            let (x, y) = (min(x, y), max(x, y));
            gcd((y - x) >> 1, x)
        }
        _ => unreachable!(),
    }
}

#[allow(unused)]
pub(crate) fn lcm(a: usize, b: usize) -> usize {
    a * b/gcd(a, b)
}
