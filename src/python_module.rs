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
use log::info;
use pyo3::types::PyDict;
use crate::python::*;


use crate::pmd_wan::create_pmd_wan_module;
use crate::st_at3px::create_st_at3px_module;
use crate::st_at4pn::create_st_at4pn_module;
use crate::st_at4px::create_st_at4px_module;
use crate::st_at_common::create_st_at_common_module;
use crate::st_atupx::create_st_atupx_module;
use crate::st_kao::create_st_kao_module;
use crate::st_pkdpx::create_st_pkdpx_module;

#[pymodule]
fn skytemple_rust(py: Python, module: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    info!("Loading skytemple_rust...");
    let sys = py.import("sys")?;
    let modules: &PyDict = sys.getattr("modules")?.extract()?;
    add_submodule(module, create_pmd_wan_module(py)?, modules)?;
    add_submodule(module, create_st_at_common_module(py)?, modules)?;
    add_submodule(module, create_st_at3px_module(py)?, modules)?;
    add_submodule(module, create_st_at4pn_module(py)?, modules)?;
    add_submodule(module, create_st_at4px_module(py)?, modules)?;
    add_submodule(module, create_st_atupx_module(py)?, modules)?;
    add_submodule(module, create_st_pkdpx_module(py)?, modules)?;
    add_submodule(module, create_st_kao_module(py)?, modules)?;

    Ok(())
}

#[inline]
fn add_submodule(parent: &PyModule, (name, module): (&str, &PyModule), modules: &PyDict) -> PyResult<()> {
    modules.set_item(name, module)?;
    parent.add_submodule(module)?;
    parent.add(&name.split('.').into_iter().skip(1).collect::<String>(), module)
}
