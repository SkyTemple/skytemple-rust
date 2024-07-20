/*
 * Copyright 2021-2024 Capypara and the SkyTemple Contributors
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
use std::fmt::{Debug, Formatter};
use std::hint::unreachable_unchecked;
use std::iter::repeat;

use bytes::BytesMut;
use pyo3::{Py, PyErr, PyResult, Python};

use crate::bytes::{AsStBytesPyRef, StBytes};

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

impl<T> Debug for Lazy<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(v) => write!(f, "Lazy::Source({:?})", v),
            Self::Instance(v) => write!(f, "Lazy::Instance({:?})", v),
        }
    }
}

impl<T> Lazy<T>
where
    T: AsStBytesPyRef,
{
    pub fn as_bytes(&self, py: Python) -> StBytes {
        match self {
            Lazy::Source(s) => s.clone(),
            Lazy::Instance(i) => i.as_bytes_pyref(py),
        }
    }

    pub fn eq_pyref(&self, other: &Self, py: Python) -> bool {
        self.as_bytes(py) == other.as_bytes(py)
    }
}

impl<T, E> Lazy<T>
where
    T: AsStBytesPyRef + TryFrom<StBytes, Error = E>,
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
    T: AsStBytesPyRef,
{
    fn from(value: StBytes) -> Self {
        Self::Source(value)
    }
}

impl<T> AsStBytesPyRef for Lazy<T>
where
    T: AsStBytesPyRef,
{
    fn as_bytes_pyref(&self, py: Python) -> StBytes {
        match self {
            Lazy::Source(v) => v.clone(),
            Lazy::Instance(v) => v.as_bytes_pyref(py),
        }
    }
}

impl<T> Lazy<Py<T>>
where
    Self: Into<StBytes>,
{
    pub fn clone_ref(&self, py: Python) -> Self {
        match self {
            Self::Source(v) => Self::Source(v.clone()),
            Self::Instance(v) => Self::Instance(v.clone_ref(py)),
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
