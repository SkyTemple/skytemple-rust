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
use crate::python::*;
#[cfg(feature = "python")]
use pyo3::types::PyTuple;

pub enum RomSource<T: RomFileProvider + Sized> {
    Folder(String),
    Rom(T)
}

#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for RomSource<&'source PyAny> {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(match ob.extract::<String>().ok() {
            Some(x) => Self::Folder(x),
            None => Self::Rom(ob)
        })
    }
}

pub trait RomFileProvider {
    #[allow(non_snake_case)]
    fn getFileByName(&self, filename: &str) -> PyResult<Vec<u8>>;
}

#[cfg(feature = "python")]
impl RomFileProvider for &PyAny {
    #[allow(non_snake_case)]
    fn getFileByName(&self, filename: &str) -> PyResult<Vec<u8>> {
        let args = PyTuple::new(self.py(), [filename]);
        self.call_method1("getFileByName", args)?.extract()
    }
}
