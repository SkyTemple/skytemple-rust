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
use syn::DeriveInput;
use syn::Ident;

/// Derive conversion from/to Python integers for PrimitiveEnums.
/// Only works if packed_struct and pyo3 are available.
#[proc_macro_derive(EnumToPy_u16)]
pub fn enum_to_py_derive_u16(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.ident.span();
    do_enum_to_py_derive(input, Ident::new("u16", span))
}

/// Derive conversion from/to Python integers for PrimitiveEnums.
/// Only works if packed_struct and pyo3 are available.
#[proc_macro_derive(EnumToPy_u8)]
pub fn enum_to_py_derive_u8(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.ident.span();
    do_enum_to_py_derive(input, Ident::new("u8", span))
}

/// Derive conversion from/to Python integers for PrimitiveEnums.
/// Only works if packed_struct and pyo3 are available.
#[proc_macro_derive(EnumToPy_i8)]
pub fn enum_to_py_derive_i8(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.ident.span();
    do_enum_to_py_derive(input, Ident::new("i8", span))
}

fn do_enum_to_py_derive(input: DeriveInput, bytesize: Ident) -> TokenStream {
    let ident = &input.ident;
    let expanded = quote! {
        impl<'py> ::pyo3::prelude::IntoPyObject<'py> for #ident {
            type Target = <#bytesize as ::pyo3::prelude::IntoPyObject<'py>>::Target;
            type Output = <#bytesize as ::pyo3::prelude::IntoPyObject<'py>>::Output;
            type Error = <#bytesize as ::pyo3::prelude::IntoPyObject<'py>>::Error;

            fn into_pyobject(self, py: ::pyo3::prelude::Python<'py>) -> Result<Self::Output, Self::Error> {
                packed_struct::PrimitiveEnum::to_primitive(&self).into_pyobject(py)
            }
        }

        impl<'source> ::pyo3::prelude::FromPyObject<'source> for #ident
        {
            fn extract_bound(ob: &::pyo3::prelude::Bound<'source, ::pyo3::prelude::PyAny>) -> ::pyo3::prelude::PyResult<Self> {
                if let Ok(obj) = ob.extract::<#bytesize>() {
                    <Self as packed_struct::PrimitiveEnum>::from_primitive(obj).ok_or_else(
                        || ::pyo3::exceptions::PyTypeError::new_err(
                            "Invalid value to convert into enum.",
                        )
                    )
                } else {
                    Err(::pyo3::exceptions::PyTypeError::new_err(
                        "Invalid type to convert into enum.",
                    ))
                }
            }
        }
    };

    TokenStream::from(expanded)
}
