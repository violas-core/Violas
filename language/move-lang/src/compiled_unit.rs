// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    expansion::ast::{ModuleIdent, ModuleIdent_, SpecId},
    hlir::ast as H,
    parser::ast::{FunctionName, ModuleName, Var},
    shared::{unique_map::UniqueMap, AddressBytes, Name},
};
use bytecode_source_map::source_map::SourceMap;
use move_binary_format::file_format as F;
use move_ir_types::location::*;
use std::collections::BTreeMap;

//**************************************************************************************************
// Compiled Unit
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub type_: H::SingleType,
    pub index: F::LocalIndex,
}

#[derive(Debug, Clone)]
pub struct SpecInfo {
    pub offset: F::CodeOffset,
    // Free locals that are used but not declared in the block
    pub used_locals: UniqueMap<Var, VarInfo>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub spec_info: BTreeMap<SpecId, SpecInfo>,
    pub parameters: Vec<(Var, VarInfo)>,
}

#[derive(Debug, Clone)]
pub struct CompiledModuleIdent {
    pub loc: Loc,
    pub address_name: Option<Name>,
    pub address_bytes: AddressBytes,
    pub module_name: ModuleName,
}

#[derive(Debug, Clone)]
pub enum CompiledUnit {
    Module {
        ident: CompiledModuleIdent,
        module: F::CompiledModule,
        source_map: SourceMap<Loc>,
        function_infos: UniqueMap<FunctionName, FunctionInfo>,
    },
    Script {
        loc: Loc,
        key: String,
        script: F::CompiledScript,
        source_map: SourceMap<Loc>,
        function_info: FunctionInfo,
    },
}

impl CompiledModuleIdent {
    pub fn new(
        loc: Loc,
        address_name: Option<Name>,
        address_bytes: AddressBytes,
        module_name: ModuleName,
    ) -> Self {
        Self {
            loc,
            address_name,
            address_bytes,
            module_name,
        }
    }

    pub fn into_module_ident(self) -> ModuleIdent {
        use crate::expansion::ast::Address;

        let Self {
            loc,
            address_name,
            address_bytes,
            module_name,
        } = self;
        let address = match address_name {
            None => Address::Anonymous(sp(loc, address_bytes)),
            Some(n) => Address::Named(n),
        };
        sp(loc, ModuleIdent_::new(address, module_name))
    }
}

impl CompiledUnit {
    pub fn name(&self) -> String {
        match self {
            CompiledUnit::Module { ident, .. } => ident.module_name.0.value.to_owned(),
            CompiledUnit::Script { key, .. } => key.to_owned(),
        }
    }

    pub fn loc(&self) -> &Loc {
        match self {
            CompiledUnit::Module { ident, .. } => &ident.loc,
            CompiledUnit::Script { loc, .. } => loc,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::<u8>::new();
        match self {
            CompiledUnit::Module { module, .. } => module.serialize(&mut serialized).unwrap(),
            CompiledUnit::Script { script, .. } => script.serialize(&mut serialized).unwrap(),
        };
        serialized
    }

    #[allow(dead_code)]
    pub fn serialize_debug(self) -> Vec<u8> {
        match self {
            CompiledUnit::Module { module, .. } => format!("{:?}", module),
            CompiledUnit::Script { script, .. } => format!("{:?}", script),
        }
        .into()
    }

    pub fn serialize_source_map(&self) -> Vec<u8> {
        match self {
            CompiledUnit::Module { source_map, .. } => bcs::to_bytes(source_map).unwrap(),
            CompiledUnit::Script { source_map, .. } => bcs::to_bytes(source_map).unwrap(),
        }
    }

    pub fn verify(self) -> (Self, Errors) {
        match self {
            CompiledUnit::Module {
                ident,
                module,
                source_map,
                function_infos,
            } => {
                let (module, errors) = verify_module(ident.loc, module);
                let verified = CompiledUnit::Module {
                    ident,
                    module,
                    source_map,
                    function_infos,
                };
                (verified, errors)
            }
            CompiledUnit::Script {
                loc,
                key,
                script,
                source_map,
                function_info,
            } => {
                let (script, errors) = verify_script(loc, script);
                let verified = CompiledUnit::Script {
                    loc,
                    key,
                    script,
                    source_map,
                    function_info,
                };
                (verified, errors)
            }
        }
    }
}

fn verify_module(loc: Loc, cm: F::CompiledModule) -> (F::CompiledModule, Errors) {
    match move_bytecode_verifier::verifier::verify_module(&cm) {
        Ok(_) => (cm, vec![]),
        Err(e) => (
            cm,
            vec![vec![(
                loc,
                format!("ICE failed bytecode verifier: {:#?}", e),
            )]],
        ),
    }
}

fn verify_script(loc: Loc, cs: F::CompiledScript) -> (F::CompiledScript, Errors) {
    match move_bytecode_verifier::verifier::verify_script(&cs) {
        Ok(_) => (cs, vec![]),
        Err(e) => (
            cs,
            vec![vec![(
                loc,
                format!("ICE failed bytecode verifier: {:#?}", e),
            )]],
        ),
    }
}

pub fn verify_units(units: Vec<CompiledUnit>) -> (Vec<CompiledUnit>, Errors) {
    let mut new_units = vec![];
    let mut errors = vec![];
    for unit in units {
        let (unit, es) = unit.verify();
        new_units.push(unit);
        errors.extend(es);
    }
    (new_units, errors)
}
