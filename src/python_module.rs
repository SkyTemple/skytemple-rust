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
use log::info;
use pyo3::types::PyDict;

#[cfg(feature = "compression")]
use crate::compression::bma_collision_rle::create_st_bma_collision_rle_compression_module;
#[cfg(feature = "compression")]
use crate::compression::bma_layer_nrl::create_st_bma_layer_nrl_compression_module;
#[cfg(feature = "compression")]
use crate::compression::bpc_image::create_st_bpc_image_compression_module;
#[cfg(feature = "compression")]
use crate::compression::bpc_tilemap::create_st_bpc_tilemap_compression_module;
#[cfg(feature = "compression")]
use crate::compression::generic::nrl::create_st_generic_nrl_compression_module;
#[cfg(feature = "dse")]
use crate::dse::st_smdl::python::create_st_smdl_module;
#[cfg(feature = "dse")]
use crate::dse::st_swdl::python::create_st_swdl_module;
#[cfg(feature = "image")]
use crate::image::tilemap_entry::TilemapEntry;
#[cfg(feature = "with_pmd_wan")]
use crate::pmd_wan::create_pmd_wan_module;
#[cfg(feature = "compression")]
use crate::st_at3px::create_st_at3px_module;
#[cfg(feature = "compression")]
use crate::st_at4pn::create_st_at4pn_module;
#[cfg(feature = "compression")]
use crate::st_at4px::create_st_at4px_module;
#[cfg(feature = "compression")]
use crate::st_at_common::create_st_at_common_module;
#[cfg(feature = "compression")]
use crate::st_atupx::create_st_atupx_module;
#[cfg(feature = "map_bg")]
use crate::st_bg_list_dat::create_st_bg_list_dat_module;
#[cfg(feature = "misc_graphics")]
use crate::st_bgp::create_st_bgp_module;
#[cfg(feature = "map_bg")]
use crate::st_bma::create_st_bma_module;
#[cfg(feature = "map_bg")]
use crate::st_bpa::create_st_bpa_module;
#[cfg(feature = "map_bg")]
use crate::st_bpc::create_st_bpc_module;
#[cfg(feature = "map_bg")]
use crate::st_bpl::create_st_bpl_module;
#[cfg(feature = "dungeon_graphics")]
use crate::st_dbg::create_st_dbg_module;
#[cfg(feature = "dungeon_graphics")]
use crate::st_dma::create_st_dma_module;
#[cfg(feature = "dungeon_graphics")]
use crate::st_dpc::create_st_dpc_module;
#[cfg(feature = "dungeon_graphics")]
use crate::st_dpci::create_st_dpci_module;
#[cfg(feature = "dungeon_graphics")]
use crate::st_dpl::create_st_dpl_module;
#[cfg(feature = "dungeon_graphics")]
use crate::st_dpla::create_st_dpla_module;
#[cfg(feature = "item_p")]
use crate::st_item_p::create_st_item_p_module;
#[cfg(feature = "kao")]
use crate::st_kao::create_st_kao_module;
#[cfg(feature = "mappa_bin")]
use crate::st_mappa_bin::create_st_mappa_bin_module;
#[cfg(feature = "md")]
use crate::st_md::create_st_md_module;
#[cfg(feature = "compression")]
use crate::st_pkdpx::create_st_pkdpx_module;
#[cfg(feature = "script_var_table")]
use crate::st_script_var_table::create_st_script_var_table_module;
#[cfg(feature = "sir0")]
use crate::st_sir0::create_st_sir0_module;
#[cfg(feature = "strings")]
use crate::st_string::create_st_string_module;
#[cfg(feature = "waza_p")]
use crate::st_waza_p::create_st_waza_p_module;

#[pymodule]
fn skytemple_rust(py: Python, module: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    info!("Loading skytemple_rust...");
    let sys = py.import("sys")?;
    let modules: &PyDict = sys.getattr("modules")?.downcast()?;
    #[cfg(feature = "sir0")]
    add_submodule(module, create_st_sir0_module(py)?, modules)?;
    #[cfg(feature = "with_pmd_wan")]
    add_submodule(module, create_pmd_wan_module(py)?, modules)?;
    #[cfg(feature = "compression")]
    add_submodule(module, create_st_at_common_module(py)?, modules)?;
    #[cfg(feature = "compression")]
    add_submodule(module, create_st_at3px_module(py)?, modules)?;
    #[cfg(feature = "compression")]
    add_submodule(module, create_st_at4pn_module(py)?, modules)?;
    #[cfg(feature = "compression")]
    add_submodule(module, create_st_at4px_module(py)?, modules)?;
    #[cfg(feature = "compression")]
    add_submodule(module, create_st_atupx_module(py)?, modules)?;
    #[cfg(feature = "compression")]
    add_submodule(module, create_st_pkdpx_module(py)?, modules)?;
    #[cfg(feature = "kao")]
    add_submodule(module, create_st_kao_module(py)?, modules)?;
    #[cfg(feature = "map_bg")]
    add_submodule(module, create_st_bg_list_dat_module(py)?, modules)?;
    #[cfg(feature = "misc_graphics")]
    add_submodule(module, create_st_bgp_module(py)?, modules)?;
    #[cfg(feature = "map_bg")]
    add_submodule(module, create_st_bma_module(py)?, modules)?;
    #[cfg(feature = "map_bg")]
    add_submodule(module, create_st_bpa_module(py)?, modules)?;
    #[cfg(feature = "map_bg")]
    add_submodule(module, create_st_bpc_module(py)?, modules)?;
    #[cfg(feature = "map_bg")]
    add_submodule(module, create_st_bpl_module(py)?, modules)?;
    #[cfg(feature = "dungeon_graphics")]
    add_submodule(module, create_st_dbg_module(py)?, modules)?;
    #[cfg(feature = "dungeon_graphics")]
    add_submodule(module, create_st_dma_module(py)?, modules)?;
    #[cfg(feature = "dungeon_graphics")]
    add_submodule(module, create_st_dpc_module(py)?, modules)?;
    #[cfg(feature = "dungeon_graphics")]
    add_submodule(module, create_st_dpci_module(py)?, modules)?;
    #[cfg(feature = "dungeon_graphics")]
    add_submodule(module, create_st_dpl_module(py)?, modules)?;
    #[cfg(feature = "dungeon_graphics")]
    add_submodule(module, create_st_dpla_module(py)?, modules)?;
    #[cfg(feature = "md")]
    add_submodule(module, create_st_md_module(py)?, modules)?;
    #[cfg(feature = "item_p")]
    add_submodule(module, create_st_item_p_module(py)?, modules)?;
    #[cfg(feature = "waza_p")]
    add_submodule(module, create_st_waza_p_module(py)?, modules)?;
    #[cfg(feature = "mappa_bin")]
    add_submodule(module, create_st_mappa_bin_module(py)?, modules)?;
    #[cfg(feature = "dse")]
    add_submodule(module, create_st_smdl_module(py)?, modules)?;
    #[cfg(feature = "dse")]
    add_submodule(module, create_st_swdl_module(py)?, modules)?;
    #[cfg(feature = "strings")]
    add_submodule(module, create_st_string_module(py)?, modules)?;
    #[cfg(feature = "script_var_table")]
    add_submodule(module, create_st_script_var_table_module(py)?, modules)?;

    #[cfg(feature = "compression")]
    add_submodule(
        module,
        create_st_generic_nrl_compression_module(py)?,
        modules,
    )?;
    #[cfg(feature = "compression")]
    add_submodule(module, create_st_bpc_image_compression_module(py)?, modules)?;
    #[cfg(feature = "compression")]
    add_submodule(
        module,
        create_st_bpc_tilemap_compression_module(py)?,
        modules,
    )?;
    #[cfg(feature = "compression")]
    add_submodule(
        module,
        create_st_bma_layer_nrl_compression_module(py)?,
        modules,
    )?;
    #[cfg(feature = "compression")]
    add_submodule(
        module,
        create_st_bma_collision_rle_compression_module(py)?,
        modules,
    )?;

    #[cfg(feature = "image")]
    module.add_class::<TilemapEntry>()?;

    Ok(())
}

#[inline]
fn add_submodule(
    parent: &PyModule,
    (name, module): (&str, &PyModule),
    modules: &PyDict,
) -> PyResult<()> {
    modules.set_item(name, module)?;
    parent.add_submodule(module)?;
    parent.add(&name.split('.').skip(1).collect::<String>(), module)
}
