//! `unpacker schema-gen`: regenerate the gamedata `FlatBuffers` schemas from
//! the official CN Windows client, with no third-party schema repository.
//!
//! The text format is the one `OpenArknightsFBS` publishes; on CN 2.7.81 the
//! output matched 59 of its 61 files after normalisation (see
//! [`normalise`]). The rules, measured against that reference:
//!
//! - Members: the serialized fields of the most-derived class in declaration
//!   order, then of each base class; then the auto-properties (public getter
//!   plus any setter, no parameters) of the most-derived class only. A base
//!   member hidden by a derived member of the same name is still visited for
//!   its type but written as a `//` comment line.
//! - A field is serialized when it is an instance field, not literal, not
//!   `[NonSerialized]`, not `[JsonIgnore]`, and public or carries
//!   `[JsonProperty]` (whose ctor string or `PropertyName` renames it) or
//!   `[SerializeField]`. The client has 52,547 `[SerializeField]` fields but
//!   none reaches the 60 reference roots, so the oracle does not test that
//!   rule.
//!   `[JsonIgnore]` drops properties too.
//! - Types: scalars by name, `Obscured*` as their scalar, enums as
//!   `enum enum__Ns_Name : <underlying>` with `ERR_ZERO = 0` appended when no
//!   member is 0, lists and arrays and `List<T>`-derived classes as `[T]`, a
//!   list of lists through `table list_X { values: [X]; }`, dictionaries
//!   (`Dictionary`, `ListDict`, derived classes) as `[dict__K__V]`,
//!   `KeyValuePair` as `kvp__K__V`, `JObject` as `hg__internal__JObject`,
//!   `short[,]` as `hg__internal__MapData`, classes as `clz_<Cecil full name
//!   with every non-alphanumeric as _>`.
//! - Order: depth-first post-order from the root in member order.
//!
//! Exit codes of the command: 0 success; 1 error; 2 `--compare` found a
//! file that is neither IDENTICAL nor allowed; 3 [`StructuralError`], the
//! client changed shape and a human must look.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::il2cpp::attrs::{Attribute, Value};
use super::il2cpp::registration::{TypeRef, kind};
use super::il2cpp::{Il2Cpp, StructuralError};

const MAPDATA: &str = "hg__internal__MapData";
const JOBJECT_TOKEN: &str = "hg__internal__JObject";

const LISTS: [&str; 6] = [
    "System.Collections.Generic.List`1",
    "System.Collections.Generic.HashSet`1",
    "Torappu.ListSet`1",
    "System.Collections.Generic.IList`1",
    "System.Collections.Generic.Queue`1",
    "System.Collections.Generic.Stack`1",
];
const DICTS: [&str; 2] = [
    "System.Collections.Generic.Dictionary`2",
    "System.Collections.Generic.SortedDictionary`2",
];
/// Derives from `List<KeyValuePair<K,V>>` but serializes as a dictionary.
const LISTDICT: &str = "Torappu.ListDict`2";
const KVP: &str = "System.Collections.Generic.KeyValuePair`2";
const JOBJECT: &str = "Newtonsoft.Json.Linq.JObject";
const NULLABLE: &str = "System.Nullable`1";
const DICTIONARY: &str = "System.Collections.Generic.Dictionary`2";
const CONST_TABLE: &str = "Torappu.DB.ConstTable`2";
/// `SimpleKVTable<TValue, TSingleton>`, matched by short name (its namespace
/// is not part of the contract).
const SIMPLE_KV_TABLE: &str = "SimpleKVTable`2";

const ATTR_JSON_PROPERTY: &str = "Newtonsoft.Json.JsonPropertyAttribute";
const ATTR_JSON_IGNORE: &str = "Newtonsoft.Json.JsonIgnoreAttribute";
const ATTR_SERIALIZE_FIELD: &str = "UnityEngine.SerializeField";
const ATTR_CREATE_ASSET_MENU: &str = "UnityEngine.CreateAssetMenuAttribute";

// FieldAttributes / MethodAttributes bits.
const ACCESS_MASK: u16 = 0x7;
const ACCESS_PUBLIC: u16 = 0x6;
const FIELD_STATIC: u16 = 0x10;
const FIELD_LITERAL: u16 = 0x40;
const FIELD_NOT_SERIALIZED: u16 = 0x80;
const METHOD_STATIC: u16 = 0x10;

/// Cecil full name -> fbs scalar.
fn prim_fbs(name: &str) -> Option<&'static str> {
    Some(match name {
        "System.String" => "string",
        "System.Int32" => "int",
        "System.Int64" => "long",
        "System.Single" => "float",
        "System.Double" => "double",
        "System.Boolean" => "bool",
        "System.Byte" => "ubyte",
        "System.SByte" => "byte",
        "System.Int16" => "short",
        "System.UInt16" => "ushort",
        "System.UInt32" => "uint",
        "System.UInt64" => "ulong",
        _ => return None,
    })
}

/// CodeStage anti-cheat wrappers serialize as their plain value.
fn obscured(name: &str) -> Option<&'static str> {
    Some(match name {
        "CodeStage.AntiCheat.ObscuredTypes.ObscuredInt" => "System.Int32",
        "CodeStage.AntiCheat.ObscuredTypes.ObscuredFloat" => "System.Single",
        "CodeStage.AntiCheat.ObscuredTypes.ObscuredLong" => "System.Int64",
        "CodeStage.AntiCheat.ObscuredTypes.ObscuredBool" => "System.Boolean",
        "CodeStage.AntiCheat.ObscuredTypes.ObscuredString" => "System.String",
        "CodeStage.AntiCheat.ObscuredTypes.ObscuredDouble" => "System.Double",
        "CodeStage.AntiCheat.ObscuredTypes.ObscuredShort" => "System.Int16",
        "CodeStage.AntiCheat.ObscuredTypes.ObscuredByte" => "System.Byte",
        _ => return None,
    })
}

/// Cecil full name of a built-in element type.
const fn kind_name(k: u8) -> Option<&'static str> {
    Some(match k {
        kind::BOOLEAN => "System.Boolean",
        kind::CHAR => "System.Char",
        kind::I1 => "System.SByte",
        kind::U1 => "System.Byte",
        kind::I2 => "System.Int16",
        kind::U2 => "System.UInt16",
        kind::I4 => "System.Int32",
        kind::U4 => "System.UInt32",
        kind::I8 => "System.Int64",
        kind::U8 => "System.UInt64",
        kind::R4 => "System.Single",
        kind::R8 => "System.Double",
        kind::STRING => "System.String",
        kind::OBJECT => "System.Object",
        kind::I => "System.IntPtr",
        kind::U => "System.UIntPtr",
        kind::VOID => "System.Void",
        _ => return None,
    })
}

/// Every non-alphanumeric ASCII character (and any non-ASCII one) as `_`.
#[must_use]
pub fn clean(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// A C# type as the generator sees it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    /// A scalar from [`prim_fbs`], by Cecil name.
    Prim(&'static str),
    /// A typedef, with generic arguments when instantiated.
    Def {
        def: u32,
        args: Vec<Self>,
    },
    /// `T[]`.
    Arr(Box<Self>),
    /// `T[,]`.
    Md(Box<Self>),
    /// An unsubstituted class generic parameter.
    Var {
        pos: u16,
        name: String,
    },
    Unknown(String),
}

/// One emitted member: name, type, hidden by a derived member.
type Member = (String, Node, bool);

/// The client's type system plus lookups the generator needs.
pub struct Model<'a> {
    pub il: &'a Il2Cpp,
    by_name: HashMap<&'a str, u32>,
    dictionary: u32,
}

impl<'a> Model<'a> {
    pub fn new(il: &'a Il2Cpp) -> Result<Self> {
        let mut by_name = HashMap::new();
        for d in 0..il.typedef_count() as u32 {
            by_name.entry(il.full_name(d)).or_insert(d);
        }
        let dictionary = *by_name
            .get(DICTIONARY)
            .ok_or_else(|| StructuralError(format!("{DICTIONARY} is not in the client")))?;
        Ok(Self {
            il,
            by_name,
            dictionary,
        })
    }

    fn name(&self, def: u32) -> &'a str {
        self.il.full_name(def)
    }

    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
    }

    /// The generator's view of a resolved type.
    #[must_use]
    pub fn node(&self, t: &TypeRef) -> Node {
        match t {
            TypeRef::Prim(k) => match kind_name(*k) {
                Some(n) if prim_fbs(n).is_some() => Node::Prim(n),
                Some(n) => self.lookup(n).map_or_else(
                    || Node::Unknown(n.to_string()),
                    |def| Node::Def { def, args: vec![] },
                ),
                None => Node::Unknown(format!("kind_{k:#04x}")),
            },
            TypeRef::Def(d) => match obscured(self.name(*d)) {
                Some(p) => Node::Prim(p),
                None => Node::Def {
                    def: *d,
                    args: vec![],
                },
            },
            TypeRef::Generic(d, args) => Node::Def {
                def: *d,
                args: args.iter().map(|a| self.node(a)).collect(),
            },
            TypeRef::SzArray(e) => Node::Arr(Box::new(self.node(e))),
            TypeRef::Array(e, _) => Node::Md(Box::new(self.node(e))),
            TypeRef::Var(p, num) => Node::Var {
                pos: *num,
                name: self.il.generic_param_name(*p).to_string(),
            },
            TypeRef::Other(k) => Node::Unknown(format!("kind_{k:#04x}")),
        }
    }

    fn type_node(&self, index: i32) -> Node {
        self.il.type_ref(index).map_or_else(
            |e| Node::Unknown(format!("type#{index}: {e}")),
            |t| self.node(&t),
        )
    }

    /// Cecil-style full name, e.g. ``Torappu.ListDict`2<System.String,Torappu.X>``.
    #[must_use]
    pub fn unparse(&self, n: &Node) -> String {
        match n {
            Node::Prim(p) => (*p).to_string(),
            Node::Def { def, args } => {
                let mut s = self.name(*def).to_string();
                if !args.is_empty() {
                    s.push('<');
                    s.push_str(
                        &args
                            .iter()
                            .map(|a| self.unparse(a))
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                    s.push('>');
                }
                s
            }
            Node::Arr(e) => format!("{}[]", self.unparse(e)),
            Node::Md(e) => format!("MD[{}]", self.unparse(e)),
            Node::Var { name, .. } => name.clone(),
            Node::Unknown(s) => s.clone(),
        }
    }

    fn base_of(&self, n: &Node) -> Option<Node> {
        let Node::Def { def, args } = n else {
            return None;
        };
        let t = &self.il.meta.type_defs[*def as usize];
        if t.parent_index < 0 {
            return None;
        }
        Some(subst(&self.type_node(t.parent_index), args))
    }

    fn def_name<'n>(&self, n: &'n Node) -> Option<(&'a str, &'n [Node])> {
        match n {
            Node::Def { def, args } => Some((self.name(*def), args.as_slice())),
            _ => None,
        }
    }

    /// Element type when `n` is list-like (array, `List<T>`, or a class
    /// deriving from one).
    fn list_elem(&self, n: &Node) -> Option<Node> {
        if let Node::Arr(e) = n {
            return Some((**e).clone());
        }
        let (name, args) = self.def_name(n)?;
        if LISTS.contains(&name) {
            return args.first().cloned();
        }
        if name == LISTDICT {
            return None;
        }
        let mut b = self.base_of(n);
        for _ in 0..20 {
            let bn = b?;
            let (bname, bargs) = self.def_name(&bn)?;
            if LISTS.contains(&bname) {
                return bargs.first().cloned();
            }
            if DICTS.contains(&bname) || bname == LISTDICT || bname.starts_with("System.") {
                return None;
            }
            b = self.base_of(&bn);
        }
        None
    }

    /// (K, V) when `n` is a class deriving from a dictionary.
    fn dict_base(&self, n: &Node) -> Option<(Node, Node)> {
        let (name, _) = self.def_name(n)?;
        if DICTS.contains(&name) || name == LISTDICT {
            return None;
        }
        let mut b = self.base_of(n);
        for _ in 0..20 {
            let bn = b?;
            let (bname, bargs) = self.def_name(&bn)?;
            if (DICTS.contains(&bname) || bname == LISTDICT) && bargs.len() == 2 {
                return Some((bargs[0].clone(), bargs[1].clone()));
            }
            if bname.starts_with("System.") {
                return None;
            }
            b = self.base_of(&bn);
        }
        None
    }

    fn dict_node(&self, k: Node, v: Node) -> Node {
        Node::Def {
            def: self.dictionary,
            args: vec![k, v],
        }
    }

    fn is_enum(&self, def: u32) -> bool {
        self.il.meta.type_defs[def as usize].is_enum()
    }

    /// The fbs name of a type as a name component.
    #[must_use]
    pub fn token(&self, n: &Node) -> String {
        if let Some((k, v)) = self.dict_base(n) {
            return self.token(&self.dict_node(k, v));
        }
        if matches!(n, Node::Md(_)) {
            return MAPDATA.to_string();
        }
        if let Some(e) = self.list_elem(n) {
            return format!("list_{}", self.token(&e));
        }
        match n {
            Node::Prim(p) => prim_fbs(p).unwrap_or("UNKNOWN").to_string(),
            Node::Def { def, args } => {
                let name = self.name(*def);
                if (DICTS.contains(&name) || name == LISTDICT) && args.len() == 2 {
                    format!(
                        "list_dict__{}__{}",
                        self.token(&args[0]),
                        self.token(&args[1])
                    )
                } else if name == KVP && args.len() == 2 {
                    format!("kvp__{}__{}", self.token(&args[0]), self.token(&args[1]))
                } else if name == JOBJECT {
                    JOBJECT_TOKEN.to_string()
                } else if name == NULLABLE && args.len() == 1 {
                    self.token(&args[0])
                } else if self.is_enum(*def) {
                    format!("enum__{}", clean(name))
                } else {
                    format!("clz_{}", clean(&self.unparse(n)))
                }
            }
            Node::Var { name, .. } => format!("UNKNOWN_{}", clean(name)),
            Node::Unknown(s) => format!("UNKNOWN_{}", clean(s)),
            Node::Arr(_) | Node::Md(_) => unreachable!("handled above"),
        }
    }

    fn attrs(&self, def: u32, token: u32) -> Vec<Attribute> {
        match self.il.attributes(def, token) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("warning: {e:#}");
                Vec::new()
            }
        }
    }

    fn has_attr(&self, attrs: &[Attribute], name: &str) -> bool {
        attrs.iter().any(|a| self.name(a.type_def) == name)
    }

    /// Serialized members of `n` in reference order (see the module doc).
    fn members(&self, n: &Node) -> Vec<Member> {
        let mut chain = Vec::new();
        let mut cur = Some(n.clone());
        while let Some(c) = cur {
            match self.def_name(&c) {
                Some((name, _)) if !name.starts_with("System.") => {}
                _ => break,
            }
            cur = self.base_of(&c);
            chain.push(c);
        }
        let mut res: Vec<Member> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut add = |name: String, node: Node| {
            let hidden = !seen.insert(name.clone());
            res.push((name, node, hidden));
        };
        let meta = &self.il.meta;
        for c in &chain {
            let Node::Def { def, args } = c else { continue };
            let t = &meta.type_defs[*def as usize];
            for fi in t.field_start..t.field_start + i32::from(t.field_count) {
                let f = &meta.fields[fi as usize];
                let Ok(raw) = self.il.raw_type(f.type_index) else {
                    continue;
                };
                let name = meta.string(f.name_index);
                if raw.attrs & (FIELD_STATIC | FIELD_LITERAL | FIELD_NOT_SERIALIZED) != 0
                    || name.starts_with('<')
                {
                    continue;
                }
                let attrs = self.attrs(*def, f.token);
                if self.has_attr(&attrs, ATTR_JSON_IGNORE) {
                    continue;
                }
                let json_property: Vec<&Attribute> = attrs
                    .iter()
                    .filter(|a| self.name(a.type_def) == ATTR_JSON_PROPERTY)
                    .collect();
                if raw.attrs & ACCESS_MASK != ACCESS_PUBLIC
                    && json_property.is_empty()
                    && !self.has_attr(&attrs, ATTR_SERIALIZE_FIELD)
                {
                    continue;
                }
                let mut out_name = name.to_string();
                for a in json_property {
                    if let Some(s) = named_str(a, "PropertyName") {
                        out_name = s.to_string();
                    } else if let Some(s) = a.args.first().and_then(Value::as_str) {
                        out_name = s.to_string();
                    }
                }
                add(out_name, subst(&self.type_node(f.type_index), args));
            }
        }
        // Auto-properties: the most-derived class only. The reference keeps
        // `SoundFXBank.mixerDesc` (declared on the class) and drops
        // `Bank.bankName` (declared on its base); both are public-get
        // auto-properties with a setter.
        if let Some(Node::Def { def, args }) = chain.first() {
            let t = &meta.type_defs[*def as usize];
            for pi in t.property_start..t.property_start + i32::from(t.property_count) {
                let p = &meta.properties[pi as usize];
                let method = |rel: i32| {
                    (rel >= 0)
                        .then(|| meta.methods.get((t.method_start + rel) as usize))
                        .flatten()
                };
                let (Some(get), Some(_set)) = (method(p.get), method(p.set)) else {
                    continue;
                };
                if get.flags & METHOD_STATIC != 0
                    || get.flags & ACCESS_MASK != ACCESS_PUBLIC
                    || get.parameter_count > 0
                {
                    continue;
                }
                let attrs = self.attrs(*def, p.token);
                if self.has_attr(&attrs, ATTR_JSON_IGNORE) {
                    continue;
                }
                let mut out_name = meta.string(p.name_index).to_string();
                for a in &attrs {
                    if self.name(a.type_def) == ATTR_JSON_PROPERTY
                        && let Some(s) = named_str(a, "PropertyName")
                    {
                        out_name = s.to_string();
                    }
                }
                add(out_name, subst(&self.type_node(get.return_type), args));
            }
        }
        res
    }

    /// Token of a typedef as a root (`enum__X` or `clz_X`), for the root map.
    #[must_use]
    pub fn def_token(&self, def: u32) -> String {
        let prefix = if self.is_enum(def) { "enum__" } else { "clz_" };
        format!("{prefix}{}", clean(self.name(def)))
    }
}

fn named_str<'v>(a: &'v Attribute, key: &str) -> Option<&'v str> {
    a.named
        .iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.as_str())
}

/// Replace class generic parameters by position.
fn subst(n: &Node, args: &[Node]) -> Node {
    match n {
        Node::Var { pos, .. } if (*pos as usize) < args.len() => args[*pos as usize].clone(),
        Node::Def { def, args: a } => Node::Def {
            def: *def,
            args: a.iter().map(|x| subst(x, args)).collect(),
        },
        Node::Arr(e) => Node::Arr(Box::new(subst(e, args))),
        Node::Md(e) => Node::Md(Box::new(subst(e, args))),
        other => other.clone(),
    }
}

/// One schema file being written.
pub struct Gen<'m, 'a> {
    m: &'m Model<'a>,
    out: Vec<String>,
    done: HashSet<String>,
    pub unknown: Vec<String>,
}

impl<'m, 'a> Gen<'m, 'a> {
    #[must_use]
    pub fn new(m: &'m Model<'a>) -> Self {
        Self {
            m,
            out: Vec::new(),
            done: HashSet::new(),
            unknown: Vec::new(),
        }
    }

    /// The fbs type expression for a member of type `n`; dependencies are
    /// emitted first.
    fn render(&mut self, n: &Node) -> String {
        let m = self.m;
        if let Some((k, v)) = m.dict_base(n) {
            return self.render(&m.dict_node(k, v));
        }
        if let Node::Md(e) = n {
            if !self.done.contains(MAPDATA) {
                self.done.insert(MAPDATA.to_string());
                let inner = self.render(e);
                self.out.push(format!(
                    "table {MAPDATA} {{\n    row_size: int;\n    column_size: int;\n    matrix_data: [{inner}];\n}}\n"
                ));
            }
            return MAPDATA.to_string();
        }
        if let Some(e) = m.list_elem(n) {
            let nested = m.list_elem(&e).is_some()
                || m.def_name(&e)
                    .is_some_and(|(name, _)| DICTS.contains(&name) || name == LISTDICT);
            if nested {
                let tok = m.token(&e);
                if !self.done.contains(&tok) {
                    let inner = self.render(&e);
                    self.done.insert(tok.clone());
                    self.out
                        .push(format!("table {tok} {{\n    values: {inner};\n}}\n"));
                }
                return format!("[{tok}]");
            }
            return format!("[{}]", self.render(&e));
        }
        if let Some((name, args)) = m.def_name(n) {
            if (DICTS.contains(&name) || name == LISTDICT) && args.len() == 2 {
                let tok = format!("dict__{}__{}", m.token(&args[0]), m.token(&args[1]));
                if !self.done.contains(&tok) {
                    let k = self.render(&args[0]);
                    let v = self.render(&args[1]);
                    self.done.insert(tok.clone());
                    self.out.push(format!(
                        "table {tok} {{\n    key: {k}(key);\n    value: {v};\n}}\n"
                    ));
                }
                return format!("[{tok}]");
            }
            if name == KVP && args.len() == 2 {
                let tok = m.token(n);
                if !self.done.contains(&tok) {
                    let k = self.render(&args[0]);
                    let v = self.render(&args[1]);
                    self.done.insert(tok.clone());
                    self.out.push(format!(
                        "table {tok} {{\n    Key: {k}(key);\n    Value: {v};\n}}\n"
                    ));
                }
                return tok;
            }
            if name == JOBJECT {
                if self.done.insert(JOBJECT_TOKEN.to_string()) {
                    self.out.push(format!(
                        "table {JOBJECT_TOKEN} {{\n    base64: string;\n}}\n"
                    ));
                }
                return JOBJECT_TOKEN.to_string();
            }
            if name == NULLABLE && args.len() == 1 {
                return self.render(&args[0]);
            }
        }
        if let Node::Prim(p) = n {
            return prim_fbs(p).unwrap_or("UNKNOWN").to_string();
        }
        let tok = m.token(n);
        if tok.starts_with("UNKNOWN_") {
            self.unknown.push(m.unparse(n));
            return tok;
        }
        self.emit_type(n);
        tok
    }

    fn emit_type(&mut self, n: &Node) {
        let m = self.m;
        let Node::Def { def, .. } = n else { return };
        let tok = m.token(n);
        if !self.done.insert(tok.clone()) {
            return;
        }
        let meta = &m.il.meta;
        let t = &meta.type_defs[*def as usize];
        if t.is_enum() {
            let under = match m.node(
                &m.il
                    .type_ref(t.element_type_index)
                    .unwrap_or(TypeRef::Other(0)),
            ) {
                Node::Prim(p) => prim_fbs(p).unwrap_or(p).to_string(),
                other => m.unparse(&other),
            };
            let mut s = format!("enum {tok} : {under} {{\n");
            let mut has_zero = false;
            for fi in t.field_start..t.field_start + i32::from(t.field_count) {
                let f = &meta.fields[fi as usize];
                let literal =
                    m.il.raw_type(f.type_index)
                        .is_ok_and(|r| r.attrs & FIELD_LITERAL != 0);
                if !literal {
                    continue;
                }
                let value = match m.il.field_default(fi as u32) {
                    Ok(Some(v)) => v.to_constant_string(),
                    Ok(None) => String::new(),
                    Err(e) => {
                        eprintln!("warning: constant of {}: {e:#}", meta.string(f.name_index));
                        String::new()
                    }
                };
                has_zero |= value == "0";
                let _ = writeln!(s, "    {} = {value},", meta.string(f.name_index));
            }
            if !has_zero {
                // flatbuffers needs a 0 member for the default; the reference
                // appends this sentinel
                s.push_str("    ERR_ZERO = 0,\n");
            }
            s.push_str("}\n");
            self.out.push(s);
            return;
        }
        let mut body = String::new();
        for (name, ty, hidden) in m.members(n) {
            let r = self.render(&ty);
            if hidden {
                let _ = writeln!(body, "    // {name}: {r}; ");
            } else {
                let _ = writeln!(body, "    {name}: {r}; ");
            }
        }
        self.out.push(format!("table {tok} {{\n{body}}}\n"));
    }

    /// The whole file for `root` (a typedef). With `kv_field`, the root is a
    /// `SimpleKVTable<root>`: a synthesized wrapper `{ <kv_field>:
    /// [dict__string__<root>]; }`, written as the reference does, without
    /// blank lines between the dictionary entry, the wrapper and `root_type`.
    #[must_use]
    pub fn run(mut self, root: u32, kv_field: Option<&str>, header: &str) -> (String, Vec<String>) {
        let m = self.m;
        let root_node = Node::Def {
            def: root,
            args: vec![],
        };
        let mut text = String::from(header);
        if let Some(kv) = kv_field {
            let dict = m.dict_node(Node::Prim("System.String"), root_node.clone());
            let before = self.out.len();
            let v = self.render(&dict);
            let wrapper_tok = format!("clz_Torappu_SimpleKVTable_{}", m.token(&root_node));
            let glued = if self.out.len() > before {
                self.out.pop()
            } else {
                None
            };
            text.push_str(&self.out.join("\n"));
            if !self.out.is_empty() {
                text.push('\n');
            }
            if let Some(g) = glued {
                text.push_str(&g);
            }
            let _ = write!(
                text,
                "table {wrapper_tok} {{\n    {kv}: {v};\n}}\nroot_type {wrapper_tok};\n"
            );
        } else {
            self.emit_type(&root_node);
            text.push_str(&self.out.join("\n"));
            let _ = write!(text, "\nroot_type {};\n", m.token(&root_node));
        }
        (text, self.unknown)
    }
}

/// A `.fbs` reduced for comparison: `//` comments stripped to end of line,
/// every whitespace run collapsed to one space, blank lines dropped.
#[must_use]
pub fn normalise(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| {
            let l = l.find("//").map_or(l, |i| &l[..i]);
            l.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .filter(|l| !l.is_empty())
        .collect()
}

/// `assets/schemas/roots.json`: file stem -> root type and, for a
/// `SimpleKVTable<T>` file, the wrapper field name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RootEntry {
    pub root: String,
    pub kv_field: Option<String>,
}

pub type RootMap = BTreeMap<String, RootEntry>;

/// A table class in the client that the root map does not cover.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Discovered {
    /// `ConstTable` or `SimpleKVTable`.
    pub kind: String,
    /// The table class, e.g. `Torappu.ItemDB`.
    pub class: String,
    /// The root token a roots.json entry would name.
    pub root: String,
    /// `[CreateAssetMenu(menuName = ...)]`, when present.
    pub menu: Option<String>,
}

/// `ConstTable<Root, Self>` and `SimpleKVTable<T>` subclasses whose root the
/// map does not name.
#[must_use]
pub fn discover(m: &Model<'_>, roots: &RootMap) -> Vec<Discovered> {
    let covered: HashSet<(&str, bool)> = roots
        .values()
        .map(|e| (e.root.as_str(), e.kv_field.is_some()))
        .collect();
    let const_table = m.lookup(CONST_TABLE);
    let kv_tables: HashSet<u32> = (0..m.il.typedef_count() as u32)
        .filter(|&d| {
            let n = m.name(d);
            n == SIMPLE_KV_TABLE || n.ends_with(&format!(".{SIMPLE_KV_TABLE}"))
        })
        .collect();
    let mut out = Vec::new();
    for d in 0..m.il.typedef_count() as u32 {
        let t = &m.il.meta.type_defs[d as usize];
        if t.parent_index < 0 {
            continue;
        }
        let Ok(TypeRef::Generic(base, args)) = m.il.type_ref(t.parent_index) else {
            continue;
        };
        let (kind, root, kv) = if Some(base) == const_table && args.len() == 2 {
            ("ConstTable", &args[0], false)
        } else if kv_tables.contains(&base) && args.len() == 2 {
            ("SimpleKVTable", &args[0], true)
        } else {
            continue;
        };
        let root = m.token(&m.node(root));
        if covered.contains(&(root.as_str(), kv)) {
            continue;
        }
        let menu = m
            .attrs(d, t.token)
            .iter()
            .filter(|a| m.name(a.type_def) == ATTR_CREATE_ASSET_MENU)
            .find_map(|a| named_str(a, "menuName").map(str::to_string));
        out.push(Discovered {
            kind: kind.to_string(),
            class: m.name(d).to_string(),
            root,
            menu,
        });
    }
    out.sort_by(|a, b| (&a.kind, &a.class).cmp(&(&b.kind, &b.class)));
    out
}

/// `<out>/schema-gen.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub client_version: String,
    /// When this run wrote the files, RFC 3339 UTC.
    pub generated_at: String,
    pub generated: usize,
    pub missing_roots: Vec<String>,
    pub discovered: Vec<Discovered>,
}

pub struct Options {
    pub client: PathBuf,
    pub out: Option<PathBuf>,
    pub roots: PathBuf,
    pub compare: Option<PathBuf>,
    pub allow_diff: Vec<String>,
    pub discover_only: bool,
}

/// `roots.json` beside the schema store.
#[must_use]
pub fn default_roots_path() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap_or(manifest)
        .join("schemas")
        .join("roots.json")
}

/// Generate every file of the root map. Returns stem -> text and the stems
/// whose root is not in the client.
pub fn generate_all(
    m: &Model<'_>,
    roots: &RootMap,
    client_version: &str,
) -> (BTreeMap<String, String>, Vec<String>) {
    let mut by_token: HashMap<String, u32> = HashMap::new();
    for d in 0..m.il.typedef_count() as u32 {
        by_token.entry(m.def_token(d)).or_insert(d);
    }
    let header = format!(
        "// Generated by unpacker schema-gen from the CN client {client_version}; see assets/schemas/README.md.\n"
    );
    let mut files = BTreeMap::new();
    let mut missing = Vec::new();
    for (stem, e) in roots {
        let Some(&root) = by_token.get(&e.root) else {
            missing.push(stem.clone());
            continue;
        };
        let (text, unknown) = Gen::new(m).run(root, e.kv_field.as_deref(), &header);
        if !unknown.is_empty() {
            let mut u = unknown;
            u.sort();
            u.dedup();
            eprintln!("warning: {stem}: unresolved types {u:?}");
        }
        files.insert(stem.clone(), text);
    }
    (files, missing)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Identical,
    Diff(Vec<String>),
    Missing,
}

/// Compare generated files with every `.fbs` in `dir`.
pub fn compare(files: &BTreeMap<String, String>, dir: &Path) -> Result<BTreeMap<String, Verdict>> {
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("fbs") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let reference = std::fs::read_to_string(&path)?;
        let verdict = match files.get(&stem) {
            None => Verdict::Missing,
            Some(g) => {
                let (a, b) = (normalise(&reference), normalise(g));
                if a == b {
                    Verdict::Identical
                } else {
                    Verdict::Diff(line_diff(&a, &b))
                }
            }
        };
        out.insert(stem, verdict);
    }
    Ok(out)
}

/// Lines only in the reference (`-`) or only in the generated file (`+`),
/// as multisets.
fn line_diff(reference: &[String], generated: &[String]) -> Vec<String> {
    let mut count: HashMap<&str, i64> = HashMap::new();
    for l in reference {
        *count.entry(l).or_default() += 1;
    }
    for l in generated {
        *count.entry(l).or_default() -= 1;
    }
    let mut out = Vec::new();
    for l in reference {
        if let Some(c) = count.get_mut(l.as_str())
            && *c > 0
        {
            *c -= 1;
            out.push(format!("- {l}"));
        }
    }
    for l in generated {
        if let Some(c) = count.get_mut(l.as_str())
            && *c < 0
        {
            *c += 1;
            out.push(format!("+ {l}"));
        }
    }
    if out.is_empty() {
        out.push("(same lines, different order)".to_string());
    }
    out
}

/// Unix seconds as `YYYY-MM-DDTHH:MM:SSZ` (civil-from-days, valid after
/// 1970).
#[must_use]
pub fn rfc3339_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem / 60) % 60,
        rem % 60
    )
}

#[derive(Debug, Deserialize)]
struct ClientJson {
    client_version: String,
}

/// The whole command. Returns the process exit code.
pub fn run(opts: &Options) -> Result<i32> {
    let dll = opts.client.join("GameAssembly.dll");
    let meta = opts.client.join("global-metadata.dat");
    let client_json = opts.client.join("client.json");
    let client_version = match std::fs::read_to_string(&client_json) {
        Ok(s) => {
            serde_json::from_str::<ClientJson>(&s)
                .with_context(|| format!("parsing {}", client_json.display()))?
                .client_version
        }
        Err(_) => {
            eprintln!(
                "warning: no {}; client version unknown (run `downloader -s cn client-download`)",
                client_json.display()
            );
            "unknown".to_string()
        }
    };
    let roots: RootMap = serde_json::from_str(
        &std::fs::read_to_string(&opts.roots)
            .with_context(|| format!("reading {}", opts.roots.display()))?,
    )
    .with_context(|| format!("parsing {}", opts.roots.display()))?;

    let il = Il2Cpp::load(&dll, &meta)?;
    let model = Model::new(&il)?;
    let discovered = discover(&model, &roots);
    for d in &discovered {
        eprintln!(
            "warning: table not in the root map: {} {} root {} menu {}",
            d.kind,
            d.class,
            d.root,
            d.menu.as_deref().unwrap_or("-")
        );
    }
    if opts.discover_only {
        println!("{}", serde_json::to_string_pretty(&discovered)?);
        return Ok(0);
    }
    let out_dir = opts
        .out
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--out is required unless --discover"))?;

    let (files, missing_roots) = generate_all(&model, &roots, &client_version);
    std::fs::create_dir_all(out_dir)?;
    for (stem, text) in &files {
        std::fs::write(out_dir.join(format!("{stem}.fbs")), text)?;
    }
    if client_json.exists() {
        std::fs::copy(&client_json, out_dir.join("client.json"))?;
    }
    for stem in &missing_roots {
        eprintln!(
            "warning: {stem}: root {} is not in the client",
            roots[stem].root
        );
    }
    let summary = Summary {
        client_version: client_version.clone(),
        generated_at: rfc3339_utc(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
        ),
        generated: files.len(),
        missing_roots,
        discovered,
    };
    std::fs::write(
        out_dir.join("schema-gen.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    println!(
        "schema-gen: client {client_version}, {} files written to {}, {} roots missing, {} tables undiscovered",
        summary.generated,
        out_dir.display(),
        summary.missing_roots.len(),
        summary.discovered.len()
    );

    let Some(dir) = &opts.compare else {
        return Ok(0);
    };
    let verdicts = compare(&files, dir)?;
    let mut identical = 0;
    let mut failing = 0;
    for (stem, v) in &verdicts {
        let allowed = opts.allow_diff.iter().any(|a| a == stem);
        let label = match v {
            Verdict::Identical => "IDENTICAL",
            Verdict::Diff(_) => "DIFF",
            Verdict::Missing => "MISSING",
        };
        let note = if allowed && *v != Verdict::Identical {
            " (allowed)"
        } else {
            ""
        };
        println!("{stem:<32} {label}{note}");
        match v {
            Verdict::Identical => identical += 1,
            Verdict::Diff(lines) => {
                for l in lines.iter().take(12) {
                    println!("    {l}");
                }
                if lines.len() > 12 {
                    println!("    ... {} more", lines.len() - 12);
                }
            }
            Verdict::Missing => {}
        }
        if *v != Verdict::Identical && !allowed {
            failing += 1;
        }
    }
    for stem in files.keys().filter(|s| !verdicts.contains_key(*s)) {
        println!("{stem:<32} NEW (not in {})", dir.display());
    }
    println!(
        "identical {identical} / {}; {failing} not identical and not allowed",
        verdicts.len()
    );
    Ok(if failing == 0 { 0 } else { 2 })
}

/// Map an error to the exit code: 3 for a [`StructuralError`], else 1.
#[must_use]
pub fn exit_code(e: &anyhow::Error) -> i32 {
    if e.chain()
        .any(|c| c.downcast_ref::<StructuralError>().is_some())
    {
        3
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_names() {
        assert_eq!(
            clean("Torappu.CharSkinData/DisplaySkin"),
            "Torappu_CharSkinData_DisplaySkin"
        );
        assert_eq!(
            clean("Torappu.ListDict`2<System.String,Torappu.Foo[]>"),
            "Torappu_ListDict_2_System_String_Torappu_Foo___"
        );
        assert_eq!(clean("Ab9_é"), "Ab9__");
    }

    #[test]
    fn formats_rfc3339() {
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339_utc(1_792_067_696), "2026-10-15T12:34:56Z");
    }

    #[test]
    fn normalises() {
        let a = "// header\ntable  X {\n    a: int; \n\n    // b: int; \n}\n\nroot_type X;\n";
        let b = "table X {\n a: int;\n}\nroot_type X; // trailing\n";
        assert_eq!(normalise(a), normalise(b));
        assert_eq!(
            normalise(a),
            vec!["table X {", "a: int;", "}", "root_type X;"]
        );
    }

    #[test]
    fn substitutes_generic_parameters() {
        let t = Node::Var {
            pos: 1,
            name: "V".into(),
        };
        let n = Node::Arr(Box::new(t));
        let out = subst(
            &n,
            &[Node::Prim("System.Int32"), Node::Prim("System.String")],
        );
        assert_eq!(out, Node::Arr(Box::new(Node::Prim("System.String"))));
    }

    #[test]
    fn diffs_as_multisets() {
        let a: Vec<String> = ["x", "y", "y"].iter().map(ToString::to_string).collect();
        let b: Vec<String> = ["y", "z"].iter().map(ToString::to_string).collect();
        assert_eq!(line_diff(&a, &b), vec!["- x", "- y", "+ z"]);
    }

    /// The full oracle: every reference file in `assets/schemas/cn` except
    /// the two known exceptions regenerates identically.
    #[test]
    #[ignore = "needs MYRTLE_CN_CLIENT_DIR pointing at a client-download dir"]
    fn oracle_against_reference() {
        let dir =
            PathBuf::from(std::env::var("MYRTLE_CN_CLIENT_DIR").expect("MYRTLE_CN_CLIENT_DIR"));
        let il = Il2Cpp::load(
            &dir.join("GameAssembly.dll"),
            &dir.join("global-metadata.dat"),
        )
        .unwrap();
        let model = Model::new(&il).unwrap();
        let roots: RootMap =
            serde_json::from_str(&std::fs::read_to_string(default_roots_path()).unwrap()).unwrap();
        let (files, _) = generate_all(&model, &roots, "test");
        let reference = default_roots_path().with_file_name("cn");
        let verdicts = compare(&files, &reference).unwrap();
        let identical = verdicts
            .values()
            .filter(|v| **v == Verdict::Identical)
            .count();
        let bad: Vec<_> = verdicts
            .iter()
            .filter(|(s, v)| {
                **v != Verdict::Identical && !["audio_data", "sandbox_table"].contains(&s.as_str())
            })
            .collect();
        assert!(bad.is_empty(), "{bad:#?}");
        assert!(identical >= 59, "identical {identical}");
    }
}
