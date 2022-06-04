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
use crate::python::*;

#[pyclass(module = "skytemple_rust.st_dpla")]
#[derive(Clone)]
pub struct Dpla {}

#[pymethods]
impl Dpla {
    #[allow(unused_variables)]
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_dpla_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_dpla";
    let m = PyModule::new(py, name)?;
    m.add_class::<Dpla>()?;

    Ok((name, m))
}
