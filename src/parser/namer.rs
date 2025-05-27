use std::collections::BTreeMap;

use crate::expr::Name;
use crate::fmt::NameTable;


/// A context that binds identifiers to the correct names.
pub struct NameContext {
    next_unique_name: Name,
    next_scope_id: usize,

    bound: Vec<BTreeMap<String, Name>>,
    unbound: BTreeMap<String, Name>,

    rev_table: NameTable
}

impl NameContext {
    /// Creates a new [NameContext]
    pub fn new() -> Self {
        Self {
            next_unique_name: Name::any(),
            next_scope_id: 0,

            bound: Vec::new(),
            unbound: BTreeMap::new(),

            rev_table: NameTable::new()
        }
    }

    pub fn rev_table(&self) -> &NameTable {
        &self.rev_table
    }

    pub fn into_rev_table(self) -> NameTable {
        self.rev_table
    }

    /// Generates a new unique name
    fn new_name(&mut self) -> Name {
        return self.next_unique_name.incr();
    }

    /// Saves a bound name to the reverse table.
    fn save_bound_name(&mut self, str: &String, name: Name) {
        let counter = self.next_scope_id;
        self.next_scope_id += 1;


        self.rev_table.add_bound(name, str.clone(), counter);
    }

    /// Saves a global name to the reverse table.
    fn save_static_name(&mut self, str: &String, name: Name) {
        self.rev_table.add_unbound(name, str.clone());
    }

    /// Resolves a global name, it will not bind it to any scope name.
    pub fn resolve_static(&mut self, str: String) -> Name {
        if let Some(name) = self.unbound.get(&str) {
            return *name;
        }

        let name = self.new_name();
        self.save_static_name(&str, name);

        self.unbound.insert(str, name);

        name
    }

    /// Resolves a bound name, it try to bind it to a scope name, otherwise it will return [None].
    pub fn resolve_bound(&mut self, str: &String) -> Option<Name> {
        if let Some(scope) = self.bound.last() {
            if let Some(name) = scope.get(str) {
                return Some(*name);
            }
        }

        None
    }

    /// Enter a new scope in which the given name is bound.
    pub fn enter(&mut self, str: String) -> Name {
        let name = self.new_name();
        self.save_bound_name(&str, name);

        let mut new_scope = self.bound.last().cloned().unwrap_or_else(|| BTreeMap::new());
        new_scope.insert(str, name);
        self.bound.push(new_scope);

        name
    }

    /// Leave a scope that was entered by [Self::enter]. Panics if no scope was ever entered.
    pub fn leave(&mut self) {
        self.bound.pop().expect("Cannot leave global scope");
    }
}




