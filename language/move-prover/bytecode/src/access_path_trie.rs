// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The obvious approach to abstracting a set of concrete paths is using a set of abstract paths.
//! An access path trie represents a set of paths in a way that avoids redundant representations of
//! the same memory. Root nodes are access path roots and each internal node is an access path offset.
//! Each node is (optionally) associated with abstract value of a generic type `T`.

use crate::{
    access_path::{AbsAddr, AccessPath, AccessPathMap, FootprintDomain, Offset, Root},
    dataflow_domains::{AbstractDomain, JoinResult, MapDomain},
};
use im::ordmap::Entry;
use move_core_types::{account_address::AccountAddress, language_storage::TypeTag};
use move_model::{
    ast::TempIndex,
    model::{FunctionEnv, GlobalEnv},
    ty::Type,
};
use std::{
    fmt,
    fmt::Formatter,
    ops::{Deref, DerefMut},
};

// =================================================================================================
// Data model

/// A node in the access Trie: `data` associated with the parent node + `children` mapping offsets to child nodes
#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
pub struct TrieNode<T: FootprintDomain> {
    /// Optional data associated with the parent in the trie
    data: Option<T>,
    /// Child pointers labeled by offsets
    children: MapDomain<Offset, TrieNode<T>>,
}

/// Set of (root node, child) associations
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct AccessPathTrie<T: FootprintDomain>(MapDomain<Root, TrieNode<T>>);

// =================================================================================================
// Abstract domain operations

impl<T: FootprintDomain> TrieNode<T> {
    pub fn new(data: T) -> Self {
        TrieNode {
            data: Some(data),
            children: MapDomain::default(),
        }
    }

    pub fn new_opt(data: Option<T>) -> Self {
        TrieNode {
            data,
            children: MapDomain::default(),
        }
    }

    /// Like join, but gracefully handles `Non` data fields by treating None as Bottom
    pub fn join_data_opt_(mut data: &mut Option<T>, other: &Option<T>) -> JoinResult {
        match (&mut data, other) {
            (Some(data1), Some(data2)) => data1.join(data2),
            (None, Some(d)) => {
                *data = Some(d.clone());
                JoinResult::Changed
            }
            (_, None) => JoinResult::Unchanged,
        }
    }

    /// Like join, but gracefully handles `Non` data fields by treating None as Bottom
    pub fn join_data_opt(&mut self, other: &Option<T>) -> JoinResult {
        Self::join_data_opt_(&mut self.data, other)
    }

    pub fn join_child_data(&self, mut acc: Option<T>) -> Option<T> {
        Self::join_data_opt_(&mut acc, &self.data);
        for v in self.children.values() {
            Self::join_data_opt_(&mut acc, &v.data);
        }
        acc
    }

    pub fn get_child_data(&self) -> Option<T> {
        let mut acc = None;
        Self::join_data_opt_(&mut acc, &self.data);
        acc
    }

    pub fn data(&self) -> &Option<T> {
        &self.data
    }

    pub fn children(&self) -> &MapDomain<Offset, TrieNode<T>> {
        &self.children
    }

    pub fn entry(&mut self, o: Offset) -> Entry<Offset, TrieNode<T>> {
        self.children.entry(o)
    }

    /// Return the node mapped to `o` from self (if any)
    pub fn get_offset(&self, o: &Offset) -> Option<&Self> {
        self.children.get(o)
    }

    /// Return true if `self`'s keys can be converted into a compact set of concrete access paths
    /// Note: this says nothing about the `data` part of `self`
    pub fn keys_statically_known(&self) -> bool {
        for (offset, child) in self.children.iter() {
            if !offset.is_statically_known() || !child.keys_statically_known() {
                return false;
            }
        }
        true
    }

    /// Bind caller data in `actuals`, `type_actuals`, and `sub_map` to `self`.
    /// (1) Bind all free type variables in `self` to `type_actuals`
    /// (2) Apply `sub_data` to `self.data` and (recursively) to the `data` fields of `self.children`
    pub fn substitute_footprint<F>(
        mut self,
        actuals: &[AbsAddr],
        type_actuals: &[Type],
        sub_map: &dyn AccessPathMap<AbsAddr>,
        mut sub_data: F,
    ) -> Self
    where
        F: FnMut(&mut T, &[AbsAddr], &[Type], &dyn AccessPathMap<AbsAddr>) + Copy,
    {
        match &mut self.data {
            Some(d) => sub_data(d, actuals, type_actuals, sub_map),
            None => (),
        }
        let mut acc = Self::new_opt(self.data);
        for (mut k, v) in self.children.into_iter() {
            k.substitute_footprint(type_actuals);
            acc.children.insert_join(
                k,
                v.substitute_footprint(actuals, type_actuals, sub_map, sub_data),
            );
        }
        acc
    }

    /// Apply `f` to each node in `self`
    pub fn iter_values<F>(&mut self, f: F)
    where
        F: FnMut(&mut TrieNode<T>) + Copy,
    {
        self.children.update_values(f);
    }

    /// Apply `f` to each offset in `self`
    pub fn iter_offsets<F>(&self, mut f: F) -> F
    where
        F: FnMut(&Offset),
    {
        for (k, v) in self.children.iter() {
            f(k);
            f = v.iter_offsets(f);
        }
        f
    }

    /// Apply `f` to all (access path, Option<data>) pairs encoded in `self`
    fn iter_paths_opt<F>(&self, ap: &AccessPath, mut f: F) -> F
    where
        F: FnMut(&AccessPath, &Option<&T>),
    {
        f(ap, &self.data.as_ref());
        for (k, v) in self.children.iter() {
            let mut new_ap = ap.clone();
            new_ap.add_offset(k.clone());
            f = v.iter_paths_opt(&new_ap, f)
        }
        // have to thread F through to avoid constraining it with Copy
        f
    }
}

impl<T: FootprintDomain> AbstractDomain for TrieNode<T> {
    fn join(&mut self, other: &Self) -> JoinResult {
        let data_result = self.join_data_opt(&other.data);
        let children_result = self.children.join(&other.children);
        if data_result == JoinResult::Unchanged && children_result == JoinResult::Unchanged {
            JoinResult::Unchanged
        } else {
            JoinResult::Changed
        }
    }
}

impl<T: FootprintDomain + PartialEq> AbstractDomain for AccessPathTrie<T> {
    fn join(&mut self, other: &Self) -> JoinResult {
        if self == other {
            return JoinResult::Unchanged;
        }
        let mut acc = AccessPathTrie::default();
        acc.join_footprint(self, other);
        acc.join_footprint(other, self);
        *self = acc;
        JoinResult::Changed
    }
}

impl<T: FootprintDomain> AccessPathMap<T> for AccessPathTrie<T> {
    fn get_access_path(&self, ap: AccessPath) -> Option<&T> {
        match self.get_node(ap) {
            Some(n) => n.data.as_ref(),
            None => None,
        }
    }
}

impl<T: FootprintDomain> AccessPathTrie<T> {
    fn join_footprint(&mut self, t1: &Self, t2: &Self) {
        t1.iter_paths_opt(|ap, data1_opt| {
            let data2_opt = t2.get_access_path(ap.clone());
            match (*data1_opt, data2_opt) {
                (Some(data1), Some(data2)) => {
                    let mut new_data = data1.clone();
                    new_data.join(&data2);
                    self.update_access_path_weak(ap.clone(), Some(new_data));
                }
                (None, Some(data)) | (Some(data), None) => {
                    let mut new_data = data.clone();
                    if let Some(footprint) = T::make_footprint(ap.clone()) {
                        new_data.join(&footprint);
                    }
                    self.update_access_path_weak(ap.clone(), Some(new_data));
                }
                (None, None) => (),
            }
        })
    }

    fn get_node(&self, ap: AccessPath) -> Option<&TrieNode<T>> {
        let mut node = match self.0.get(ap.root()) {
            Some(n) => n,
            None => return None,
        };
        for offset in ap.offsets() {
            node = match node.get_offset(offset) {
                Some(n) => n,
                None => return None,
            }
        }
        Some(node)
    }

    pub fn get_child_data(&self) -> Option<T> {
        let mut acc = None;
        for v in self.values() {
            acc = v.join_child_data(acc)
        }
        acc
    }

    /// Like `update_access_path`, but always performs a weak update
    pub fn update_access_path_weak(&mut self, ap: AccessPath, data: Option<T>) {
        self.update_access_path_(ap, TrieNode::new_opt(data), true)
    }

    /// Update `ap` in `global`.
    /// Performs a strong update if the base of `ap` is a local and all offsets are Field's.
    /// Otherwise, performs a weak update (TODO: more details).
    /// Creates nodes for each offset in `ap` if they do not already exist
    pub fn update_access_path(&mut self, ap: AccessPath, data: Option<T>) {
        self.update_access_path_(ap, TrieNode::new_opt(data), false)
    }

    /// Join the value bound to `ap` with `node`
    pub fn join_access_path(&mut self, ap: AccessPath, node: TrieNode<T>) {
        self.update_access_path_(ap, node, true)
    }

    /// Update the value bound to `ap` with `new_node`.
    /// If `weak_update` is true, do this by joining `new_node` with the old value`
    /// If `weak_update` is false, attempt to replace the old value with `new_node`.
    /// However, this may still result in a weak update if `ap` does not permit a strong
    /// update (e.g., if it contains a vector index)
    fn update_access_path_(
        &mut self,
        ap: AccessPath,
        new_node: TrieNode<T>,
        mut weak_update: bool,
    ) {
        let (root, offsets) = ap.into();
        let needs_weak_update = match &root {
            // local base. strong update possible because of Move aliasing semantics
            Root::Local(_) | Root::Formal(_) | Root::Return(_) => false,
            // global base. must do weak update unless g is statically known
            Root::Global(g) => !g.is_statically_known(),
        };
        if needs_weak_update {
            weak_update = true
        };

        let mut node = self.0.entry(root).or_insert_with(TrieNode::default);
        for offset in offsets.into_iter() {
            // if one of the offsets is not statically known, we must do a weak update
            weak_update = weak_update || !offset.is_statically_known();
            node = node.entry(offset).or_insert_with(TrieNode::default);
        }
        if weak_update {
            node.join(&new_node);
        } else {
            // strong update; overwrite data
            *node = new_node
        }
    }

    /// Bind `data` to `local_index` in the trie, overwriting the old value of `local_index`
    pub fn bind_local(&mut self, local_index: TempIndex, data: T, fun_env: &FunctionEnv) {
        self.bind_root(Root::from_index(local_index, fun_env), data)
    }

    /// Bind `node` to `local_index` in the trie, overwriting the old value of `local_index`
    pub fn bind_local_node(
        &mut self,
        local_index: TempIndex,
        node: TrieNode<T>,
        fun_env: &FunctionEnv,
    ) {
        self.0.insert(Root::from_index(local_index, fun_env), node);
    }

    /// Remove the value bound to the local variable `local_index`
    pub fn remove_local(&mut self, local_index: TempIndex, fun_env: &FunctionEnv) {
        self.0.remove(&Root::from_index(local_index, fun_env));
    }

    /// Bind `data` to the return variable `return_index`
    pub fn bind_return(&mut self, return_index: usize, data: T) {
        self.bind_root(Root::ret(return_index), data)
    }

    fn bind_root(&mut self, root: Root, data: T) {
        self.0.insert(root, TrieNode::new(data));
    }

    /// Retrieve the data associated with `local_index` in the trie. Returns `None` if there is no associated data
    pub fn get_local(&self, local_index: TempIndex, fun_env: &FunctionEnv) -> Option<&T> {
        self.get_local_node(local_index, fun_env)
            .map(|n| n.data.as_ref())
            .flatten()
    }

    /// Retrieve the node associated with `local_index` in the trie. Returns `None` if there is no associated node
    pub fn get_local_node(
        &self,
        local_index: TempIndex,
        fun_env: &FunctionEnv,
    ) -> Option<&TrieNode<T>> {
        self.0.get(&Root::from_index(local_index, fun_env))
    }

    /// Return `true` if there is a value bound to local variable `local_index`
    pub fn local_exists(&self, local_index: TempIndex, fun_env: &FunctionEnv) -> bool {
        self.0.contains_key(&Root::from_index(local_index, fun_env))
    }

    /// Return `true` if the keys of `self` have no dynamic components and thus can be converted into
    /// a compact set of concrete access paths.
    pub fn keys_statically_known(&self) -> bool {
        for (root, node) in self.0.iter() {
            if !root.is_statically_known() || !node.keys_statically_known() {
                return false;
            }
        }
        true
    }

    /// Bind caller data in `actuals`, `type_actuals`, and `sub_map` to `self`.
    /// (1) Bind all free type variables in `self` to `type_actuals`
    /// (2) Apply `sub_data` to `self.data` and (recursively) to the `data` fields of `self.children`
    pub fn substitute_footprint<F>(
        self,
        actuals: &[AbsAddr],
        type_actuals: &[Type],
        sub_map: &dyn AccessPathMap<AbsAddr>,
        sub_data: F,
    ) -> Self
    where
        F: FnMut(&mut T, &[AbsAddr], &[Type], &dyn AccessPathMap<AbsAddr>) + Copy,
    {
        let mut acc = Self::default();
        for (mut k, v) in self.0.into_iter() {
            k.substitute_footprint(actuals, type_actuals, sub_map);
            let new_v = v.substitute_footprint(actuals, type_actuals, sub_map, sub_data);
            acc.insert_join(k, new_v);
        }
        acc
    }

    /// Same as `substitute_footprint`, but does not change the `data` field of any node
    pub fn substitute_footprint_skip_data(
        self,
        actuals: &[AbsAddr],
        type_actuals: &[Type],
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) -> Self {
        // TODO: is there a less hacky way to do this?
        fn no_op<T>(_: &mut T, _: &[AbsAddr], _: &[Type], _: &dyn AccessPathMap<AbsAddr>) {}
        self.substitute_footprint(actuals, type_actuals, sub_map, no_op)
    }

    /// Substitute concrete values `actuals` and `type_actuals` into `self`
    pub fn substitute_footprint_concrete(
        self,
        actuals: &[Option<AccountAddress>],
        type_actuals: &[TypeTag],
        sub_map: &dyn AccessPathMap<AbsAddr>,
        env: &GlobalEnv,
    ) -> Self {
        let values = actuals
            .iter()
            .map(|addr_opt| {
                addr_opt
                    .map(|addr| AbsAddr::from(&addr))
                    .unwrap_or_default()
            })
            .collect::<Vec<AbsAddr>>();
        let types = type_actuals
            .iter()
            .map(|t| Type::from_type_tag(t, env))
            .collect::<Vec<Type>>();
        self.substitute_footprint_skip_data(&values, &types, sub_map)
    }

    /// Apply `f` to each node in `self`
    pub fn iter_values<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut TrieNode<T>) + Copy,
    {
        self.update_values(|node| {
            f(node);
            node.iter_values(f);
        });
    }

    /// Apply `f` to each offset in `self`
    pub fn iter_offsets<F>(&self, mut f: F)
    where
        F: FnMut(&Offset),
    {
        for (_k, node) in self.0.iter() {
            f = node.iter_offsets(f)
        }
    }

    /// Apply `f` to each (access path, Option(data)) pair encoded in `self`
    pub fn iter_paths_opt<F>(&self, mut f: F)
    where
        F: FnMut(&AccessPath, &Option<&T>),
    {
        for (root, node) in self.iter() {
            let ap = AccessPath::new_root(root.clone());
            f = node.iter_paths_opt(&ap, f)
        }
    }

    /// Apply `f` to each (access path, data) pair encoded in `self`
    pub fn iter_paths<F>(&self, mut f: F)
    where
        F: FnMut(&AccessPath, &T),
    {
        self.iter_paths_opt(|ap, t_opt| {
            t_opt.map(|t| f(ap, t));
        })
    }

    /// Return a wrapper that of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionEnv) -> AccessPathTrieDisplay<'a, T> {
        AccessPathTrieDisplay { t: self, env }
    }
}

// =================================================================================================
// Boilerplate traits and formatting

impl<T: FootprintDomain> Default for TrieNode<T> {
    fn default() -> Self {
        TrieNode {
            data: None,
            children: MapDomain::default(),
        }
    }
}

impl<T: FootprintDomain> Default for AccessPathTrie<T> {
    fn default() -> Self {
        AccessPathTrie(MapDomain::default())
    }
}

impl<T: FootprintDomain> Deref for AccessPathTrie<T> {
    type Target = MapDomain<Root, TrieNode<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: FootprintDomain> DerefMut for AccessPathTrie<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct AccessPathTrieDisplay<'a, T: FootprintDomain> {
    t: &'a AccessPathTrie<T>,
    env: &'a FunctionEnv<'a>,
}

impl<'a, T: FootprintDomain> fmt::Display for AccessPathTrieDisplay<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.t
            .iter_paths(|path, v| writeln!(f, "{}: {:?}", path.display(self.env), v).unwrap());
        Ok(())
    }
}
