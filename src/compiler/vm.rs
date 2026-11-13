use super::ast::Expr;
use super::ir::{initial_value_for_type, ClassIR, MethodIR, Op, OutputBinding, ProgramIR, Value, VarIR};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Clone, Debug)]
struct ExceptionHandler {
    catch_type: String,
    target: usize,
    stack_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMResult {
    pub output: Vec<String>,
    pub variables: HashMap<String, Value>,
    pub steps: usize,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u64,
    #[serde(rename = "selectedRows")]
    pub selected_rows: usize,
}

#[derive(Clone)]
pub struct EmlPendingOp {
    pub operation: String,
    pub key: String,
    pub data: Option<Value>,
}

#[derive(Clone, Default)]
pub struct SharedExecution {
    pub output: Vec<String>,
    pub classes: HashMap<String, ClassIR>,
    pub static_data: HashMap<String, HashMap<String, Value>>,
    pub class_initialized: HashSet<String>,
    pub steps: usize,
    pub selected_rows: usize,
    pub depth: usize,
    pub eml_pending: Vec<EmlPendingOp>,
    pub newline_required: bool,
    pub globals: HashMap<String, Value>,
}

pub struct IteratorState {
    pub table_name: Option<String>,
    pub values: Vec<Value>,
    pub index: usize,
    pub target_fs: Option<String>,
}

pub fn default_vm_database() -> HashMap<String, Vec<Value>> {
    let mut db = HashMap::new();
    db.insert(
        "SCARR".to_string(),
        vec![
            serde_json::json!({"CARRID":"LH","CARRNAME":"Lufthansa","CURRCODE":"EUR"}),
            serde_json::json!({"CARRID":"AA","CARRNAME":"American Airlines","CURRCODE":"USD"}),
            serde_json::json!({"CARRID":"SQ","CARRNAME":"Singapore Airlines","CURRCODE":"SGD"}),
        ],
    );
    db.insert(
        "SPFLI".to_string(),
        vec![
            serde_json::json!({"CARRID":"LH","CONNID":400,"COUNTRYFR":"DE","CITYFROM":"Frankfurt","COUNTRYTO":"US","CITYTO":"New York"}),
            serde_json::json!({"CARRID":"LH","CONNID":401,"COUNTRYFR":"DE","CITYFROM":"Munich","COUNTRYTO":"JP","CITYTO":"Tokyo"}),
            serde_json::json!({"CARRID":"SQ","CONNID":88,"COUNTRYFR":"SG","CITYFROM":"Singapore","COUNTRYTO":"IN","CITYTO":"Mumbai"}),
        ],
    );
    db.insert(
        "SFLIGHT".to_string(),
        vec![
            serde_json::json!({"CARRID":"LH","CONNID":400,"FLDATE":"2026-10-01","PRICE":520}),
            serde_json::json!({"CARRID":"LH","CONNID":401,"FLDATE":"2026-10-02","PRICE":610}),
            serde_json::json!({"CARRID":"SQ","CONNID":88,"FLDATE":"2026-10-03","PRICE":430}),
        ],
    );
    db.insert(
        "ZPRODUCT".to_string(),
        vec![
            serde_json::json!({"PRODUCT_ID":"P-100","PRODUCT_NAME":"Keyboard","CATEGORY":"ACCESSORY","PRICE":1200,"CURRENCY":"INR"}),
            serde_json::json!({"PRODUCT_ID":"P-200","PRODUCT_NAME":"Mouse","CATEGORY":"ACCESSORY","PRICE":700,"CURRENCY":"INR"}),
            serde_json::json!({"PRODUCT_ID":"P-300","PRODUCT_NAME":"Monitor","CATEGORY":"DISPLAY","PRICE":15000,"CURRENCY":"INR"}),
        ],
    );
    db.insert(
        "USR02".to_string(),
        vec![
            serde_json::json!({"CLIENT":"001","UNAME":"DEVELOPER"}),
            serde_json::json!({"CLIENT":"001","UNAME":"BCUSER"}),
            serde_json::json!({"CLIENT":"001","UNAME":"GAURAV"}),
        ],
    );
    db
}

pub struct VM {
    pub db: HashMap<String, Vec<Value>>,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            db: default_vm_database(),
        }
    }
}

impl VM {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_database(db: HashMap<String, Vec<Value>>) -> Self {
        Self { db }
    }

    pub fn execute(
        &mut self,
        ir: &ProgramIR,
        max_steps: usize,
        parameter_values: &HashMap<String, Value>,
        stop_before_apply_parameters: bool,
        skip_obligatory: bool,
        shared_opt: Option<&mut SharedExecution>,
        initial_vars: Option<&HashMap<String, Value>>,
    ) -> Result<VMResult, String> {
        let started = Instant::now();
        let mut vars: HashMap<String, Value> = HashMap::new();
        let mut stack: Vec<Value> = Vec::new();

        let mut default_shared = SharedExecution {
            output: Vec::new(),
            classes: ir.classes.clone(),
            static_data: HashMap::new(),
            class_initialized: HashSet::new(),
            steps: 0,
            selected_rows: 0,
            depth: 0,
            eml_pending: Vec::new(),
            newline_required: false,
            globals: HashMap::new(),
        };

        let shared = match shared_opt {
            Some(s) => s,
            None => &mut default_shared,
        };

        let mut iterators: Vec<IteratorState> = Vec::new();
        let mut provided_parameters: HashSet<String> = HashSet::new();

        for v in &ir.vars {
            vars.insert(v.name.to_ascii_uppercase(), initial_value_for_type(&v.type_ref));
        }

        let mut sy = serde_json::Map::new();
        sy.insert("SUBRC".to_string(), serde_json::json!(0));
        sy.insert("TABIX".to_string(), serde_json::json!(0));
        sy.insert("DBCNT".to_string(), serde_json::json!(0));
        vars.insert("SY".to_string(), Value::Object(sy));
        vars.insert("SY-SUBRC".to_string(), serde_json::json!(0));
        vars.insert("SY-TABIX".to_string(), serde_json::json!(0));
        vars.insert("SY-DBCNT".to_string(), serde_json::json!(0));

        if let Some(init_vars) = initial_vars {
            for (k, v) in init_vars {
                vars.insert(k.to_ascii_uppercase(), v.clone());
            }
        }

        let mut pc = 0;
        let mut return_flag = false;
        let mut handlers: Vec<ExceptionHandler> = Vec::new();
        let mut alias_map: HashMap<String, String> = HashMap::new();
        let mut table_alias_map: HashMap<String, (String, usize)> = HashMap::new();

        macro_rules! try_op {
            ($expr:expr) => {
                match $expr {
                    Ok(v) => v,
                    Err(e) => {
                        let err_str = e.to_string();
                        let mut handled = false;
                        while let Some(h) = handlers.pop() {
                            let matches_type = h.catch_type.eq_ignore_ascii_case("CX_ROOT")
                                || h.catch_type.eq_ignore_ascii_case(&err_str)
                                || err_str.to_ascii_uppercase().contains(&h.catch_type)
                                || (err_str == "CX_SY_ZERODIVIDE" && h.catch_type.eq_ignore_ascii_case("CX_SY_ZERODIVIDE"))
                                || (err_str.starts_with("Division by zero") && h.catch_type.eq_ignore_ascii_case("CX_SY_ZERODIVIDE"));
                            if matches_type {
                                stack.truncate(h.stack_depth);
                                pc = h.target;
                                handled = true;
                                break;
                            }
                        }
                        if !handled {
                            return Err(err_str);
                        }
                        continue;
                    }
                }
            };
        }

        while pc < ir.ops.len() {
            let op = &ir.ops[pc];
            if stop_before_apply_parameters && matches!(op, Op::ApplyParameters) {
                break;
            }

            shared.steps += 1;
            if shared.steps > max_steps {
                return Err(format!("Execution step limit exceeded ({}); possible infinite loop.", max_steps));
            }

            match op {
                Op::Push { value } => {
                    stack.push(value.clone());
                    pc += 1;
                }
                Op::Load { name } => {
                    let key = name.to_ascii_uppercase();
                    let v = if let Some(aliased) = alias_map.get(&key) {
                        get_var(&vars, shared, aliased)
                    } else if let Some((tbl, row_idx)) = table_alias_map.get(&key) {
                        if let Some(Value::Array(arr)) = vars.get(tbl) {
                            arr.get(*row_idx).cloned().unwrap_or(Value::Null)
                        } else {
                            get_var(&vars, shared, name)
                        }
                    } else {
                        get_var(&vars, shared, name)
                    };
                    stack.push(v);
                    pc += 1;
                }
                Op::Store { name } => {
                    let v = stack.pop().ok_or("VM stack underflow on STORE")?;
                    let key = name.to_ascii_uppercase();
                    if let Some(aliased) = alias_map.get(&key).cloned() {
                        set_var(&mut vars, shared, &aliased, v.clone());
                        vars.insert(key, v);
                    } else if let Some((tbl, row_idx)) = table_alias_map.get(&key).cloned() {
                        if let Some(Value::Array(arr)) = vars.get_mut(&tbl) {
                            if row_idx < arr.len() {
                                arr[row_idx] = v.clone();
                            }
                        }
                        vars.insert(key, v);
                    } else {
                        set_var(&mut vars, shared, name, v);
                    }
                    pc += 1;
                }
                Op::LoadStatic { class_name, name } => {
                    let found = find_static_value(shared, class_name, name);
                    if !found.found {
                        return Err(format!("Unknown static attribute {}=>{}.", class_name, name));
                    }
                    self.ensure_class_initialized(class_name, shared, max_steps)?;
                    ensure_static_data(shared, class_name);
                    let val = shared
                        .static_data
                        .get(&found.owner)
                        .and_then(|m| m.get(&name.to_ascii_uppercase()))
                        .cloned()
                        .unwrap_or(serde_json::json!(0));
                    stack.push(val);
                    pc += 1;
                }
                Op::StoreStatic { class_name, name } => {
                    let v = stack.pop().ok_or("VM stack underflow on static attribute store")?;
                    let found = find_static_value(shared, class_name, name);
                    if !found.found {
                        return Err(format!("Unknown static attribute {}=>{}.", class_name, name));
                    }
                    self.ensure_class_initialized(class_name, shared, max_steps)?;
                    ensure_static_data(shared, class_name);
                    let class_data = shared.static_data.entry(found.owner).or_default();
                    class_data.insert(name.to_ascii_uppercase(), v);
                    pc += 1;
                }
                Op::LoadField { field } => {
                    let base = stack.pop().unwrap_or(Value::Null);
                    stack.push(field_get(&base, field));
                    pc += 1;
                }
                Op::StoreField { field } => {
                    let mut base = stack.pop().ok_or("VM stack underflow on field store")?;
                    let v = stack.pop().ok_or("VM stack underflow on field store")?;
                    field_set(&mut base, field, v)?;
                    stack.push(base);
                    pc += 1;
                }
                Op::IndexGet => {
                    let index = stack.pop().unwrap_or(Value::Null);
                    let base = stack.pop().unwrap_or(Value::Null);
                    stack.push(index_get(&base, &index));
                    pc += 1;
                }
                Op::IndexSet => {
                    let index = stack.pop().unwrap_or(Value::Null);
                    let mut base = stack.pop().unwrap_or(Value::Null);
                    let v = stack.pop().unwrap_or(Value::Null);
                    index_set(&mut base, &index, v)?;
                    stack.push(base);
                    pc += 1;
                }
                Op::Neg => {
                    let v = stack.pop().unwrap_or(Value::Null);
                    let n = try_op!(to_number(&v));
                    if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                        stack.push(serde_json::json!(-(n as i64)));
                    } else {
                        stack.push(serde_json::json!(-n));
                    }
                    pc += 1;
                }
                Op::Not => {
                    let v = stack.pop().unwrap_or(Value::Null);
                    stack.push(serde_json::json!(!truthy(&v)));
                    pc += 1;
                }
                Op::Binary { op } => {
                    let r = stack.pop().unwrap_or(Value::Null);
                    let l = stack.pop().unwrap_or(Value::Null);
                    let res = try_op!(binary_op(op, &l, &r));
                    stack.push(res);
                    pc += 1;
                }
                Op::Call { name, argc } => {
                    let take_count = (*argc).min(stack.len());
                    let args: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                    let special = iterator_call(name, &args, &mut iterators, &mut vars, &mut table_alias_map);
                    if special.handled {
                        if name.eq_ignore_ascii_case("__iter_value") {
                            shared.newline_required = true;
                        }
                        if special.push {
                            stack.push(special.value.unwrap_or(Value::Null));
                        }
                    } else {
                        stack.push(call_builtin(name, &args)?);
                    }
                    pc += 1;
                }
                Op::Constructor { fields } => {
                    let take_count = fields.len().min(stack.len());
                    let values: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                    let mut obj = serde_json::Map::new();
                    for (i, f) in fields.iter().enumerate() {
                        let val = values.get(i).cloned().unwrap_or(Value::Null);
                        obj.insert(f.to_ascii_uppercase(), val);
                    }
                    stack.push(Value::Object(obj));
                    pc += 1;
                }
                Op::TableConstructor { count } => {
                    let take_count = (*count).min(stack.len());
                    let items: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                    stack.push(Value::Array(items));
                    pc += 1;
                }
                Op::Raise { exception_type } => {
                    try_op!(Err::<(), _>(exception_type.clone()));
                    pc += 1;
                }
                Op::NewObject { class_name, positional_count, named } => {
                    if !shared.classes.contains_key(&class_name.to_ascii_uppercase()) {
                        return Err(format!("Class '{}' is not available in the compiled program.", class_name));
                    }
                    let class_ir = shared.classes.get(&class_name.to_ascii_uppercase()).unwrap();
                    if class_ir.is_abstract {
                        return Err(format!("Cannot instantiate abstract class '{}'.", class_ir.name));
                    }
                    self.ensure_class_initialized(class_name, shared, max_steps)?;
                    ensure_static_data(shared, class_name);

                    let arg_count = positional_count + named.len();
                    let take_count = arg_count.min(stack.len());
                    let values: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                    let positional = values.get(..*positional_count).unwrap_or(&[]).to_vec();
                    let mut named_map: HashMap<String, Value> = HashMap::new();
                    for (i, name) in named.iter().enumerate() {
                        named_map.insert(
                            name.to_ascii_uppercase(),
                            values.get(*positional_count + i).cloned().unwrap_or(Value::Null),
                        );
                    }

                    let mut object = initialize_object(shared, class_name);
                    let ctor = resolve_runtime_method(shared, class_name, "CONSTRUCTOR");
                    if ctor.is_some() {
                        let res = self.invoke_user_method(&object, "CONSTRUCTOR", &positional, &named_map, shared, max_steps)?;
                        if let Some(me) = res.receiver {
                            object = me;
                        }
                    } else if arg_count > 0 {
                        return Err(format!("Class '{}' has no constructor accepting arguments.", class_name));
                    }
                    stack.push(object);
                    pc += 1;
                }
                Op::CallMethodExpr { method, positional_count, named, output_bindings } => {
                    let arg_count = positional_count + named.len();
                    let take_count = (arg_count + 1).min(stack.len());
                    let values: Vec<Value> = stack.drain(stack.len() - take_count..).collect();

                    let receiver = values.first().cloned().unwrap_or(Value::Null);
                    let positional = values.get(1..1 + *positional_count).unwrap_or(&[]).to_vec();
                    let mut named_map = HashMap::new();
                    for (i, name) in named.iter().enumerate() {
                        named_map.insert(
                            name.to_ascii_uppercase(),
                            values.get(1 + positional_count + i).cloned().unwrap_or(Value::Null),
                        );
                    }

                    if let Value::Object(obj) = &receiver {
                        if obj.get("__STATIC_CONTEXT").and_then(|v| v.as_bool()).unwrap_or(false) {
                            if let Some(Value::String(cname)) = obj.get("__CLASS") {
                                self.ensure_class_initialized(cname, shared, max_steps)?;
                            }
                        }
                    }

                    shared.globals.extend(vars.clone());
                    let result = try_op!(self.invoke_user_method(&receiver, method, &positional, &named_map, shared, max_steps));
                    vars.extend(shared.globals.clone());
                    apply_output_bindings(&result.outputs, output_bindings, &mut vars);
                    stack.push(result.value);
                    pc += 1;
                }
                Op::Write { groups, newline } => {
                    let total: usize = groups.iter().sum();
                    let take_count = total.min(stack.len());
                    let vals: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                    let mut offset = 0;
                    for (g_idx, count) in groups.iter().enumerate() {
                        let slice = &vals[offset..offset + count];
                        let line = slice.iter().map(format_value).collect::<Vec<_>>().join(" ");
                        let is_new = (*newline && g_idx == 0) || g_idx > 0 || shared.newline_required || shared.output.is_empty();
                        shared.newline_required = false;
                        if is_new {
                            shared.output.push(line);
                        } else if let Some(last) = shared.output.last_mut() {
                            if !last.is_empty() && !line.is_empty() {
                                last.push(' ');
                            }
                            last.push_str(&line);
                        }
                        offset += count;
                    }
                    pc += 1;
                }
                Op::JumpIfFalse { target } => {
                    let v = stack.pop().unwrap_or(Value::Null);
                    if truthy(&v) {
                        pc += 1;
                    } else {
                        pc = *target as usize;
                    }
                }
                Op::Jump { target } => {
                    pc = *target as usize;
                }
                Op::Declare { name } => {
                    let key = name.to_ascii_uppercase();
                    if !vars.contains_key(&key) {
                        vars.insert(key, serde_json::json!(0));
                    }
                    pc += 1;
                }
                Op::ApplyParameters => {
                    for v in &ir.vars {
                        if !v.parameter.unwrap_or(false) {
                            continue;
                        }
                        let key = v.name.to_ascii_uppercase();
                        let input_key = parameter_values.keys().find(|k| k.eq_ignore_ascii_case(&key));
                        let has_input = input_key.is_some();

                        if has_input {
                            let raw = &parameter_values[input_key.unwrap()];
                            let is_so = v.selection_options.unwrap_or(false);
                            let empty_range = is_so && match raw {
                                Value::Array(arr) => {
                                    arr.is_empty()
                                        || arr.iter().all(|r| {
                                            let low = r.get("low").or_else(|| r.get("LOW")).and_then(|x| x.as_str()).unwrap_or("").trim();
                                            let high = r.get("high").or_else(|| r.get("HIGH")).and_then(|x| x.as_str()).unwrap_or("").trim();
                                            low.is_empty() && high.is_empty()
                                        })
                                }
                                _ => true,
                            };

                            let is_ob = v.obligatory.unwrap_or(false);
                            if !skip_obligatory && is_ob {
                                if is_so && empty_range {
                                    return Err(format!("Parameter '{}' is obligatory.", v.name));
                                } else if !is_so && (raw.is_null() || raw.as_str().map(|s| s.trim().is_empty()).unwrap_or(false)) {
                                    return Err(format!("Parameter '{}' is obligatory.", v.name));
                                }
                            }

                            if is_so {
                                validate_selection_ranges(raw, v)?;
                            }
                            vars.insert(key.clone(), coerce_parameter(v, raw, &v.name)?);
                            provided_parameters.insert(key);
                        } else if !skip_obligatory && v.obligatory.unwrap_or(false) && !v.has_default.unwrap_or(false) {
                            return Err(format!("Parameter '{}' is obligatory.", v.name));
                        }
                    }
                    pc += 1;
                }
                Op::StoreIfUnset { name } => {
                    let value = stack.pop().ok_or("VM stack underflow on STORE_IF_UNSET")?;
                    let key = name.to_ascii_uppercase();
                    if !provided_parameters.contains(&key) {
                        let meta = ir.vars.iter().find(|x| x.name.eq_ignore_ascii_case(&key));
                        let is_so = meta.and_then(|m| m.selection_options).unwrap_or(false);
                        let final_val = if is_so {
                            coerce_parameter(meta.unwrap(), &value, &meta.unwrap().name)?
                        } else {
                            value
                        };
                        set_var(&mut vars, shared, name, final_val);
                    }
                    pc += 1;
                }
                Op::Translate { mode, has_mask } => {
                    let mask = if *has_mask {
                        stack.pop().map(|v| format_value(&v)).unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let original = stack.pop().map(|v| format_value(&v)).unwrap_or_default();
                    let changed = match mode.as_str() {
                        "upper" => original.to_ascii_uppercase(),
                        "lower" => original.to_ascii_lowercase(),
                        _ => {
                            let mut map = HashMap::new();
                            let mask_chars: Vec<char> = mask.chars().collect();
                            let mut idx = 0;
                            while idx + 1 < mask_chars.len() {
                                map.insert(mask_chars[idx], mask_chars[idx + 1]);
                                idx += 2;
                            }
                            original.chars().map(|ch| map.get(&ch).copied().unwrap_or(ch)).collect()
                        }
                    };
                    stack.push(Value::String(changed));
                    pc += 1;
                }
                Op::Concatenate { count, has_separator, respecting_blanks } => {
                    let separator = if *has_separator {
                        format_value(&stack.pop().unwrap_or(Value::Null))
                    } else {
                        String::new()
                    };
                    let take_count = (*count).min(stack.len());
                    let vals: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                    let cooked: Vec<String> = vals
                        .iter()
                        .map(|v| {
                            let s = format_value(v);
                            if *respecting_blanks {
                                s
                            } else {
                                s.trim_end().to_string()
                            }
                        })
                        .collect();
                    stack.push(Value::String(cooked.join(&separator)));
                    pc += 1;
                }
                Op::Split { target_count, into_table } => {
                    let separator = stack.pop().map(|v| format_value(&v)).unwrap_or_default();
                    let source = stack.pop().map(|v| format_value(&v)).unwrap_or_default();
                    if separator.is_empty() {
                        return Err("SPLIT separator must not be empty.".to_string());
                    }
                    let all: Vec<String> = source.split(&separator).map(|s| s.to_string()).collect();
                    let parts = if *into_table {
                        Value::Array(all.into_iter().map(Value::String).collect())
                    } else if *target_count == 0 {
                        Value::Array(Vec::new())
                    } else {
                        let mut arr = Vec::with_capacity(*target_count);
                        for i in 0..*target_count {
                            if i == *target_count - 1 {
                                arr.push(Value::String(all[i..].join(&separator)));
                            } else {
                                arr.push(Value::String(all.get(i).cloned().unwrap_or_default()));
                            }
                        }
                        Value::Array(arr)
                    };
                    stack.push(parts);
                    pc += 1;
                }
                Op::Condense { no_gaps } => {
                    let old = stack.pop().map(|v| format_value(&v)).unwrap_or_default();
                    let res = if *no_gaps {
                        old.split_whitespace().collect::<String>()
                    } else {
                        old.split_whitespace().collect::<Vec<_>>().join(" ")
                    };
                    stack.push(Value::String(res));
                    pc += 1;
                }
                Op::Message => {
                    let msg_type = stack.pop().map(|v| format_value(&v).to_ascii_uppercase()).unwrap_or_else(|| "S".to_string());
                    let message = stack.pop().map(|v| format_value(&v)).unwrap_or_default();
                    if matches!(msg_type.as_str(), "E" | "A" | "X") {
                        return Err(format!("MESSAGE {}: {}", msg_type, message));
                    }
                    shared.output.push(message);
                    pc += 1;
                }
                Op::Append => {
                    let table = stack.pop().ok_or("VM stack underflow on APPEND table")?;
                    let value = stack.pop().ok_or("VM stack underflow on APPEND value")?;
                    match table {
                        Value::Array(mut arr) => {
                            arr.push(value);
                            stack.push(Value::Array(arr));
                        }
                        _ => return Err("APPEND target is not a table".to_string()),
                    }
                    pc += 1;
                }
                Op::Insert => {
                    let table = stack.pop().ok_or("VM stack underflow on INSERT table")?;
                    let value = stack.pop().ok_or("VM stack underflow on INSERT value")?;
                    match table {
                        Value::Array(mut arr) => {
                            arr.push(value);
                            stack.push(Value::Array(arr));
                        }
                        _ => return Err("INSERT target is not a table".to_string()),
                    }
                    pc += 1;
                }
                Op::ReadTable { table_name, target, index, key } => {
                    let mut idx: i64 = -1;
                    let table = if *index {
                        let n_val = stack.pop().unwrap_or(Value::Null);
                        let n = to_number(&n_val)? as i64;
                        let tbl = stack.pop().unwrap_or(Value::Null);
                        idx = n - 1;
                        tbl
                    } else if let Some(k_fields) = key {
                        let take_count = k_fields.len().min(stack.len());
                        let vals: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                        let tbl = stack.pop().unwrap_or(Value::Null);
                        let mut key_map = HashMap::new();
                        for (i, k) in k_fields.iter().enumerate() {
                            key_map.insert(k.to_ascii_uppercase(), vals.get(i).cloned().unwrap_or(Value::Null));
                        }
                        if let Value::Array(arr) = &tbl {
                            idx = arr.iter().position(|r| matches_record(r, &key_map)).map(|p| p as i64).unwrap_or(-1);
                        }
                        tbl
                    } else {
                        let tbl = stack.pop().unwrap_or(Value::Null);
                        if let Value::Array(arr) = &tbl {
                            idx = if !arr.is_empty() { 0 } else { -1 };
                        }
                        tbl
                    };

                    let arr = match &table {
                        Value::Array(a) => a,
                        _ => return Err("READ TABLE source is not a table".to_string()),
                    };

                    let found = if idx >= 0 && (idx as usize) < arr.len() {
                        Some(&arr[idx as usize])
                    } else {
                        None
                    };

                    if let Some(tgt) = target {
                        let record = found.cloned().unwrap_or_else(|| default_record(arr.first()));
                        set_var(&mut vars, shared, tgt, record.clone());
                        let tgt_key = tgt.to_ascii_uppercase();
                        if tgt.starts_with('<') && tgt.ends_with('>') {
                            if found.is_some() {
                                if let Some(ref tbl) = table_name {
                                    table_alias_map.insert(tgt_key.clone(), (tbl.to_ascii_uppercase(), idx as usize));
                                }
                            } else {
                                table_alias_map.remove(&tgt_key);
                                vars.remove(&tgt_key);
                            }
                        }
                    }

                    let subrc_val = if found.is_some() { 0 } else { 4 };
                    let tabix_val = if found.is_some() { idx + 1 } else { 0 };
                    vars.insert("SY-SUBRC".to_string(), serde_json::json!(subrc_val));
                    vars.insert("SY-TABIX".to_string(), serde_json::json!(tabix_val));
                    if let Some(Value::Object(sy_map)) = vars.get_mut("SY") {
                        sy_map.insert("SUBRC".to_string(), serde_json::json!(subrc_val));
                        sy_map.insert("TABIX".to_string(), serde_json::json!(tabix_val));
                    }
                    stack.push(Value::Bool(idx >= 0));
                    pc += 1;
                }
                Op::DeleteTable { index, key } => {
                    if *index {
                        let n_val = stack.pop().unwrap_or(Value::Null);
                        let n = (to_number(&n_val)? as i64) - 1;
                        let mut table = stack.pop().unwrap_or(Value::Null);
                        if let Value::Array(arr) = &mut table {
                            if n >= 0 && (n as usize) < arr.len() {
                                arr.remove(n as usize);
                            }
                            stack.push(table);
                        } else {
                            return Err("DELETE source is not a table".to_string());
                        }
                    } else if let Some(k_fields) = key {
                        let take_count = k_fields.len().min(stack.len());
                        let vals: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                        let mut table = stack.pop().unwrap_or(Value::Null);
                        let mut key_map = HashMap::new();
                        for (i, k) in k_fields.iter().enumerate() {
                            key_map.insert(k.to_ascii_uppercase(), vals.get(i).cloned().unwrap_or(Value::Null));
                        }
                        if let Value::Array(arr) = &mut table {
                            arr.retain(|r| !matches_record(r, &key_map));
                            stack.push(table);
                        } else {
                            return Err("DELETE source is not a table".to_string());
                        }
                    } else {
                        // pop and push table back
                        let table = stack.pop().unwrap_or(Value::Null);
                        stack.push(table);
                    }
                    pc += 1;
                }
                Op::Sort { keys } => {
                    let mut table = stack.pop().ok_or("SORT source is not a table")?;
                    if let Value::Array(arr) = &mut table {
                        arr.sort_by(|a, b| {
                            for k in keys {
                                let av = field_get(a, &k.name);
                                let bv = field_get(b, &k.name);
                                let cmp = compare_values(&av, &bv);
                                if cmp != std::cmp::Ordering::Equal {
                                    return if k.descending { cmp.reverse() } else { cmp };
                                }
                            }
                            std::cmp::Ordering::Equal
                        });
                        stack.push(table);
                    } else {
                        return Err("SORT source is not a table".to_string());
                    }
                    pc += 1;
                }
                Op::Describe { target } => {
                    let table = stack.pop().unwrap_or(Value::Null);
                    let count = match &table {
                        Value::Array(a) => a.len(),
                        _ => 0,
                    };
                    set_var(&mut vars, shared, target, serde_json::json!(count));
                    pc += 1;
                }
                Op::Select { source, fields, target, single, where_clause } => {
                    let table_name = source.to_ascii_uppercase();
                    let raw_rows = self.db.get(&table_name).cloned().unwrap_or_default();
                    let filtered_rows: Vec<Value> = raw_rows
                        .into_iter()
                        .filter(|r| {
                            if let Some(w) = where_clause {
                                eval_expr_where(w, r, &vars).unwrap_or(false)
                            } else {
                                true
                            }
                        })
                        .collect();

                    shared.selected_rows += filtered_rows.len();
                    let subrc_val = if !filtered_rows.is_empty() { 0 } else { 4 };
                    let dbcnt_val = filtered_rows.len();
                    vars.insert("SY-SUBRC".to_string(), serde_json::json!(subrc_val));
                    vars.insert("SY-DBCNT".to_string(), serde_json::json!(dbcnt_val));
                    if let Some(Value::Object(sy_map)) = vars.get_mut("SY") {
                        sy_map.insert("SUBRC".to_string(), serde_json::json!(subrc_val));
                        sy_map.insert("DBCNT".to_string(), serde_json::json!(dbcnt_val));
                    }

                    let mut projected_rows = Vec::new();
                    let is_wildcard = fields.is_empty() || (fields.len() == 1 && fields[0] == "*");

                    for mut row in filtered_rows {
                        if !is_wildcard {
                            let mut map = serde_json::Map::new();
                            for f in fields {
                                let val = field_get(&row, f);
                                map.insert(f.to_ascii_uppercase(), val);
                            }
                            row = Value::Object(map);
                        }
                        projected_rows.push(row);
                    }

                    if let Some(tgt) = target {
                        if *single {
                            let rec = projected_rows.first().cloned().unwrap_or(Value::Object(serde_json::Map::new()));
                            set_var(&mut vars, shared, tgt, rec);
                        } else {
                            set_var(&mut vars, shared, tgt, Value::Array(projected_rows));
                        }
                    } else {
                        for r in &projected_rows {
                            shared.output.push(format_value(r));
                        }
                    }
                    pc += 1;
                }
                Op::EmlRead { entity: _, filter_rows, result_target, failed_target, reported_target } => {
                    let value_count: usize = filter_rows.iter().map(|r| r.names.len()).sum();
                    let take_count = value_count.min(stack.len());
                    let values: Vec<Value> = stack.drain(stack.len() - take_count..).collect();

                    let mut filters = Vec::new();
                    let mut offset = 0;
                    for row in filter_rows {
                        let mut filter = HashMap::new();
                        for name in &row.names {
                            filter.insert(canonical_product_field(name), values.get(offset).cloned().unwrap_or(Value::Null));
                            offset += 1;
                        }
                        filters.push(filter);
                    }

                    let mut rows = effective_product_rows(&self.db, shared);
                    if !filters.is_empty() {
                        rows.retain(|row| filters.iter().any(|filter| matches_record(row, filter)));
                    }

                    let count = rows.len();
                    set_var(&mut vars, shared, result_target, Value::Array(rows));
                    if let Some(ft) = failed_target {
                        set_var(&mut vars, shared, ft, Value::Array(Vec::new()));
                    }
                    if let Some(rt) = reported_target {
                        set_var(&mut vars, shared, rt, Value::Array(Vec::new()));
                    }
                    shared.selected_rows += count;
                    if let Some(Value::Object(sy_map)) = vars.get_mut("SY") {
                        sy_map.insert("SUBRC".to_string(), serde_json::json!(if count > 0 { 0 } else { 4 }));
                        sy_map.insert("DBCNT".to_string(), serde_json::json!(count));
                    }
                    pc += 1;
                }
                Op::EmlModify { entity: _, operation, rows, mapped_target, failed_target, reported_target } => {
                    let value_count: usize = rows.iter().map(|r| r.names.len()).sum();
                    let take_count = value_count.min(stack.len());
                    let values: Vec<Value> = stack.drain(stack.len() - take_count..).collect();

                    let mut offset = 0;
                    let mut mapped = Vec::new();
                    let mut failed = Vec::new();
                    let mut reported = Vec::new();

                    for row_spec in rows {
                        let mut row = serde_json::Map::new();
                        for name in &row_spec.names {
                            row.insert(canonical_product_field(name), values.get(offset).cloned().unwrap_or(Value::Null));
                            offset += 1;
                        }
                        let key = row
                            .get("PRODUCT_ID")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let res = (|| -> Result<(), String> {
                            if key.is_empty() {
                                return Err("PRODUCT_ID is required.".to_string());
                            }
                            let current = effective_product_rows(&self.db, shared);
                            let existing = current.iter().find(|x| {
                                x.get("PRODUCT_ID")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s == key)
                                    .unwrap_or(false)
                            });

                            if operation == "CREATE" {
                                if existing.is_some() {
                                    return Err(format!("Product '{}' already exists.", key));
                                }
                                let name_str = row
                                    .get("PRODUCT_NAME")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                if name_str.trim().is_empty() {
                                    return Err("PRODUCT_NAME is required.".to_string());
                                }
                                let price_val = row.get("PRICE").and_then(|p| to_number(p).ok()).unwrap_or(0.0);
                                if price_val < 0.0 {
                                    return Err("PRICE must be non-negative.".to_string());
                                }
                                let mut data = serde_json::Map::new();
                                data.insert("PRODUCT_ID".to_string(), serde_json::json!(key));
                                data.insert("PRODUCT_NAME".to_string(), serde_json::json!(name_str));
                                data.insert("CATEGORY".to_string(), row.get("CATEGORY").cloned().unwrap_or_else(|| serde_json::json!("")));
                                data.insert("PRICE".to_string(), serde_json::json!(price_val));
                                data.insert("CURRENCY".to_string(), row.get("CURRENCY").cloned().unwrap_or_else(|| serde_json::json!("INR")));

                                shared.eml_pending.push(EmlPendingOp {
                                    operation: "CREATE".to_string(),
                                    key: key.clone(),
                                    data: Some(Value::Object(data)),
                                });
                                mapped.push(serde_json::json!({"PRODUCT_ID": key}));
                            } else if operation == "UPDATE" {
                                let exist_rec = match existing {
                                    Some(r) => r,
                                    None => return Err(format!("Product '{}' does not exist.", key)),
                                };
                                let mut merged = match exist_rec {
                                    Value::Object(m) => m.clone(),
                                    _ => serde_json::Map::new(),
                                };
                                for (k, v) in row {
                                    merged.insert(k, v);
                                }
                                if let Some(p) = merged.get("PRICE") {
                                    if let Ok(price_val) = to_number(p) {
                                        if price_val < 0.0 {
                                            return Err("PRICE must be non-negative.".to_string());
                                        }
                                    }
                                }
                                shared.eml_pending.push(EmlPendingOp {
                                    operation: "UPDATE".to_string(),
                                    key: key.clone(),
                                    data: Some(Value::Object(merged)),
                                });
                            } else {
                                if existing.is_none() {
                                    return Err(format!("Product '{}' does not exist.", key));
                                }
                                shared.eml_pending.push(EmlPendingOp {
                                    operation: "DELETE".to_string(),
                                    key: key.clone(),
                                    data: None,
                                });
                            }
                            Ok(())
                        })();

                        if let Err(e) = res {
                            failed.push(serde_json::json!({"PRODUCT_ID": key, "MESSAGE": e}));
                            reported.push(serde_json::json!({"PRODUCT_ID": key, "TYPE": "E", "MESSAGE": e}));
                        }
                    }

                    if let Some(mt) = mapped_target {
                        set_var(&mut vars, shared, mt, Value::Array(mapped));
                    }
                    if let Some(ft) = failed_target {
                        set_var(&mut vars, shared, ft, Value::Array(failed.clone()));
                    }
                    if let Some(rt) = reported_target {
                        set_var(&mut vars, shared, rt, Value::Array(reported));
                    }

                    if let Some(Value::Object(sy_map)) = vars.get_mut("SY") {
                        sy_map.insert("SUBRC".to_string(), serde_json::json!(if !failed.is_empty() { 4 } else { 0 }));
                    }
                    pc += 1;
                }
                Op::EmlCommit => {
                    let mut candidate = self.db.get("ZPRODUCT").cloned().unwrap_or_default();
                    for item in &shared.eml_pending {
                        let idx = candidate.iter().position(|r| {
                            r.get("PRODUCT_ID")
                                .and_then(|v| v.as_str())
                                .map(|s| s == item.key)
                                .unwrap_or(false)
                        });

                        if item.operation == "CREATE" {
                            if idx.is_some() {
                                return Err(format!("EML commit conflict: product '{}' already exists.", item.key));
                            }
                            candidate.push(item.data.clone().unwrap_or(Value::Null));
                        } else if item.operation == "UPDATE" {
                            match idx {
                                Some(i) => candidate[i] = item.data.clone().unwrap_or(Value::Null),
                                None => return Err(format!("EML commit conflict: product '{}' no longer exists.", item.key)),
                            }
                        } else {
                            match idx {
                                Some(i) => {
                                    candidate.remove(i);
                                }
                                None => return Err(format!("EML commit conflict: product '{}' no longer exists.", item.key)),
                            }
                        }
                    }
                    self.db.insert("ZPRODUCT".to_string(), candidate);
                    shared.eml_pending.clear();
                    if let Some(Value::Object(sy_map)) = vars.get_mut("SY") {
                        sy_map.insert("SUBRC".to_string(), serde_json::json!(0));
                    }
                    pc += 1;
                }
                Op::EmlRollback => {
                    shared.eml_pending.clear();
                    if let Some(Value::Object(sy_map)) = vars.get_mut("SY") {
                        sy_map.insert("SUBRC".to_string(), serde_json::json!(0));
                    }
                    pc += 1;
                }
                Op::CallFunction { name, params } => {
                    let take_count = params.len().min(stack.len());
                    let vals: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                    let mut pmap = HashMap::new();
                    for (i, p) in params.iter().enumerate() {
                        pmap.insert(p.clone(), vals.get(i).cloned().unwrap_or(Value::Null));
                    }
                    if name.eq_ignore_ascii_case("THROW_ERROR") {
                        let msg = pmap
                            .get("MESSAGE")
                            .and_then(|v| v.as_str())
                            .unwrap_or("External function error.");
                        return Err(msg.to_string());
                    }
                    pc += 1;
                }
                Op::CallMethod { receiver, method, argc, named, output_bindings, static_call } => {
                    let take_count = (*argc).min(stack.len());
                    let vals: Vec<Value> = stack.drain(stack.len() - take_count..).collect();
                    let positional_count = argc - named.len();
                    let positional = vals.get(..positional_count).unwrap_or(&[]).to_vec();
                    let mut named_map = HashMap::new();
                    for (i, k) in named.iter().enumerate() {
                        named_map.insert(
                            k.to_ascii_uppercase(),
                            vals.get(positional_count + i).cloned().unwrap_or(Value::Null),
                        );
                    }

                    if receiver.eq_ignore_ascii_case("OUT") {
                        call_out_method(method, &positional, &named_map, &mut shared.output)?;
                    } else {
                        let recv = if *static_call {
                            let mut map = serde_json::Map::new();
                            map.insert("__CLASS".to_string(), serde_json::json!(receiver.to_ascii_uppercase()));
                            map.insert("__STATIC_CONTEXT".to_string(), serde_json::json!(true));
                            Value::Object(map)
                        } else {
                            get_var(&vars, shared, receiver)
                        };

                        if *static_call {
                            self.ensure_class_initialized(receiver, shared, max_steps)?;
                        }
                        shared.globals.extend(vars.clone());
                        let result = try_op!(self.invoke_user_method(&recv, method, &positional, &named_map, shared, max_steps));
                        vars.extend(shared.globals.clone());
                        if let Some(me) = result.receiver {
                            if !*static_call {
                                set_var(&mut vars, shared, receiver, me);
                            }
                        }
                        apply_output_bindings(&result.outputs, output_bindings, &mut vars);
                    }
                    pc += 1;
                }
                Op::Clear => {
                    let mut v = stack.pop().unwrap_or(Value::Null);
                    clear_value(&mut v);
                    pc += 1;
                }
                Op::ClearVar { name } => {
                    let mut cur = get_var(&vars, shared, name);
                    clear_value(&mut cur);
                    if matches!(cur, Value::Array(..) | Value::Object(..)) {
                        set_var(&mut vars, shared, name, cur);
                    } else {
                        set_var(&mut vars, shared, name, serde_json::json!(0));
                    }
                    pc += 1;
                }
                Op::Assert => {
                    let v = stack.pop().unwrap_or(Value::Null);
                    if !truthy(&v) {
                        return Err("ASSERT condition failed.".to_string());
                    }
                    pc += 1;
                }
                Op::Drop => {
                    stack.pop();
                    pc += 1;
                }
                Op::Nop => {
                    pc += 1;
                }
                Op::Continue | Op::ExitLoop => {
                    pc += 1;
                }
                Op::Return => {
                    return_flag = true;
                }
                Op::PushHandler { catch_type, target } => {
                    handlers.push(ExceptionHandler {
                        catch_type: catch_type.to_ascii_uppercase(),
                        target: *target as usize,
                        stack_depth: stack.len(),
                    });
                    pc += 1;
                }
                Op::PopHandler => {
                    handlers.pop();
                    pc += 1;
                }
                Op::AssignAlias { source, target } => {
                    let tgt_key = target.to_ascii_uppercase();
                    let src_key = source.to_ascii_uppercase();
                    alias_map.insert(tgt_key.clone(), src_key.clone());
                    let val = get_var(&vars, shared, &src_key);
                    vars.insert(tgt_key, val);
                    pc += 1;
                }
                Op::AssignTableAlias { target, table_name } => {
                    let tabix = vars.get("SY-TABIX").and_then(|v| v.as_i64()).unwrap_or(1);
                    let idx = if tabix > 0 { (tabix - 1) as usize } else { 0 };
                    table_alias_map.insert(target.to_ascii_uppercase(), (table_name.to_ascii_uppercase(), idx));
                    pc += 1;
                }
            }

            if return_flag {
                break;
            }
        }

        Ok(VMResult {
            output: shared.output.clone(),
            variables: vars,
            steps: shared.steps,
            elapsed_ms: started.elapsed().as_millis() as u64,
            selected_rows: shared.selected_rows,
        })
    }

    fn ensure_class_initialized(
        &mut self,
        class_name: &str,
        shared: &mut SharedExecution,
        max_steps: usize,
    ) -> Result<(), String> {
        let mut lineage = class_lineage_runtime(shared, class_name);
        lineage.reverse();

        for class_ir in lineage {
            let key = class_ir.name.to_ascii_uppercase();
            if shared.class_initialized.contains(&key) {
                continue;
            }
            shared.class_initialized.insert(key.clone());
            ensure_static_data(shared, &key);
            let ctor = resolve_runtime_method(shared, &key, "CLASS_CONSTRUCTOR");
            if ctor.is_some() {
                let mut ctx = serde_json::Map::new();
                ctx.insert("__CLASS".to_string(), serde_json::json!(key));
                ctx.insert("__STATIC_CONTEXT".to_string(), serde_json::json!(true));
                self.invoke_user_method(
                    &Value::Object(ctx),
                    "CLASS_CONSTRUCTOR",
                    &[],
                    &HashMap::new(),
                    shared,
                    max_steps,
                )?;
            }
        }
        Ok(())
    }

    fn invoke_user_method(
        &mut self,
        receiver: &Value,
        method_name: &str,
        positional: &[Value],
        named: &HashMap<String, Value>,
        shared: &mut SharedExecution,
        max_steps: usize,
    ) -> Result<InvokeMethodResult, String> {
        let obj_map = match receiver {
            Value::Object(m) => m,
            _ => return Err(format!("Cannot call method '{}' on a non-object reference.", method_name)),
        };

        let class_name = obj_map
            .get("__CLASS")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_uppercase();

        let resolved = resolve_runtime_method(shared, &class_name, method_name)
            .ok_or_else(|| format!("Class '{}' and its superclasses have no executable method '{}'.", class_name, method_name))?;

        let is_static_ctx = obj_map.get("__STATIC_CONTEXT").and_then(|v| v.as_bool()).unwrap_or(false);
        let method_static = resolved.method.is_static.unwrap_or(false);

        if is_static_ctx != method_static {
            return Err(if is_static_ctx {
                format!("Method '{}' is not static.", resolved.method.name)
            } else {
                format!("Method '{}' is static and must be called with =>.", resolved.method.name)
            });
        }

        if shared.depth >= 128 {
            return Err("Maximum method call depth (128) exceeded; possible recursion overflow.".to_string());
        }

        if is_static_ctx {
            ensure_static_data(shared, &class_name);
        }

        let mut bindings: HashMap<String, Value> = shared.globals.clone();
        bindings.insert("ME".to_string(), receiver.clone());

        let mut positional_idx = 0;
        for formal in &resolved.method.params {
            let key = formal.name.to_ascii_uppercase();
            let value = if let Some(v) = named.get(&key) {
                v.clone()
            } else if formal.direction == "importing" && positional_idx < positional.len() {
                let v = positional[positional_idx].clone();
                positional_idx += 1;
                v
            } else if formal.direction == "importing" && formal.optional.unwrap_or(false) {
                if let Some(Expr::Literal { value, .. }) = &formal.default_value {
                    value.clone()
                } else {
                    initial_value_for_type(&formal.type_ref)
                }
            } else if formal.direction == "importing" {
                return Err(format!("Missing required parameter '{}' for {}->{}.", formal.name, resolved.class_ir.name, resolved.method.name));
            } else if formal.direction == "changing" {
                return Err(format!("Missing CHANGING parameter '{}' for {}->{}.", formal.name, resolved.class_ir.name, resolved.method.name));
            } else {
                initial_value_for_type(&formal.type_ref)
            };

            bindings.insert(key, value);
        }

        for k in named.keys() {
            if !resolved.method.params.iter().any(|p| p.name.eq_ignore_ascii_case(k)) {
                return Err(format!("Unknown parameter '{}' for {}->{}.", k, resolved.class_ir.name, resolved.method.name));
            }
        }
        if positional_idx < positional.len() {
            return Err(format!("Too many positional arguments for {}->{}.", resolved.class_ir.name, resolved.method.name));
        }

        shared.depth += 1;
        let method_program = ProgramIR {
            vars: resolved.method.vars.clone(),
            ops: resolved.method.ops.clone(),
            mode: Some("method".to_string()),
            entry_point: None,
            classes: shared.classes.clone(),
        };

        let empty_params = HashMap::new();
        let exec_result = self.execute(
            &method_program,
            max_steps,
            &empty_params,
            false,
            true,
            Some(shared),
            Some(&bindings),
        );
        shared.depth -= 1;

        let res = exec_result?;
        shared.globals.extend(res.variables.clone());
        let mut outputs = HashMap::new();
        for formal in &resolved.method.params {
            if formal.direction == "exporting" || formal.direction == "changing" {
                let val = res.variables.get(&formal.name.to_ascii_uppercase()).cloned().unwrap_or(Value::Null);
                outputs.insert(formal.name.to_ascii_uppercase(), val);
            }
        }

        let ret_val = if let Some(ret) = &resolved.method.returning {
            res.variables.get(&ret.name.to_ascii_uppercase()).cloned().unwrap_or(Value::Null)
        } else {
            Value::Null
        };

        let mutated_me = res.variables.get("ME").cloned();

        Ok(InvokeMethodResult {
            value: ret_val,
            outputs,
            receiver: mutated_me,
        })
    }
}

pub struct InvokeMethodResult {
    pub value: Value,
    pub outputs: HashMap<String, Value>,
    pub receiver: Option<Value>,
}

pub struct ResolvedMethodInfo {
    pub class_ir: ClassIR,
    pub method: MethodIR,
}

fn resolve_runtime_method(
    shared: &SharedExecution,
    class_name: &str,
    method_name: &str,
) -> Option<ResolvedMethodInfo> {
    let m_upper = method_name.to_ascii_uppercase();
    for class_ir in class_lineage_runtime(shared, class_name) {
        if let Some(m) = class_ir.methods.get(&m_upper).cloned() {
            return Some(ResolvedMethodInfo {
                class_ir,
                method: m,
            });
        }
        if let Some(idx) = m_upper.rfind('~') {
            let bare = &m_upper[idx + 1..];
            if let Some(m) = class_ir.methods.get(bare).cloned() {
                return Some(ResolvedMethodInfo {
                    class_ir,
                    method: m,
                });
            }
        } else {
            for (k, m) in &class_ir.methods {
                if k.ends_with(&format!("~{}", m_upper)) {
                    return Some(ResolvedMethodInfo {
                        class_ir: class_ir.clone(),
                        method: m.clone(),
                    });
                }
            }
        }
    }
    None
}

fn class_lineage_runtime(shared: &SharedExecution, class_name: &str) -> Vec<ClassIR> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let mut current_name = class_name.to_ascii_uppercase();

    while let Some(current) = shared.classes.get(&current_name) {
        let key = current.name.to_ascii_uppercase();
        if seen.contains(&key) {
            break;
        }
        seen.insert(key);
        result.push(current.clone());
        if let Some(super_name) = &current.super_class {
            current_name = super_name.to_ascii_uppercase();
        } else {
            break;
        }
    }
    result
}

struct StaticValueLookup {
    found: bool,
    owner: String,
    value: Option<Value>,
}

fn find_static_value(shared: &SharedExecution, class_name: &str, name: &str) -> StaticValueLookup {
    for class_ir in class_lineage_runtime(shared, class_name) {
        if let Some(attr) = class_ir.attributes.get(&name.to_ascii_uppercase()) {
            if attr.is_static {
                let owner = class_ir.name.to_ascii_uppercase();
                let val = shared
                    .static_data
                    .get(&owner)
                    .and_then(|m| m.get(&name.to_ascii_uppercase()))
                    .cloned();
                return StaticValueLookup {
                    found: true,
                    owner,
                    value: val,
                };
            }
        }
    }
    StaticValueLookup {
        found: false,
        owner: String::new(),
        value: None,
    }
}

fn ensure_static_data(shared: &mut SharedExecution, class_name: &str) {
    let mut lineage = class_lineage_runtime(shared, class_name);
    lineage.reverse();

    for class_ir in lineage {
        let key = class_ir.name.to_ascii_uppercase();
        if shared.static_data.contains_key(&key) {
            continue;
        }
        let mut values = HashMap::new();
        for (name, attr) in &class_ir.attributes {
            if attr.is_static {
                let val = attr
                    .initial_value
                    .clone()
                    .unwrap_or_else(|| initial_value_for_type(&attr.type_ref));
                values.insert(name.to_ascii_uppercase(), val);
            }
        }
        shared.static_data.insert(key, values);
    }
}

fn initialize_object(shared: &SharedExecution, class_name: &str) -> Value {
    let mut object = serde_json::Map::new();
    object.insert("__CLASS".to_string(), serde_json::json!(class_name.to_ascii_uppercase()));

    let mut lineage = class_lineage_runtime(shared, class_name);
    lineage.reverse();

    for class_ir in lineage {
        for (name, attr) in &class_ir.attributes {
            if !attr.is_static {
                let val = attr
                    .initial_value
                    .clone()
                    .unwrap_or_else(|| initial_value_for_type(&attr.type_ref));
                object.insert(name.to_ascii_uppercase(), val);
            }
        }
    }
    Value::Object(object)
}

fn apply_output_bindings(
    outputs: &HashMap<String, Value>,
    bindings: &[OutputBinding],
    vars: &mut HashMap<String, Value>,
) {
    for b in bindings {
        let key = b.formal.to_ascii_uppercase();
        if let Some(val) = outputs.get(&key) {
            vars.insert(b.target.to_ascii_uppercase(), val.clone());
        }
    }
}

struct IteratorCallResult {
    handled: bool,
    push: bool,
    value: Option<Value>,
}

fn iterator_call(
    name: &str,
    args: &[Value],
    iters: &mut Vec<IteratorState>,
    vars: &mut HashMap<String, Value>,
    table_alias_map: &mut HashMap<String, (String, usize)>,
) -> IteratorCallResult {
    match name.to_ascii_uppercase().as_str() {
        "__ITER_START" => {
            if let Some(tbl_name) = args.first().and_then(|v| v.as_str()) {
                let fs_name = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string()).filter(|s| !s.is_empty());
                iters.push(IteratorState {
                    table_name: Some(tbl_name.to_ascii_uppercase()),
                    values: Vec::new(),
                    index: 0,
                    target_fs: fs_name,
                });
            } else {
                let table = match args.first() {
                    Some(Value::Array(arr)) => arr.clone(),
                    _ => Vec::new(),
                };
                iters.push(IteratorState {
                    table_name: None,
                    values: table,
                    index: 0,
                    target_fs: None,
                });
            }
            IteratorCallResult {
                handled: true,
                push: true,
                value: Some(Value::Null),
            }
        }
        "__ITER_HAS_NEXT" => {
            let has = if let Some(it) = iters.last() {
                if let Some(ref tbl) = it.table_name {
                    if let Some(Value::Array(arr)) = vars.get(tbl) {
                        it.index < arr.len()
                    } else {
                        false
                    }
                } else {
                    it.index < it.values.len()
                }
            } else {
                false
            };
            if !has && !iters.is_empty() {
                iters.pop();
            }
            IteratorCallResult {
                handled: true,
                push: true,
                value: Some(Value::Bool(has)),
            }
        }
        "__ITER_VALUE" => {
            let (val, tabix) = if let Some(it) = iters.last() {
                let tabix = it.index + 1;
                let v = if let Some(ref tbl) = it.table_name {
                    if let Some(ref fs) = it.target_fs {
                        table_alias_map.insert(fs.to_ascii_uppercase(), (tbl.clone(), it.index));
                    }
                    if let Some(Value::Array(arr)) = vars.get(tbl) {
                        arr.get(it.index).cloned().unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    }
                } else {
                    it.values.get(it.index).cloned().unwrap_or(Value::Null)
                };
                (v, tabix)
            } else {
                (Value::Null, 0)
            };
            vars.insert("SY-TABIX".to_string(), serde_json::json!(tabix));
            if let Some(Value::Object(sy_map)) = vars.get_mut("SY") {
                sy_map.insert("TABIX".to_string(), serde_json::json!(tabix));
            }
            IteratorCallResult {
                handled: true,
                push: true,
                value: Some(val),
            }
        }
        "__ITER_NEXT" => {
            if let Some(it) = iters.last_mut() {
                it.index += 1;
            }
            IteratorCallResult {
                handled: true,
                push: true,
                value: Some(Value::Null),
            }
        }
        _ => IteratorCallResult {
            handled: false,
            push: false,
            value: None,
        },
    }
}

fn get_var(vars: &HashMap<String, Value>, shared: &SharedExecution, name: &str) -> Value {
    let key = name.to_ascii_uppercase();
    if let Some(v) = vars.get(&key) {
        return v.clone();
    }
    if key.contains('-') {
        let mut parts = key.splitn(2, '-');
        if let (Some(base), Some(field)) = (parts.next(), parts.next()) {
            if let Some(v) = vars.get(base) {
                let fv = field_get(v, field);
                if !fv.is_null() {
                    return fv;
                }
            }
        }
    }
    if let Some(Value::Object(me)) = vars.get("ME") {
        if !me.get("__STATIC_CONTEXT").and_then(|v| v.as_bool()).unwrap_or(false) {
            if let Some(val) = me.get(&key) {
                return val.clone();
            }
        }
        if let Some(Value::String(cname)) = me.get("__CLASS") {
            let found = find_static_value(shared, cname, &key);
            if found.found {
                return found.value.unwrap_or(serde_json::json!(0));
            }
        }
    }
    serde_json::json!(0)
}

fn set_var(vars: &mut HashMap<String, Value>, shared: &mut SharedExecution, name: &str, v: Value) {
    let key = name.to_ascii_uppercase();
    if key.contains('-') {
        let mut parts = key.splitn(2, '-');
        if let (Some(base), Some(field)) = (parts.next(), parts.next()) {
            if let Some(base_val) = vars.get_mut(base) {
                let _ = field_set(base_val, field, v.clone());
            }
        }
    }
    if vars.contains_key(&key) {
        vars.insert(key, v);
        return;
    }
    if let Some(Value::Object(me)) = vars.get_mut("ME") {
        if !me.get("__STATIC_CONTEXT").and_then(|x| x.as_bool()).unwrap_or(false) {
            if me.contains_key(&key) {
                me.insert(key, v);
                return;
            }
        }
        if let Some(Value::String(cname)) = me.get("__CLASS").cloned() {
            let found = find_static_value(shared, &cname, &key);
            if found.found {
                let class_data = shared.static_data.entry(found.owner).or_default();
                class_data.insert(key, v);
                return;
            }
        }
    }
    vars.insert(key, v);
}

fn call_out_method(
    method: &str,
    args: &[Value],
    named: &HashMap<String, Value>,
    output: &mut Vec<String>,
) -> Result<(), String> {
    let m = method.to_ascii_uppercase();
    if m == "WRITE" || m.ends_with("~WRITE") {
        let values: Vec<Value> = if !args.is_empty() {
            args.to_vec()
        } else if let Some(d) = named.get("DATA") {
            vec![d.clone()]
        } else {
            Vec::new()
        };
        for v in values {
            output.push(format_value(&v));
        }
        return Ok(());
    }
    Err(format!("Unknown runtime method OUT->{}.", method))
}

fn field_get(base: &Value, field: &str) -> Value {
    match base {
        Value::Object(map) => {
            map.get(&field.to_ascii_uppercase())
                .or_else(|| map.get(field))
                .cloned()
                .unwrap_or(Value::Null)
        }
        Value::Array(arr) => {
            if let Some(first) = arr.first() {
                field_get(first, field)
            } else {
                Value::Null
            }
        }
        _ => Value::Null,
    }
}

fn field_set(base: &mut Value, field: &str, v: Value) -> Result<(), String> {
    if !base.is_object() {
        *base = Value::Object(serde_json::Map::new());
    }
    match base {
        Value::Object(map) => {
            map.insert(field.to_ascii_uppercase(), v);
            Ok(())
        }
        _ => Err(format!("Cannot assign component {} on a non-structure.", field)),
    }
}

fn index_get(base: &Value, index: &Value) -> Value {
    if let Ok(n) = to_number(index) {
        let idx = (n as i64) - 1;
        if idx >= 0 {
            if let Value::Array(arr) = base {
                return arr.get(idx as usize).cloned().unwrap_or(Value::Null);
            }
            if let Value::String(s) = base {
                return s.chars().nth(idx as usize).map(|c| Value::String(c.to_string())).unwrap_or_else(|| Value::String(String::new()));
            }
        }
    }
    Value::Null
}

fn index_set(base: &mut Value, index: &Value, v: Value) -> Result<(), String> {
    let n = to_number(index)?;
    let idx = (n as i64) - 1;
    if idx >= 0 {
        if let Value::Array(arr) = base {
            let uidx = idx as usize;
            if uidx >= arr.len() {
                arr.resize(uidx + 1, Value::Null);
            }
            arr[uidx] = v;
            return Ok(());
        }
    }
    Err("Indexed target is not an array.".to_string())
}

pub fn format_value(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => if *b { "X".to_string() } else { String::new() },
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                    (f as i64).to_string()
                } else {
                    n.to_string()
                }
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Array(..) | Value::Object(..) => serde_json::to_string(v).unwrap_or_default(),
    }
}

fn clear_value(v: &mut Value) {
    match v {
        Value::Array(arr) => arr.clear(),
        Value::Object(map) => {
            for val in map.values_mut() {
                *val = Value::Null;
            }
        }
        _ => *v = serde_json::json!(0),
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        Value::Object(map) => !map.is_empty(),
    }
}

fn to_number(v: &Value) -> Result<f64, String> {
    match v {
        Value::Number(n) => n.as_f64().ok_or_else(|| "Invalid number".to_string()),
        Value::String(s) => s.trim().parse::<f64>().map_err(|_| format!("Expected numeric value, got '{}'.", s)),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::Null => Ok(0.0),
        _ => Err(format!("Expected numeric value, got {}.", format_value(v))),
    }
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    if let (Some(na), Some(nb)) = (a.as_f64(), b.as_f64()) {
        return na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal);
    }
    if let (Some(sa), Some(sb)) = (a.as_str(), b.as_str()) {
        return sa.cmp(sb);
    }
    let sa = format_value(a);
    let sb = format_value(b);
    sa.cmp(&sb)
}

fn matches_abap_pattern(text: &str, pattern: &str) -> bool {
    let t: Vec<char> = text.to_ascii_uppercase().chars().collect();
    let p: Vec<char> = pattern.to_ascii_uppercase().chars().collect();
    let mut dp = vec![vec![false; p.len() + 1]; t.len() + 1];
    dp[0][0] = true;
    for j in 1..=p.len() {
        if p[j - 1] == '*' {
            dp[0][j] = dp[0][j - 1];
        }
    }
    for i in 1..=t.len() {
        for j in 1..=p.len() {
            if p[j - 1] == '*' {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if p[j - 1] == '+' || p[j - 1] == t[i - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }
    dp[t.len()][p.len()]
}

fn binary_op(op: &str, l: &Value, r: &Value) -> Result<Value, String> {
    match op {
        "+" => {
            if l.is_string() || r.is_string() {
                Ok(Value::String(format!("{}{}", format_value(l), format_value(r))))
            } else {
                let nl = to_number(l)?;
                let nr = to_number(r)?;
                let sum = nl + nr;
                if sum.fract() == 0.0 && sum >= i64::MIN as f64 && sum <= i64::MAX as f64 {
                    Ok(serde_json::json!(sum as i64))
                } else {
                    Ok(serde_json::json!(sum))
                }
            }
        }
        "-" => {
            let nl = to_number(l)?;
            let nr = to_number(r)?;
            let diff = nl - nr;
            if diff.fract() == 0.0 && diff >= i64::MIN as f64 && diff <= i64::MAX as f64 {
                Ok(serde_json::json!(diff as i64))
            } else {
                Ok(serde_json::json!(diff))
            }
        }
        "*" => {
            let nl = to_number(l)?;
            let nr = to_number(r)?;
            let prod = nl * nr;
            if prod.fract() == 0.0 && prod >= i64::MIN as f64 && prod <= i64::MAX as f64 {
                Ok(serde_json::json!(prod as i64))
            } else {
                Ok(serde_json::json!(prod))
            }
        }
        "/" => {
            let d = to_number(r)?;
            if d == 0.0 {
                return Err("CX_SY_ZERODIVIDE".to_string());
            }
            let nl = to_number(l)?;
            let q = nl / d;
            if q.fract() == 0.0 && q >= i64::MIN as f64 && q <= i64::MAX as f64 {
                Ok(serde_json::json!(q as i64))
            } else {
                Ok(serde_json::json!(q))
            }
        }
        "MOD" => {
            let d = to_number(r)?;
            if d == 0.0 {
                return Err("CX_SY_ZERODIVIDE".to_string());
            }
            let nl = to_number(l)?;
            Ok(serde_json::json!((nl as i64) % (d as i64)))
        }
        "DIV" => {
            let d = to_number(r)?;
            if d == 0.0 {
                return Err("CX_SY_ZERODIVIDE".to_string());
            }
            let nl = to_number(l)?;
            Ok(serde_json::json!((nl as i64) / (d as i64)))
        }
        "=" | "==" | "EQ" => Ok(Value::Bool(compare_values(l, r) == std::cmp::Ordering::Equal)),
        "<>" | "NE" => Ok(Value::Bool(compare_values(l, r) != std::cmp::Ordering::Equal)),
        "<" | "LT" => Ok(Value::Bool(compare_values(l, r) == std::cmp::Ordering::Less)),
        "<=" | "LE" => Ok(Value::Bool(compare_values(l, r) != std::cmp::Ordering::Greater)),
        ">" | "GT" => Ok(Value::Bool(compare_values(l, r) == std::cmp::Ordering::Greater)),
        ">=" | "GE" => Ok(Value::Bool(compare_values(l, r) != std::cmp::Ordering::Less)),
        "CS" => Ok(Value::Bool(format_value(l).to_uppercase().contains(&format_value(r).to_uppercase()))),
        "NS" => Ok(Value::Bool(!format_value(l).to_uppercase().contains(&format_value(r).to_uppercase()))),
        "CP" => Ok(Value::Bool(matches_abap_pattern(&format_value(l), &format_value(r)))),
        "NP" => Ok(Value::Bool(!matches_abap_pattern(&format_value(l), &format_value(r)))),
        "CA" => {
            let ls = format_value(l);
            let rs = format_value(r);
            Ok(Value::Bool(ls.chars().any(|c| rs.contains(c))))
        }
        "NA" => {
            let ls = format_value(l);
            let rs = format_value(r);
            Ok(Value::Bool(!ls.chars().any(|c| rs.contains(c))))
        }
        "CO" => {
            let ls = format_value(l);
            let rs = format_value(r);
            Ok(Value::Bool(ls.chars().all(|c| rs.contains(c))))
        }
        "CN" => {
            let ls = format_value(l);
            let rs = format_value(r);
            Ok(Value::Bool(!ls.chars().all(|c| rs.contains(c))))
        }
        "AND" => Ok(Value::Bool(truthy(l) && truthy(r))),
        "OR" => Ok(Value::Bool(truthy(l) || truthy(r))),
        "&&" | "||" => Ok(Value::String(format!("{}{}", format_value(l), format_value(r)))),
        "IS" => {
            let pred = r.as_str().unwrap_or("").to_ascii_uppercase();
            if pred == "ASSIGNED" {
                Ok(Value::Bool(!l.is_null()))
            } else if pred == "INITIAL" {
                Ok(Value::Bool(!truthy(l)))
            } else if r.is_null() {
                Ok(Value::Bool(l.is_null()))
            } else {
                Ok(Value::Bool(truthy(l)))
            }
        }
        "IN" => Ok(Value::Bool(is_in_ranges(l, r))),
        _ => Err(format!("Unknown operator {}", op)),
    }
}

fn matches_record(row: &Value, key: &HashMap<String, Value>) -> bool {
    for (k, v) in key {
        let field_val = field_get(row, k);
        if compare_values(&field_val, v) != std::cmp::Ordering::Equal {
            return false;
        }
    }
    true
}

fn default_record(found: Option<&Value>) -> Value {
    match found {
        Some(Value::Object(..)) => Value::Object(serde_json::Map::new()),
        _ => serde_json::json!(0),
    }
}

fn call_builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    let u = name.to_ascii_uppercase();
    match u.as_str() {
        "STRLEN" => {
            let s = args.first().map(format_value).unwrap_or_default();
            Ok(serde_json::json!(s.len()))
        }
        "XSTRLEN" => {
            let s = args.first().map(format_value).unwrap_or_default();
            Ok(serde_json::json!(s.as_bytes().len()))
        }
        "TO_UPPER" => {
            let s = args.first().map(format_value).unwrap_or_default();
            Ok(Value::String(s.to_ascii_uppercase()))
        }
        "TO_LOWER" => {
            let s = args.first().map(format_value).unwrap_or_default();
            Ok(Value::String(s.to_ascii_lowercase()))
        }
        "CONCATENATE" => {
            let s = args.iter().map(format_value).collect::<Vec<_>>().join("");
            Ok(Value::String(s))
        }
        "CONDENSE" => {
            let s = args.first().map(format_value).unwrap_or_default();
            Ok(Value::String(s.split_whitespace().collect::<Vec<_>>().join(" ")))
        }
        "ABS" => {
            let n = args.first().map(to_number).transpose()?.unwrap_or(0.0);
            Ok(serde_json::json!(n.abs()))
        }
        "FLOOR" => {
            let n = args.first().map(to_number).transpose()?.unwrap_or(0.0);
            Ok(serde_json::json!(n.floor()))
        }
        "CEIL" => {
            let n = args.first().map(to_number).transpose()?.unwrap_or(0.0);
            Ok(serde_json::json!(n.ceil()))
        }
        "ROUND" => {
            let n = args.first().map(to_number).transpose()?.unwrap_or(0.0);
            Ok(serde_json::json!(n.round()))
        }
        "LINES" => {
            let count = match args.first() {
                Some(Value::Array(a)) => a.len(),
                _ => 0,
            };
            Ok(serde_json::json!(count))
        }
        _ => Err(format!("Unknown built-in function '{}'.", name)),
    }
}

fn canonical_product_field(name: &str) -> String {
    let cleaned: String = name.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let upper = cleaned.to_ascii_uppercase();
    match upper.as_str() {
        "PRODUCTID" => "PRODUCT_ID".to_string(),
        "PRODUCTNAME" => "PRODUCT_NAME".to_string(),
        _ => upper,
    }
}

fn effective_product_rows(
    db: &HashMap<String, Vec<Value>>,
    shared: &SharedExecution,
) -> Vec<Value> {
    let mut rows = db.get("ZPRODUCT").cloned().unwrap_or_default();
    for op in &shared.eml_pending {
        let index = rows.iter().position(|r| {
            r.get("PRODUCT_ID")
                .and_then(|v| v.as_str())
                .map(|s| s == op.key)
                .unwrap_or(false)
        });
        if op.operation == "CREATE" {
            if let Some(d) = &op.data {
                rows.push(d.clone());
            }
        } else if op.operation == "UPDATE" {
            if let Some(i) = index {
                if let Some(d) = &op.data {
                    rows[i] = d.clone();
                }
            }
        } else if op.operation == "DELETE" {
            if let Some(i) = index {
                rows.remove(i);
            }
        }
    }
    rows
}

fn coerce_parameter(v: &VarIR, value: &Value, name: &str) -> Result<Value, String> {
    if v.selection_options.unwrap_or(false) {
        let rows = match value {
            Value::Array(arr) => arr.clone(),
            Value::Object(..) => vec![value.clone()],
            _ => Vec::new(),
        };

        let mut out = Vec::new();
        for r in rows {
            let sign = r.get("sign").or_else(|| r.get("SIGN")).and_then(|s| s.as_str()).unwrap_or("I").to_ascii_uppercase();
            let option = r.get("option").or_else(|| r.get("OPTION")).and_then(|s| s.as_str()).unwrap_or("EQ").to_ascii_uppercase();
            let low = r.get("low").or_else(|| r.get("LOW")).map(format_value).unwrap_or_default();
            let high = r.get("high").or_else(|| r.get("HIGH")).map(format_value).unwrap_or_default();
            if !low.trim().is_empty() || !high.trim().is_empty() {
                out.push(serde_json::json!({
                    "SIGN": sign,
                    "OPTION": option,
                    "LOW": low,
                    "HIGH": high
                }));
            }
        }
        return Ok(Value::Array(out));
    }

    if let super::ast::TypeRef::Table { .. } = &v.type_ref {
        return Ok(if value.is_array() { value.clone() } else { Value::Array(Vec::new()) });
    }

    if let super::ast::TypeRef::Scalar { name: sname, .. } = &v.type_ref {
        if matches!(sname.as_str(), "i" | "int8" | "decfloat16" | "decfloat34") {
            if value.is_null() || value.as_str().map(|s| s.trim().is_empty()).unwrap_or(false) {
                return Ok(serde_json::json!(0));
            }
            let n = to_number(value).map_err(|_| format!("Parameter '{}' requires a numeric value.", name))?;
            return Ok(if sname == "i" || sname == "int8" {
                serde_json::json!(n as i64)
            } else {
                serde_json::json!(n)
            });
        }
        if sname == "abap_bool" {
            if let Some(b) = value.as_bool() {
                return Ok(Value::Bool(b));
            }
            let s = format_value(value).to_ascii_uppercase();
            return Ok(Value::Bool(s == "X" || s == "TRUE" || s == "1"));
        }
    }

    Ok(Value::String(format_value(value)))
}

fn validate_selection_ranges(raw: &Value, meta: &VarIR) -> Result<(), String> {
    let arr = match raw {
        Value::Null => return Ok(()),
        Value::Array(a) => a,
        _ => return Err(format!("Selection '{}' must be a range table.", meta.name)),
    };

    let allowed: HashSet<&str> = if meta.no_intervals.unwrap_or(false) {
        ["EQ", "NE", "GT", "GE", "LT", "LE", "CP", "NP"].into_iter().collect()
    } else {
        ["EQ", "NE", "GT", "GE", "LT", "LE", "BT", "NB", "CP", "NP"].into_iter().collect()
    };

    for (index, row) in arr.iter().enumerate() {
        let low = row.get("low").or_else(|| row.get("LOW")).map(format_value).unwrap_or_default();
        let high = row.get("high").or_else(|| row.get("HIGH")).map(format_value).unwrap_or_default();
        if low.trim().is_empty() && high.trim().is_empty() {
            continue;
        }
        let option = row
            .get("option")
            .or_else(|| row.get("OPTION"))
            .map(|v| format_value(v).to_ascii_uppercase())
            .unwrap_or_else(|| "EQ".to_string());

        if !allowed.contains(option.as_str()) {
            return Err(format!("Selection '{}' row {}: unsupported option '{}'.", meta.name, index + 1, option));
        }
        if low.trim().is_empty() {
            return Err(format!("Selection '{}' row {}: a low value is required.", meta.name, index + 1));
        }
        if (option == "BT" || option == "NB") && high.trim().is_empty() {
            return Err(format!("Selection '{}' row {}: option {} requires a high value.", meta.name, index + 1, option));
        }
    }
    Ok(())
}

fn is_in_ranges(value: &Value, ranges: &Value) -> bool {
    let arr = match ranges {
        Value::Array(a) => a,
        _ => return false,
    };
    let active: Vec<&Value> = arr
        .iter()
        .filter(|r| {
            let low = r.get("LOW").or_else(|| r.get("low")).map(format_value).unwrap_or_default();
            let high = r.get("HIGH").or_else(|| r.get("high")).map(format_value).unwrap_or_default();
            !low.trim().is_empty() || !high.trim().is_empty()
        })
        .collect();

    if active.is_empty() {
        return true;
    }

    let includes: Vec<&&Value> = active
        .iter()
        .filter(|r| {
            let sign = r.get("SIGN").or_else(|| r.get("sign")).map(|v| format_value(v).to_ascii_uppercase()).unwrap_or_else(|| "I".to_string());
            sign != "E"
        })
        .collect();

    let excludes: Vec<&&Value> = active
        .iter()
        .filter(|r| {
            let sign = r.get("SIGN").or_else(|| r.get("sign")).map(|v| format_value(v).to_ascii_uppercase()).unwrap_or_else(|| "I".to_string());
            sign == "E"
        })
        .collect();

    let matches = |r: &Value| -> bool {
        let option = r.get("OPTION").or_else(|| r.get("option")).map(|v| format_value(v).to_ascii_uppercase()).unwrap_or_else(|| "EQ".to_string());
        let low = r.get("LOW").or_else(|| r.get("low")).cloned().unwrap_or(Value::Null);
        let high = r.get("HIGH").or_else(|| r.get("high")).cloned().unwrap_or(Value::Null);

        let cmp_low = range_compare(value, &low);
        let cmp_high = range_compare(value, &high);

        match option.as_str() {
            "EQ" => cmp_low == std::cmp::Ordering::Equal,
            "NE" => cmp_low != std::cmp::Ordering::Equal,
            "GT" => cmp_low == std::cmp::Ordering::Greater,
            "GE" => cmp_low != std::cmp::Ordering::Less,
            "LT" => cmp_low == std::cmp::Ordering::Less,
            "LE" => cmp_low != std::cmp::Ordering::Greater,
            "BT" => cmp_low != std::cmp::Ordering::Less && cmp_high != std::cmp::Ordering::Greater,
            "NB" => !(cmp_low != std::cmp::Ordering::Less && cmp_high != std::cmp::Ordering::Greater),
            "CP" => cp_match(&format_value(value), &format_value(&low)),
            "NP" => !cp_match(&format_value(value), &format_value(&low)),
            _ => false,
        }
    };

    let included = includes.is_empty() || includes.iter().any(|r| matches(r));
    included && !excludes.iter().any(|r| matches(r))
}

fn range_compare(value: &Value, bound: &Value) -> std::cmp::Ordering {
    if let Some(v_num) = value.as_f64() {
        if let Ok(b_num) = to_number(bound) {
            return v_num.partial_cmp(&b_num).unwrap_or(std::cmp::Ordering::Equal);
        }
    }
    format_value(value).cmp(&format_value(bound))
}

fn cp_match(value: &str, pattern: &str) -> bool {
    let mut regex_str = String::from("^");
    for ch in pattern.chars() {
        match ch {
            '*' => regex_str.push_str(".*"),
            '+' => regex_str.push('.'),
            _ => regex_str.push_str(&regex::escape(&ch.to_string())),
        }
    }
    regex_str.push('$');
    regex::Regex::new(&regex_str).map(|re| re.is_match(value)).unwrap_or(false)
}

fn eval_expr_where(
    expr: &super::ast::Expr,
    row: &Value,
    vars: &HashMap<String, Value>,
) -> Result<bool, String> {
    let val = eval_where_val(expr, row, vars)?;
    Ok(truthy(&val))
}

fn eval_where_val(
    expr: &super::ast::Expr,
    row: &Value,
    vars: &HashMap<String, Value>,
) -> Result<Value, String> {
    match expr {
        super::ast::Expr::Literal { value, .. } => Ok(value.clone()),
        super::ast::Expr::Variable { name, .. } => {
            let field_val = field_get(row, name);
            if !field_val.is_null() {
                Ok(field_val)
            } else if let Some(v) = vars.get(&name.to_ascii_uppercase()) {
                Ok(v.clone())
            } else {
                Ok(Value::Null)
            }
        }
        super::ast::Expr::Field { base, field, .. } => {
            let b = eval_where_val(base, row, vars)?;
            Ok(field_get(&b, field))
        }
        super::ast::Expr::Binary { op, left, right, .. } => {
            let l = eval_where_val(left, row, vars)?;
            let r = eval_where_val(right, row, vars)?;
            binary_op(op, &l, &r)
        }
        super::ast::Expr::Unary { op, operand, .. } => {
            let val = eval_where_val(operand, row, vars)?;
            if op == "NOT" {
                Ok(Value::Bool(!truthy(&val)))
            } else {
                let n = to_number(&val)?;
                Ok(serde_json::json!(-n))
            }
        }
        _ => Ok(Value::Null),
    }
}
