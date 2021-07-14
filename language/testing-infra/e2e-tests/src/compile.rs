// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.

use compiler::Compiler;

use diem_types::{
    account_address::AccountAddress,
    transaction::{Module, Script},
};
use move_binary_format::CompiledModule;

/// Compile the provided Move code into a blob which can be used as the code to be published
/// (a Module).
pub fn compile_module_with_address(
    address: &AccountAddress,
    file_name: &str,
    code: &str,
) -> (CompiledModule, Module) {
    let compiled_module = Compiler {
        address: *address,
        skip_stdlib_deps: false,
        extra_deps: vec![],
    }
    .into_compiled_module(file_name, code)
    .expect("Module compilation failed");
    let module = Module::new(
        Compiler {
            address: *address,
            skip_stdlib_deps: false,
            extra_deps: vec![],
        }
        .into_module_blob(file_name, code)
        .expect("Module compilation failed"),
    );
    (compiled_module, module)
}

/// Compile the provided Move code into a blob which can be used as the code to be executed
/// (a Script).
pub fn compile_script_with_address(
    address: &AccountAddress,
    file_name: &str,
    code: &str,
    extra_deps: Vec<CompiledModule>,
) -> Script {
    let compiler = Compiler {
        address: *address,
        skip_stdlib_deps: false,
        extra_deps,
    };
    Script::new(
        compiler
            .into_script_blob(file_name, code)
            .expect("Script compilation failed"),
        vec![],
        vec![],
    )
}
