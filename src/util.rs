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

#[inline]
pub fn slice_to_array<const N: usize>(slice: &[u8]) -> [u8; N] {
    let mut arr: [u8; N] = [0; N];
    arr.copy_from_slice(slice);
    arr
}

pub fn init_default_vec<T>(size: usize) -> Vec<T> where T: Default {
    (0..size).into_iter().map(|_| Default::default()).collect()
}
