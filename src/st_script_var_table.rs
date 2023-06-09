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
use crate::err::convert_packing_err;
use crate::python::*;
use packed_struct::prelude::*;
use std::ffi::CString;

pub const COUNT_GLOBAL_VARS: u32 = 115;
pub const COUNT_LOCAL_VARS: u32 = 4;
pub const DEFINITION_STRUCT_SIZE: u32 = 16;

#[derive(PrimitiveEnum_u16, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u16))]
pub enum ScriptVariableType {
    None = 0,
    Bit = 1,
    String = 2,
    U8 = 3,
    I8 = 4,
    U16 = 5,
    I16 = 6,
    U32 = 7,
    I32 = 8,
    Special = 9,
}

#[derive(Clone, PackedStruct, Debug)]
#[packed_struct(endian = "lsb")]
pub struct ScriptVariableDefinitionData {
    #[packed_field(size_bytes = "2", ty = "enum")]
    pub r#type: ScriptVariableType,
    pub unk1: u16,
    pub memoffset: u16,
    pub bitshift: u16,
    pub nbvalues: u16,
    pub default: i16,
    name_ptr: u32, // char*
}

#[pyclass(module = "skytemple_rust.st_script_var_table")]
#[derive(Clone, Debug)]
pub struct ScriptVariableDefinition {
    #[pyo3(get)]
    pub id: usize,
    pub data: ScriptVariableDefinitionData,
    #[pyo3(get)]
    pub name: String,
}

#[pymethods]
impl ScriptVariableDefinition {
    // <editor-fold desc="Proxy getters for ScriptVariableDefinitionData" defaultstate="collapsed">
    #[getter]
    #[cfg(feature = "python")]
    pub fn r#type(&self) -> ScriptVariableType {
        self.data.r#type
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk1(&self) -> u16 {
        self.data.unk1
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn memoffset(&self) -> u16 {
        self.data.memoffset
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn bitshift(&self) -> u16 {
        self.data.bitshift
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn nbvalues(&self) -> u16 {
        self.data.nbvalues
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn default(&self) -> i16 {
        self.data.default
    }
    // </editor-fold>

    pub fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

impl ScriptVariableDefinition {
    fn new<F>(id: usize, mem: &[u8], name_reader: F) -> PyResult<Self>
    where
        F: Fn(u32) -> Result<CString, ()>,
    {
        let data = ScriptVariableDefinitionData::unpack(mem.try_into().unwrap())
            .map_err(convert_packing_err)?;
        Ok(Self {
            id,
            name: name_reader(data.name_ptr)
                .map_err(|_| {
                    exceptions::PyValueError::new_err(
                        "Failed reading game variable name as string.".to_string(),
                    )
                })?
                .to_string_lossy()
                .to_string(),
            data,
        })
    }
}

#[pyclass(module = "skytemple_rust.st_script_var_table")]
#[derive(Clone, Debug)]
pub struct ScriptVariableTables {
    #[pyo3(get)]
    pub globals: Vec<ScriptVariableDefinition>,
    #[pyo3(get)]
    pub locals: Vec<ScriptVariableDefinition>,
}

#[pymethods]
impl ScriptVariableTables {
    #[new]
    pub fn new(
        mem: StBytes,
        global_start: usize,
        local_start: usize,
        subtract_from_name_addrs: u32,
    ) -> PyResult<Self> {
        static_assert_size!(
            <ScriptVariableDefinitionData as PackedStruct>::ByteArray,
            DEFINITION_STRUCT_SIZE as usize
        );

        let load_name = |addr: u32| {
            let slice = &mem.as_ref()[((addr - subtract_from_name_addrs) as usize)..];
            let nul_range_end = slice
                .iter()
                .position(|&c| c == b'\0')
                .unwrap_or(slice.len());
            CString::new(&slice[0..nul_range_end]).map_err(|_| ())
        };

        Ok(Self {
            globals: mem.as_ref()[global_start
                ..(global_start + (COUNT_GLOBAL_VARS * DEFINITION_STRUCT_SIZE) as usize)]
                .chunks(DEFINITION_STRUCT_SIZE as usize)
                .enumerate()
                .map(|(i, gmem)| ScriptVariableDefinition::new(i, gmem, load_name))
                .collect::<PyResult<Vec<_>>>()?,
            locals: mem.as_ref()
                [local_start..(local_start + (COUNT_LOCAL_VARS * DEFINITION_STRUCT_SIZE) as usize)]
                .chunks(DEFINITION_STRUCT_SIZE as usize)
                .enumerate()
                .map(|(i, lmem)| ScriptVariableDefinition::new(i + 0x400, lmem, load_name))
                .collect::<PyResult<Vec<_>>>()?,
        })
    }
}

impl ScriptVariableTables {
    pub fn new_with_name_reader<F>(
        global_mem: StBytes,
        local_mem: StBytes,
        name_reader: &F,
    ) -> PyResult<Self>
    where
        F: Fn(u32) -> Result<CString, ()>,
    {
        Ok(Self {
            globals: global_mem
                .as_ref()
                .chunks(DEFINITION_STRUCT_SIZE as usize)
                .enumerate()
                .map(|(i, mem)| ScriptVariableDefinition::new(i, mem, name_reader))
                .collect::<PyResult<Vec<_>>>()?,
            locals: local_mem
                .as_ref()
                .chunks(DEFINITION_STRUCT_SIZE as usize)
                .enumerate()
                .map(|(i, mem)| ScriptVariableDefinition::new(i + 0x400, mem, name_reader))
                .collect::<PyResult<Vec<_>>>()?,
        })
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_script_var_table_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_script_var_table";
    let m = PyModule::new(py, name)?;
    m.add_class::<ScriptVariableDefinition>()?;
    m.add_class::<ScriptVariableTables>()?;
    m.add("COUNT_GLOBAL_VARS", COUNT_GLOBAL_VARS)?;
    m.add("COUNT_LOCAL_VARS", COUNT_LOCAL_VARS)?;
    m.add("DEFINITION_STRUCT_SIZE", DEFINITION_STRUCT_SIZE)?;

    Ok((name, m))
}
