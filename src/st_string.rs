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
use crate::bytes::StBytes;
use crate::encoding::pmd2_encoder::Pmd2Encoding;
use crate::err::convert_encoding_err;
use crate::python::*;
use encoding::{DecoderTrap, EncoderTrap, Encoding};

#[pyclass(module = "skytemple_rust.st_string")]
#[derive(Clone)]
pub struct StPmd2String(String);

#[pymethods]
impl StPmd2String {
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        Ok(Self(
            Pmd2Encoding
                .decode(&data[..], DecoderTrap::Strict)
                .map_err(convert_encoding_err)?,
        ))
    }
    fn __str__(&self) -> String {
        self.0.clone()
    }
}

#[pyclass(module = "skytemple_rust.st_string")]
#[derive(Default)]
pub struct StPmd2StringEncoder;

#[pymethods]
impl StPmd2StringEncoder {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<StPmd2String>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);
        Ok(Pmd2Encoding
            .encode(&model.0, EncoderTrap::Strict)
            .map_err(convert_encoding_err)?
            .into())
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_string_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_string";
    let m = PyModule::new(py, name)?;
    m.add_class::<StPmd2String>()?;
    m.add_class::<StPmd2StringEncoder>()?;

    Ok((name, m))
}
