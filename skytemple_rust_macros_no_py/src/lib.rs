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
/** Dummy macros for using skytemple_rust without Pyo3 */
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// pyclass for Python/PyO3-less environments.
#[proc_macro_attribute]
pub fn pyclass(_: TokenStream, item: TokenStream) -> TokenStream {
    // We are removing all #[pyo3] inner attributes.
    let mut class = parse_macro_input!(item as syn::ItemStruct);

    match &mut class.fields {
        syn::Fields::Named(fields) => fields.named.iter_mut().for_each(|field| {
            field.attrs = vec![];
        }),
        syn::Fields::Unnamed(fields) => fields.unnamed.iter_mut().for_each(|field| {
            field.attrs = vec![];
        }),
        syn::Fields::Unit => {}
    };

    quote!(#class).into()
}

/// pymethods for Python/PyO3-less environments.
#[proc_macro_attribute]
pub fn pymethods(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// getter for Python/PyO3-less environments.
#[proc_macro_attribute]
pub fn getter(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// setter for Python/PyO3-less environments.
#[proc_macro_attribute]
pub fn setter(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// classmethod for Python/PyO3-less environments.
#[proc_macro_attribute]
pub fn classmethod(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// new for Python/PyO3-less environments.
#[proc_macro_attribute]
pub fn new(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// pyo3 for Python/PyO3-less environments.
#[proc_macro_attribute]
pub fn pyo3(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}
