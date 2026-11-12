use super::types::*;
use crate::compiler::ast::TypeRef;
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::LazyLock;

static NORMALIZE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?i)(CHAR|NUMC|DEC)\s*\(\s*(\d+)(?:\s*,\s*(\d+))?\s*\)$").unwrap()
});

static SIZED_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?i)(CHAR|NUMC)\((\d+)\)$").unwrap()
});

static CDS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bdefine\s+(?:root\s+)?view\s+entity\s+([A-Za-z_][A-Za-z0-9_]*)\s+as\s+select\s+from\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{").unwrap()
});

static CDS_FIELD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?i)(key\s+)?([A-Za-z_][A-Za-z0-9_]*)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$").unwrap()
});

static ANNOTATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"@([A-Za-z0-9_.]+)\s*:\s*([^\n]+)").unwrap()
});

pub fn default_table_objects() -> Vec<LocalObject> {
    vec![
        LocalObject {
            name: "SCARR".to_string(),
            kind: "TABL".to_string(),
            description: "Airline master data (fictional local fixture)".to_string(),
            fields: Some(vec![
                LocalField { name: "CARRID".to_string(), field_type: "CHAR(3)".to_string(), key: Some(true), description: "Airline ID".to_string(), nullable: None },
                LocalField { name: "CARRNAME".to_string(), field_type: "CHAR(25)".to_string(), key: None, description: "Airline name".to_string(), nullable: None },
                LocalField { name: "CURRCODE".to_string(), field_type: "CUKY".to_string(), key: None, description: "Currency key".to_string(), nullable: None },
            ]),
            source: None,
            active: Some(true),
            base_type: None,
        },
        LocalObject {
            name: "SPFLI".to_string(),
            kind: "TABL".to_string(),
            description: "Flight connection sample data".to_string(),
            fields: Some(vec![
                LocalField { name: "CARRID".to_string(), field_type: "CHAR(3)".to_string(), key: Some(true), description: "Airline ID".to_string(), nullable: None },
                LocalField { name: "CONNID".to_string(), field_type: "NUMC(4)".to_string(), key: Some(true), description: "Connection number".to_string(), nullable: None },
                LocalField { name: "COUNTRYFR".to_string(), field_type: "CHAR(3)".to_string(), key: None, description: "Departure country".to_string(), nullable: None },
                LocalField { name: "CITYFROM".to_string(), field_type: "CHAR(20)".to_string(), key: None, description: "Departure city".to_string(), nullable: None },
                LocalField { name: "COUNTRYTO".to_string(), field_type: "CHAR(3)".to_string(), key: None, description: "Arrival country".to_string(), nullable: None },
                LocalField { name: "CITYTO".to_string(), field_type: "CHAR(20)".to_string(), key: None, description: "Arrival city".to_string(), nullable: None },
            ]),
            source: None,
            active: Some(true),
            base_type: None,
        },
        LocalObject {
            name: "SFLIGHT".to_string(),
            kind: "TABL".to_string(),
            description: "Flight schedule sample data".to_string(),
            fields: Some(vec![
                LocalField { name: "CARRID".to_string(), field_type: "CHAR(3)".to_string(), key: Some(true), description: "Airline ID".to_string(), nullable: None },
                LocalField { name: "CONNID".to_string(), field_type: "NUMC(4)".to_string(), key: Some(true), description: "Connection number".to_string(), nullable: None },
                LocalField { name: "FLDATE".to_string(), field_type: "DATS".to_string(), key: Some(true), description: "Flight date".to_string(), nullable: None },
                LocalField { name: "PRICE".to_string(), field_type: "CURR".to_string(), key: None, description: "Sample price".to_string(), nullable: None },
            ]),
            source: None,
            active: Some(true),
            base_type: None,
        },
        LocalObject {
            name: "ZPRODUCT".to_string(),
            kind: "TABL".to_string(),
            description: "Local RAP product entity persistence table".to_string(),
            fields: Some(vec![
                LocalField { name: "PRODUCT_ID".to_string(), field_type: "CHAR(12)".to_string(), key: Some(true), description: "Product key".to_string(), nullable: None },
                LocalField { name: "PRODUCT_NAME".to_string(), field_type: "CHAR(60)".to_string(), key: None, description: "Product description".to_string(), nullable: None },
                LocalField { name: "CATEGORY".to_string(), field_type: "CHAR(20)".to_string(), key: None, description: "Product category".to_string(), nullable: None },
                LocalField { name: "PRICE".to_string(), field_type: "DEC(13,2)".to_string(), key: None, description: "Product price".to_string(), nullable: None },
                LocalField { name: "CURRENCY".to_string(), field_type: "CUKY".to_string(), key: None, description: "Currency key".to_string(), nullable: None },
            ]),
            source: None,
            active: Some(true),
            base_type: None,
        },
    ]
}

pub fn default_built_in_objects() -> Vec<LocalObject> {
    let mut objs = default_table_objects();
    let zprod_fields = objs.iter().find(|x| x.name == "ZPRODUCT").unwrap().fields.clone();

    objs.push(LocalObject {
        name: "ZDM_PRODUCT_ID".to_string(),
        kind: "DOMA".to_string(),
        description: "Product identifier domain".to_string(),
        fields: None,
        source: None,
        active: Some(true),
        base_type: Some("CHAR(12)".to_string()),
    });
    objs.push(LocalObject {
        name: "ZDM_CURRENCY".to_string(),
        kind: "DOMA".to_string(),
        description: "Three-character currency key domain".to_string(),
        fields: None,
        source: None,
        active: Some(true),
        base_type: Some("CUKY".to_string()),
    });
    objs.push(LocalObject {
        name: "ZDE_PRODUCT_ID".to_string(),
        kind: "DTEL".to_string(),
        description: "Product identifier data element".to_string(),
        fields: Some(vec![LocalField {
            name: "PRODUCT_ID".to_string(),
            field_type: "CHAR(12)".to_string(),
            key: Some(true),
            description: "Product identifier".to_string(),
            nullable: None,
        }]),
        source: None,
        active: Some(true),
        base_type: None,
    });
    objs.push(LocalObject {
        name: "ZDE_PRODUCT_NAME".to_string(),
        kind: "DTEL".to_string(),
        description: "Product name data element".to_string(),
        fields: Some(vec![LocalField {
            name: "PRODUCT_NAME".to_string(),
            field_type: "CHAR(60)".to_string(),
            key: None,
            description: "Product name".to_string(),
            nullable: None,
        }]),
        source: None,
        active: Some(true),
        base_type: None,
    });
    objs.push(LocalObject {
        name: "ZSTR_PRODUCT".to_string(),
        kind: "STRU".to_string(),
        description: "Product structure".to_string(),
        fields: zprod_fields.clone(),
        source: None,
        active: Some(true),
        base_type: None,
    });
    objs.push(LocalObject {
        name: "ZI_PRODUCT".to_string(),
        kind: "VIEW".to_string(),
        description: "Sample product interface view entity".to_string(),
        source: Some("ZPRODUCT".to_string()),
        active: Some(true),
        fields: zprod_fields,
        base_type: None,
    });
    objs.push(LocalObject {
        name: "ZR_PRODUCT".to_string(),
        kind: "BDEF".to_string(),
        description: "Sample managed-style product behavior definition (simulated)".to_string(),
        source: Some("ZI_PRODUCT".to_string()),
        active: Some(true),
        fields: None,
        base_type: None,
    });
    objs
}

#[derive(Debug, Clone)]
pub struct LocalPlatform {
    pub tables: HashMap<String, Vec<Value>>,
    pub views: HashMap<String, CdsView>,
    pub custom_objects: Vec<LocalObject>,
    pub sessions: HashMap<String, Vec<RapOperation>>,
}

impl Default for LocalPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalPlatform {
    pub fn new() -> Self {
        Self {
            tables: crate::compiler::vm::default_vm_database(),
            views: HashMap::new(),
            custom_objects: Vec::new(),
            sessions: HashMap::new(),
        }
    }

    pub fn list_objects(&self) -> Vec<LocalObject> {
        let mut list = Vec::new();
        let builtins = default_built_in_objects();
        list.extend(builtins.into_iter().filter(|o| o.kind != "VIEW"));

        for view in self.views.values() {
            let fields = view
                .fields
                .iter()
                .map(|f| LocalField {
                    name: f.alias.clone(),
                    field_type: self.field_type(&view.source, &f.source),
                    key: Some(f.key),
                    description: format!("Projected from {}.{}", view.source, f.source),
                    nullable: None,
                })
                .collect();

            list.push(LocalObject {
                name: view.name.clone(),
                kind: "VIEW".to_string(),
                description: format!("Activated local CDS projection on {}", view.source),
                fields: Some(fields),
                source: Some(view.source.clone()),
                active: Some(true),
                base_type: None,
            });
        }

        list.extend(self.custom_objects.clone());
        list
    }

    pub fn get_object(&self, name: &str) -> Option<LocalObject> {
        let key = name.to_ascii_uppercase();
        self.list_objects().into_iter().find(|o| o.name.eq_ignore_ascii_case(&key))
    }

    pub fn list_tables(&self) -> Vec<String> {
        let mut t: Vec<String> = self.tables.keys().cloned().collect();
        t.sort();
        t
    }

    pub fn table_rows(&self, name: &str) -> Result<Vec<Value>, String> {
        let key = name.to_ascii_uppercase();
        self.tables
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("Local table '{}' does not exist.", key))
    }

    pub fn table_schema(&self, name: &str) -> Option<LocalObject> {
        let key = name.to_ascii_uppercase();
        let all = default_table_objects();
        all.into_iter()
            .chain(self.custom_objects.clone())
            .find(|o| o.kind == "TABL" && o.name.eq_ignore_ascii_case(&key))
    }

    pub fn create_object(
        &mut self,
        name: &str,
        kind: &str,
        description: Option<&str>,
        base_type: Option<&str>,
        fields: Option<Vec<LocalField>>,
        initial_rows: Option<Vec<Value>>,
    ) -> Result<LocalObject, String> {
        let name_upper = name.trim().to_ascii_uppercase();
        let kind_upper = kind.trim().to_ascii_uppercase();

        let ident_re = Regex::new(r"^[ZY][A-Z0-9_]{0,29}$").unwrap();
        if !ident_re.is_match(&name_upper) {
            return Err("Local repository object names must start with Z or Y and contain at most 30 letters, digits or underscores.".to_string());
        }

        if !matches!(kind_upper.as_str(), "TABL" | "STRU" | "DTEL" | "DOMA") {
            return Err(format!("Unsupported local object kind '{}'.", kind));
        }

        if self.get_object(&name_upper).is_some() {
            return Err(format!("Local repository object '{}' already exists.", name_upper));
        }

        let normalize_type = |val: &str| -> Result<String, String> {
            let t = val.trim().to_ascii_uppercase();
            if let Some(caps) = NORMALIZE_REGEX.captures(&t) {
                let p1 = caps.get(1).unwrap().as_str();
                let p2 = caps.get(2).unwrap().as_str();
                let p3 = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                return Ok(if p3.is_empty() {
                    format!("{}({})", p1, p2)
                } else {
                    format!("{}({},{})", p1, p2, p3)
                });
            }
            let allowed: HashSet<&str> = [
                "I", "INT8", "DECFLOAT16", "DECFLOAT34", "STRING", "XSTRING", "ABAP_BOOL", "DATS", "TIMS",
                "CUKY", "CURR", "I1", "I2", "I4", "F", "CHAR", "NUMC", "DEC",
            ]
            .into_iter()
            .collect();
            if allowed.contains(t.as_str()) {
                return Ok(t);
            }
            if let Some(referenced) = self.get_object(&t) {
                if matches!(referenced.kind.as_str(), "DTEL" | "DOMA" | "STRU" | "TABL") {
                    return Ok(t);
                }
            }
            Err(format!(
                "Unsupported local DDIC type '{}'. Use a built-in type or an existing local DTEL, DOMA, STRU or TABL object.",
                val
            ))
        };

        let mut processed_fields = Vec::new();
        if let Some(fs) = fields {
            for f in fs {
                let norm = normalize_type(&f.field_type)?;
                processed_fields.push(LocalField {
                    name: f.name.trim().to_ascii_uppercase(),
                    field_type: norm,
                    key: f.key,
                    description: f.description,
                    nullable: f.nullable,
                });
            }
        }

        if matches!(kind_upper.as_str(), "TABL" | "STRU") {
            if processed_fields.is_empty() {
                return Err(format!("{} objects require at least one field.", kind_upper));
            }
            let mut names = HashSet::new();
            let field_re = Regex::new(r"^[A-Z][A-Z0-9_]{0,29}$").unwrap();
            for f in &processed_fields {
                if !field_re.is_match(&f.name) {
                    return Err(format!("Invalid field name '{}'.", f.name));
                }
                if names.contains(&f.name) {
                    return Err(format!("Duplicate field '{}'.", f.name));
                }
                names.insert(f.name.clone());
            }
            if kind_upper == "TABL" && !processed_fields.iter().any(|f| f.key.unwrap_or(false)) {
                return Err("A local transparent training table requires at least one key field.".to_string());
            }
        }

        let base_ty = if matches!(kind_upper.as_str(), "DTEL" | "DOMA") {
            if let Some(bt) = base_type {
                Some(normalize_type(bt)?)
            } else {
                None
            }
        } else {
            None
        };

        let obj = LocalObject {
            name: name_upper.clone(),
            kind: kind_upper.clone(),
            description: description
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("User-created local {} practice object", kind_upper)),
            fields: if !processed_fields.is_empty() {
                Some(processed_fields.clone())
            } else {
                None
            },
            source: None,
            active: Some(true),
            base_type: base_ty,
        };

        if kind_upper == "TABL" {
            let mut prepared = Vec::new();
            if let Some(rows) = initial_rows {
                for (index, row) in rows.iter().enumerate() {
                    let mut out = serde_json::Map::new();
                    if let Value::Object(row_map) = row {
                        for (k, v) in row_map {
                            let field = processed_fields
                                .iter()
                                .find(|f| f.name.eq_ignore_ascii_case(k))
                                .ok_or_else(|| format!("Initial row contains unknown field '{}'.", k))?;
                            out.insert(field.name.clone(), v.clone());
                        }
                    }
                    for f in &processed_fields {
                        if !out.contains_key(&f.name) {
                            if f.key.unwrap_or(false) {
                                return Err(format!("Initial row {} is missing key field '{}'.", index + 1, f.name));
                            }
                            out.insert(f.name.clone(), Value::Null);
                        }
                    }
                    prepared.push(Value::Object(out));
                }
            }

            let key_fields: Vec<String> = processed_fields
                .iter()
                .filter(|f| f.key.unwrap_or(false))
                .map(|f| f.name.clone())
                .collect();
            let mut signatures = HashSet::new();
            for row in &prepared {
                if let Value::Object(row_map) = row {
                    let sig: Vec<String> = key_fields
                        .iter()
                        .map(|k| crate::compiler::vm::format_value(row_map.get(k).unwrap_or(&Value::Null)))
                        .collect();
                    let sig_str = sig.join("|");
                    if signatures.contains(&sig_str) {
                        return Err(format!("Initial rows contain a duplicate table key ({}).", sig_str));
                    }
                    signatures.insert(sig_str);
                }
            }
            self.tables.insert(name_upper.clone(), prepared);
        }

        self.custom_objects.push(obj.clone());
        Ok(obj)
    }

    pub fn delete_object(&mut self, name: &str) -> Result<LocalObject, String> {
        let key = name.trim().to_ascii_uppercase();
        let index = self
            .custom_objects
            .iter()
            .position(|o| o.name.eq_ignore_ascii_case(&key))
            .ok_or_else(|| format!("Local custom repository object '{}' does not exist or is not deletable.", key))?;

        let dependents: Vec<&LocalObject> = self
            .custom_objects
            .iter()
            .filter(|o| {
                o.name != key
                    && (o.fields
                        .as_ref()
                        .map(|fs| fs.iter().any(|f| f.field_type.eq_ignore_ascii_case(&key)))
                        .unwrap_or(false)
                        || o.base_type
                            .as_ref()
                            .map(|b| b.eq_ignore_ascii_case(&key))
                            .unwrap_or(false))
            })
            .collect();

        let views: Vec<&CdsView> = self.views.values().filter(|v| v.source.eq_ignore_ascii_case(&key)).collect();
        if !dependents.is_empty() || !views.is_empty() {
            return Err(format!("Cannot delete '{}' while dependent local objects or CDS views exist.", key));
        }

        let removed = self.custom_objects.remove(index);
        if removed.kind == "TABL" {
            self.tables.remove(&key);
        }
        Ok(removed)
    }

    pub fn get_compiler_types(&self) -> HashMap<String, TypeRef> {
        let mut result = HashMap::new();
        let catalogue = self.list_objects();
        let by_name: HashMap<String, LocalObject> = catalogue
            .iter()
            .map(|o| (o.name.to_ascii_uppercase(), o.clone()))
            .collect();

        fn resolve(
            raw: &str,
            by_name: &HashMap<String, LocalObject>,
            seen: &mut HashSet<String>,
        ) -> Result<TypeRef, String> {
            let t = raw.trim().to_ascii_uppercase();
            if let Some(object) = by_name.get(&t) {
                if seen.contains(&t) {
                    return Err(format!("Circular local DDIC type reference involving '{}'.", t));
                }
                seen.insert(t.clone());
                if object.kind == "DTEL" || object.kind == "DOMA" {
                    let base = object
                        .base_type
                        .as_deref()
                        .or_else(|| object.fields.as_ref().and_then(|fs| fs.first()).map(|f| f.field_type.as_str()));
                    if let Some(b) = base {
                        return resolve(b, by_name, seen);
                    }
                    return Err(format!("Local DDIC object '{}' has no base type.", t));
                }
                if object.kind == "STRU" || object.kind == "TABL" {
                    let mut fields = BTreeMap::new();
                    if let Some(fs) = &object.fields {
                        for f in fs {
                            fields.insert(f.name.clone(), resolve(&f.field_type, by_name, seen)?);
                        }
                    }
                    return Ok(TypeRef::Struct {
                        name: t,
                        fields,
                    });
                }
            }

            if matches!(t.as_str(), "I" | "I1" | "I2" | "I4") {
                return Ok(TypeRef::scalar("i"));
            }
            if t == "INT8" {
                return Ok(TypeRef::scalar("int8"));
            }
            if t == "STRING" {
                return Ok(TypeRef::scalar("string"));
            }
            if t == "XSTRING" {
                return Ok(TypeRef::scalar("xstring"));
            }
            if t == "ABAP_BOOL" {
                return Ok(TypeRef::scalar("abap_bool"));
            }
            if t == "DATS" {
                return Ok(TypeRef::scalar("d"));
            }
            if t == "TIMS" {
                return Ok(TypeRef::scalar("t"));
            }
            if t == "DECFLOAT16" {
                return Ok(TypeRef::scalar("decfloat16"));
            }
            if t == "DECFLOAT34" || t.starts_with("DEC(") || t == "DEC" || t == "CURR" || t == "F" {
                return Ok(TypeRef::scalar("decfloat34"));
            }
            if let Some(caps) = SIZED_REGEX.captures(&t) {
                let p1 = caps.get(1).unwrap().as_str();
                let len = caps.get(2).unwrap().as_str().parse::<usize>().ok();
                let sname = if p1 == "NUMC" { "n" } else { "c" };
                return Ok(TypeRef::Scalar {
                    name: sname.to_string(),
                    length: len,
                });
            }
            if t == "CUKY" {
                return Ok(TypeRef::Scalar {
                    name: "c".to_string(),
                    length: Some(5),
                });
            }
            Err(format!("Unknown local DDIC type '{}'.", raw))
        }

        for object in &catalogue {
            if matches!(object.kind.as_str(), "STRU" | "TABL" | "DTEL" | "DOMA") {
                let mut seen = HashSet::new();
                if let Ok(ty) = resolve(&object.name, &by_name, &mut seen) {
                    result.insert(object.name.clone(), ty);
                }
            }
        }
        result
    }

    pub fn get_catalog(&self) -> CatalogResponse {
        CatalogResponse {
            objects: self.list_objects(),
            tables: self.list_tables(),
            cds_views: self.views.keys().cloned().collect(),
            rap_entities: vec!["ZI_PRODUCT".to_string(), "ZC_PRODUCT".to_string()],
            database: "ABAPC local relational fixture store".to_string(),
            is_sap_backend: false,
        }
    }

    pub fn activate_cds(&mut self, source_text: &str) -> Result<CdsView, String> {
        let caps = CDS_REGEX
            .captures(source_text)
            .ok_or_else(|| "CDS subset expects: define view entity <name> as select from <table> { ... }.".to_string())?;

        let name = caps.get(1).unwrap().as_str().to_ascii_uppercase();
        let table = caps.get(2).unwrap().as_str().to_ascii_uppercase();

        if self.views.contains_key(&name) {
            return Err(format!("CDS view '{}' is already activated. Use a new name or replace it explicitly.", name));
        }

        let schema = self
            .table_schema(&table)
            .ok_or_else(|| format!("CDS source '{}' is not in the local DDIC catalogue.", table))?;

        let match_pos = caps.get(0).unwrap().start();
        let open_brace = source_text[match_pos..]
            .find('{')
            .map(|p| match_pos + p)
            .ok_or_else(|| "CDS projection must contain an opening { brace.".to_string())?;
        let close_brace = source_text[open_brace + 1..]
            .find('}')
            .map(|p| open_brace + 1 + p)
            .ok_or_else(|| "CDS projection must contain a closing } brace.".to_string())?;

        let projection = &source_text[open_brace + 1..close_brace];
        let field_parts: Vec<&str> = projection.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if field_parts.is_empty() {
            return Err("CDS projection must include at least one field.".to_string());
        }

        let mut fields = Vec::new();
        for raw in field_parts {
            let m = CDS_FIELD_REGEX.captures(raw).ok_or_else(|| {
                format!(
                    "Unsupported CDS projection expression '{}'. This release supports direct field projections with optional aliases.",
                    raw
                )
            })?;
            let is_key = m.get(1).is_some();
            let source_field = m.get(2).unwrap().as_str().to_ascii_uppercase();
            let alias = m
                .get(3)
                .map(|a| a.as_str().to_ascii_uppercase())
                .unwrap_or_else(|| source_field.clone());

            let schema_fields = schema.fields.as_ref().ok_or("No schema fields available.")?;
            if !schema_fields.iter().any(|f| f.name.eq_ignore_ascii_case(&source_field)) {
                return Err(format!("CDS field '{}' does not exist in '{}'.", source_field, table));
            }
            fields.push(CdsField {
                source: source_field,
                alias,
                key: is_key,
            });
        }

        if !fields.iter().any(|f| f.key) {
            return Err("This local CDS subset requires at least one key field.".to_string());
        }

        let annotations: Vec<String> = ANNOTATION_REGEX
            .captures_iter(source_text)
            .map(|c| format!("{}: {}", c.get(1).unwrap().as_str(), c.get(2).unwrap().as_str().trim()))
            .collect();

        let view = CdsView {
            name: name.clone(),
            source: table,
            fields,
            annotations,
            source_text: source_text.to_string(),
            active: true,
        };
        self.views.insert(name, view.clone());
        Ok(view)
    }

    pub fn replace_cds(&mut self, source_text: &str) -> Result<CdsView, String> {
        let re = Regex::new(r"(?i)\bdefine\s+(?:root\s+)?view\s+entity\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap();
        let m = re
            .captures(source_text)
            .ok_or_else(|| "Could not identify CDS view entity name.".to_string())?;
        let name = m.get(1).unwrap().as_str().to_ascii_uppercase();
        self.views.remove(&name);
        self.activate_cds(source_text)
    }

    pub fn list_cds_views(&self) -> Vec<CdsView> {
        self.views.values().cloned().collect()
    }

    pub fn query_cds(&self, name: &str, filter: &HashMap<String, Value>) -> Result<Vec<Value>, String> {
        let view = self
            .views
            .get(&name.to_ascii_uppercase())
            .ok_or_else(|| format!("CDS view '{}' is not active in the local repository.", name))?;

        let rows = self.table_rows(&view.source)?;
        let filtered = rows.into_iter().filter(|row| {
            filter.iter().all(|(k, v)| {
                let rf = field_get_platform(row, k);
                crate::compiler::vm::format_value(&rf) == crate::compiler::vm::format_value(v)
            })
        });

        let mut projected = Vec::new();
        for row in filtered {
            let mut map = serde_json::Map::new();
            for f in &view.fields {
                let v = field_get_platform(&row, &f.source);
                map.insert(f.alias.clone(), v);
            }
            projected.push(Value::Object(map));
        }
        Ok(projected)
    }

    pub fn rap_read(
        &self,
        entity: &str,
        session_id: &str,
        filter: &HashMap<String, Value>,
    ) -> Result<Vec<Value>, String> {
        let table = self.rap_table(entity)?;
        let rows = self.working_rows(&table, session_id);
        let filtered = rows
            .into_iter()
            .filter(|row| {
                filter.iter().all(|(k, v)| {
                    let rf = field_get_platform(row, k);
                    crate::compiler::vm::format_value(&rf) == crate::compiler::vm::format_value(v)
                })
            })
            .collect();
        Ok(filtered)
    }

    pub fn rap_modify(
        &mut self,
        entity: &str,
        session_id: &str,
        operation: &str,
        payload: &HashMap<String, Value>,
    ) -> Result<RapModifyResult, String> {
        let table = self.rap_table(entity)?;
        let rows = self.working_rows(&table, session_id);
        let key = payload
            .get("PRODUCT_ID")
            .or_else(|| payload.get("product_id"))
            .map(|v| crate::compiler::vm::format_value(v))
            .unwrap_or_default();

        if key.is_empty() {
            return Err("PRODUCT_ID is required for the local RAP practice entity.".to_string());
        }

        let op: RapOperation;
        if operation == "CREATE" {
            if rows.iter().any(|r| field_get_platform(r, "PRODUCT_ID").as_str().map(|s| s == key).unwrap_or(false)) {
                return Err(format!("Product '{}' already exists.", key));
            }
            let name_str = payload
                .get("PRODUCT_NAME")
                .or_else(|| payload.get("product_name"))
                .map(|v| crate::compiler::vm::format_value(v))
                .unwrap_or_default();
            if name_str.trim().is_empty() {
                return Err("PRODUCT_NAME is required.".to_string());
            }
            let price = payload
                .get("PRICE")
                .or_else(|| payload.get("price"))
                .map(|v| crate::compiler::vm::format_value(v).parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);
            if price < 0.0 {
                return Err("PRICE must be a non-negative number.".to_string());
            }
            let cat = payload
                .get("CATEGORY")
                .or_else(|| payload.get("category"))
                .map(|v| crate::compiler::vm::format_value(v))
                .unwrap_or_default();
            let curr = payload
                .get("CURRENCY")
                .or_else(|| payload.get("currency"))
                .map(|v| crate::compiler::vm::format_value(v))
                .unwrap_or_else(|| "INR".to_string());

            let mut d = serde_json::Map::new();
            d.insert("PRODUCT_ID".to_string(), serde_json::json!(key));
            d.insert("PRODUCT_NAME".to_string(), serde_json::json!(name_str));
            d.insert("CATEGORY".to_string(), serde_json::json!(cat));
            d.insert("PRICE".to_string(), serde_json::json!(price));
            d.insert("CURRENCY".to_string(), serde_json::json!(curr));

            op = RapOperation {
                kind: "CREATE".to_string(),
                key: key.clone(),
                data: Some(Value::Object(d)),
            };
        } else if operation == "UPDATE" {
            let existing = rows
                .iter()
                .find(|r| field_get_platform(r, "PRODUCT_ID").as_str().map(|s| s == key).unwrap_or(false))
                .ok_or_else(|| format!("Product '{}' does not exist.", key))?;

            let mut data = match existing {
                Value::Object(m) => m.clone(),
                _ => serde_json::Map::new(),
            };
            for (k, v) in payload {
                data.insert(k.to_ascii_uppercase(), v.clone());
            }
            if let Some(p) = data.get("PRICE") {
                let pr = crate::compiler::vm::format_value(p).parse::<f64>().unwrap_or(0.0);
                if pr < 0.0 {
                    return Err("PRICE must be a non-negative number.".to_string());
                }
            }
            op = RapOperation {
                kind: "UPDATE".to_string(),
                key: key.clone(),
                data: Some(Value::Object(data)),
            };
        } else {
            if !rows.iter().any(|r| field_get_platform(r, "PRODUCT_ID").as_str().map(|s| s == key).unwrap_or(false)) {
                return Err(format!("Product '{}' does not exist.", key));
            }
            op = RapOperation {
                kind: "DELETE".to_string(),
                key: key.clone(),
                data: None,
            };
        }

        let session_ops = self.sessions.entry(session_id.to_string()).or_default();
        session_ops.push(op);
        let staged = session_ops.len();

        let mapped = if operation == "CREATE" {
            let mut m = serde_json::Map::new();
            m.insert("PRODUCT_ID".to_string(), serde_json::json!(key));
            Some(Value::Object(m))
        } else {
            None
        };

        Ok(RapModifyResult {
            staged,
            mapped,
            reported: format!("{} staged for {}; call COMMIT ENTITIES to persist.", operation, key),
        })
    }

    pub fn rap_commit(&mut self, session_id: &str) -> Result<RapCommitResult, String> {
        let ops = self.sessions.remove(session_id).unwrap_or_default();
        let mut candidate = self.tables.get("ZPRODUCT").cloned().unwrap_or_default();

        for op in &ops {
            let idx = candidate.iter().position(|r| {
                field_get_platform(r, "PRODUCT_ID")
                    .as_str()
                    .map(|s| s == op.key)
                    .unwrap_or(false)
            });

            if op.kind == "CREATE" {
                if idx.is_some() {
                    return Err(format!("Commit conflict: product '{}' already exists.", op.key));
                }
                candidate.push(op.data.clone().unwrap_or(Value::Null));
            } else if op.kind == "UPDATE" {
                match idx {
                    Some(i) => candidate[i] = op.data.clone().unwrap_or(Value::Null),
                    None => return Err(format!("Commit conflict: product '{}' was removed.", op.key)),
                }
            } else {
                match idx {
                    Some(i) => {
                        candidate.remove(i);
                    }
                    None => return Err(format!("Commit conflict: product '{}' was removed.", op.key)),
                }
            }
        }

        let committed = ops.len();
        let rows = candidate.len();
        self.tables.insert("ZPRODUCT".to_string(), candidate);

        Ok(RapCommitResult {
            committed,
            rows,
            reported: format!("{} local RAP operation(s) committed atomically.", committed),
        })
    }

    pub fn rap_rollback(&mut self, session_id: &str) -> RapRollbackResult {
        let count = self.sessions.remove(session_id).map(|s| s.len()).unwrap_or(0);
        RapRollbackResult {
            rolled_back: count,
            reported: format!("{} pending local RAP operation(s) rolled back.", count),
        }
    }

    pub fn rap_pending(&self, session_id: &str) -> Vec<RapOperation> {
        self.sessions.get(session_id).cloned().unwrap_or_default()
    }

    pub fn export_state(&self) -> Value {
        serde_json::json!({
            "tables": self.tables,
            "views": self.list_cds_views(),
            "objects": self.custom_objects,
        })
    }

    pub fn import_state(&mut self, state: &Value) {
        if let Some(tables_obj) = state.get("tables").and_then(|v| v.as_object()) {
            for (name, rows) in tables_obj {
                if let Some(arr) = rows.as_array() {
                    let key = name.to_ascii_uppercase();
                    if self.tables.contains_key(&key) {
                        self.tables.insert(key, arr.clone());
                    }
                }
            }
        }
        if let Some(objects_arr) = state.get("objects").and_then(|v| v.as_array()) {
            for obj in objects_arr {
                if let Ok(loc_obj) = serde_json::from_value::<LocalObject>(obj.clone()) {
                    if self.get_object(&loc_obj.name).is_none() {
                        let init_rows = state
                            .get("tables")
                            .and_then(|t| t.get(&loc_obj.name))
                            .and_then(|r| r.as_array())
                            .cloned();
                        let _ = self.create_object(
                            &loc_obj.name,
                            &loc_obj.kind,
                            Some(&loc_obj.description),
                            loc_obj.base_type.as_deref(),
                            loc_obj.fields,
                            init_rows,
                        );
                    }
                }
            }
        }
        if let Some(views_arr) = state.get("views").and_then(|v| v.as_array()) {
            for view in views_arr {
                if let Some(source_text) = view.get("sourceText").and_then(|s| s.as_str()) {
                    let _ = self.replace_cds(source_text);
                }
            }
        }
    }

    fn rap_table(&self, entity: &str) -> Result<String, String> {
        let name = entity.to_ascii_uppercase();
        if !matches!(name.as_str(), "ZI_PRODUCT" | "ZC_PRODUCT" | "ZR_PRODUCT" | "PRODUCT") {
            return Err(format!("Unknown local RAP training entity '{}'. Supported entity: ZI_PRODUCT.", entity));
        }
        Ok("ZPRODUCT".to_string())
    }

    fn working_rows(&self, table: &str, session_id: &str) -> Vec<Value> {
        let mut rows = self.tables.get(table).cloned().unwrap_or_default();
        if let Some(ops) = self.sessions.get(session_id) {
            for op in ops {
                let idx = rows.iter().position(|r| {
                    field_get_platform(r, "PRODUCT_ID")
                        .as_str()
                        .map(|s| s == op.key)
                        .unwrap_or(false)
                });
                if op.kind == "CREATE" {
                    if let Some(d) = &op.data {
                        rows.push(d.clone());
                    }
                } else if op.kind == "UPDATE" {
                    if let Some(i) = idx {
                        if let Some(d) = &op.data {
                            rows[i] = d.clone();
                        }
                    }
                } else if op.kind == "DELETE" {
                    if let Some(i) = idx {
                        rows.remove(i);
                    }
                }
            }
        }
        rows
    }

    fn field_type(&self, table: &str, field: &str) -> String {
        self.table_schema(table)
            .and_then(|s| s.fields)
            .and_then(|fs| fs.into_iter().find(|f| f.name.eq_ignore_ascii_case(field)))
            .map(|f| f.field_type)
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }
}

pub struct RapModifyResult {
    pub staged: usize,
    pub mapped: Option<Value>,
    pub reported: String,
}

pub struct RapCommitResult {
    pub committed: usize,
    pub rows: usize,
    pub reported: String,
}

pub struct RapRollbackResult {
    pub rolled_back: usize,
    pub reported: String,
}

fn field_get_platform(base: &Value, field: &str) -> Value {
    match base {
        Value::Object(map) => {
            map.get(&field.to_ascii_uppercase())
                .or_else(|| map.get(field))
                .cloned()
                .unwrap_or(Value::Null)
        }
        _ => Value::Null,
    }
}
