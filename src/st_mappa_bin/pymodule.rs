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

use crate::python::{PyModule, Python};
use crate::st_mappa_bin::{
    MappaBin, MappaBinWriter, MappaFloor, MappaFloorLayout, MappaFloorTerrainSettings,
    MappaItemList, MappaMonster, MappaMonsterList, MappaMonsterListIterator, MappaTrapList,
};
use crate::PyResult;

#[cfg(feature = "python")]
pub(crate) fn create_st_mappa_bin_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_mappa_bin";
    let m = PyModule::new(py, name)?;
    m.add_class::<MappaTrapList>()?;
    m.add_class::<MappaMonster>()?;
    m.add_class::<MappaMonsterList>()?;
    m.add_class::<MappaMonsterListIterator>()?;
    m.add_class::<MappaItemList>()?;
    m.add_class::<MappaFloorTerrainSettings>()?;
    m.add_class::<MappaFloorLayout>()?;
    m.add_class::<MappaFloor>()?;
    m.add_class::<MappaBin>()?;
    m.add_class::<MappaBinWriter>()?;

    Ok((name, m))
}
