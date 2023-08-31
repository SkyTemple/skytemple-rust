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
use crate::bytes::{AsStBytes, StBytes};
use crate::python::PyErr;
use crate::PyResult;
use bytes::BytesMut;
use std::cmp::{max, min};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::hint::unreachable_unchecked;
use std::iter::repeat;

#[inline]
#[allow(unused)]
pub(crate) fn slice_to_array<const N: usize>(slice: &[u8]) -> [u8; N] {
    let mut arr: [u8; N] = [0; N];
    arr.copy_from_slice(slice);
    arr
}

#[allow(unused)]
pub(crate) fn init_default_vec<U, T>(size: usize) -> U
where
    U: FromIterator<T>,
    T: Default,
{
    (0..size).map(|_| Default::default()).collect()
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
    a * b / gcd(a, b)
}

/// Smart pointer to lazily build data from StBytes.
/// Can fail converting from StBytes/T and into T.
pub enum Lazy<T> {
    Source(StBytes),
    Instance(T),
}

impl<T> Clone for Lazy<T>
where
    T: AsStBytes + TryFrom<StBytes> + Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Source(v) => Self::Source(v.clone()),
            Self::Instance(v) => Self::Instance(v.clone()),
        }
    }
}

impl<T> Debug for Lazy<T>
where
    T: AsStBytes + TryFrom<StBytes> + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(v) => write!(f, "Lazy::Source({:?})", v),
            Self::Instance(v) => write!(f, "Lazy::Instance({:?})", v),
        }
    }
}

impl<T, E> PartialEq for Lazy<T>
where
    T: AsStBytes + TryFrom<StBytes, Error = E>,
    E: Into<PyErr>,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<T, E> Eq for Lazy<T>
where
    T: AsStBytes + TryFrom<StBytes, Error = E>,
    E: Into<PyErr>,
{
}

impl<T, E> Hash for Lazy<T>
where
    T: AsStBytes + TryFrom<StBytes, Error = E> + Hash,
    E: Into<PyErr>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state)
    }
}

impl<T, E> Lazy<T>
where
    T: AsStBytes + TryFrom<StBytes, Error = E>,
    E: Into<PyErr>,
{
    /// Creates a Lazy wrapper from some byte source but immediately constructs it.
    /// This is useful, if the source has an unbound length.
    pub fn instance_from_source(source: StBytes) -> PyResult<Self> {
        Ok(Self::Instance(source.try_into().map_err(Into::into)?))
    }

    pub fn instance(&mut self) -> PyResult<&T> {
        self.instance_mut().map(|v| &*v)
    }

    pub fn instance_mut(&mut self) -> PyResult<&mut T> {
        Ok(match self {
            Lazy::Source(v) => {
                *self = Self::Instance(T::try_from(v.clone()).map_err(Into::into)?);
                match self {
                    Lazy::Instance(v) => v,
                    Lazy::Source(_) => unsafe { unreachable_unchecked() },
                }
            }
            Lazy::Instance(v) => v,
        })
    }
}

impl<T> From<StBytes> for Lazy<T>
where
    T: AsStBytes + TryFrom<StBytes>,
{
    fn from(value: StBytes) -> Self {
        Self::Source(value)
    }
}

impl<T, E> From<Lazy<T>> for StBytes
where
    T: AsStBytes + TryFrom<StBytes, Error = E>,
    E: Into<PyErr>,
{
    fn from(source: Lazy<T>) -> Self {
        source.as_bytes()
    }
}

impl<T, E> AsStBytes for Lazy<T>
where
    T: AsStBytes + TryFrom<StBytes, Error = E>,
    E: Into<PyErr>,
{
    fn as_bytes(&self) -> StBytes {
        match self {
            Lazy::Source(s) => s.clone(),
            Lazy::Instance(i) => i.as_bytes(),
        }
    }
}

#[allow(dead_code)]
pub fn pad(data: &mut BytesMut, padlen: usize, padwith: u8) {
    let lenp = data.len() % padlen;
    if lenp != 0 {
        data.extend(repeat(padwith).take(padlen - lenp))
    }
}
