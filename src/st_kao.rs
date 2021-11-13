/*
 * Copyright 2021-2021 Parakoopa and the SkyTemple Contributors
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

#[pyclass(module = "st_kao")]
#[derive(Clone)]
pub struct Dummy {
    test: String
}

#[pymethods]
impl Dummy {
    #[new]
    pub fn new(value: u8) -> PyResult<Self> {
        match value {
            0 => Ok(Dummy {test: "1".to_string() }),
            1 => Ok(Dummy {test: "2".to_string() }),
            3 => Ok(Dummy {test: "3".to_string() }),
            _ => Err(exceptions::PyValueError::new_err("no"))
        }
    }
    pub fn get_test(&self) -> PyResult<&str> {
        Ok(self.test.as_str())
    }
    pub fn set_test(&mut self, value: &str) {
        self.test = value.to_string();
    }
}

#[cfg(not(feature = "no-python"))]
#[pymodule]
fn st_kao(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Dummy>()?;
    Ok(())
}
