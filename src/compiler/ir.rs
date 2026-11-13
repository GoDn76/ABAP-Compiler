use super::ast::*;
use super::class_model::{build_class_models, ClassModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Value = serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputBinding {
    pub formal: String,
    pub target: String,
    pub changing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmlFilterRowNames {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmlModifyRowNames {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Op {
    #[serde(rename = "push")]
    Push { value: Value },
    #[serde(rename = "load")]
    Load { name: String },
    #[serde(rename = "store")]
    Store { name: String },
    #[serde(rename = "load_field")]
    LoadField { field: String },
    #[serde(rename = "store_field")]
    StoreField { field: String },
    #[serde(rename = "load_static")]
    LoadStatic {
        #[serde(rename = "className")]
        class_name: String,
        name: String,
    },
    #[serde(rename = "store_static")]
    StoreStatic {
        #[serde(rename = "className")]
        class_name: String,
        name: String,
    },
    #[serde(rename = "index_get")]
    IndexGet,
    #[serde(rename = "index_set")]
    IndexSet,
    #[serde(rename = "neg")]
    Neg,
    #[serde(rename = "not")]
    Not,
    #[serde(rename = "binary")]
    Binary { op: String },
    #[serde(rename = "call")]
    Call { name: String, argc: usize },
    #[serde(rename = "constructor")]
    Constructor { fields: Vec<String> },
    #[serde(rename = "table_constructor")]
    TableConstructor { count: usize },
    #[serde(rename = "raise")]
    Raise { exception_type: String },
    #[serde(rename = "write")]
    Write { groups: Vec<usize>, newline: bool },
    #[serde(rename = "jump_if_false")]
    JumpIfFalse { target: i64 },
    #[serde(rename = "jump")]
    Jump { target: i64 },
    #[serde(rename = "declare")]
    Declare { name: String },
    #[serde(rename = "apply_parameters")]
    ApplyParameters,
    #[serde(rename = "store_if_unset")]
    StoreIfUnset { name: String },
    #[serde(rename = "translate")]
    Translate {
        mode: String,
        #[serde(rename = "hasMask", default)]
        has_mask: bool,
    },
    #[serde(rename = "concatenate")]
    Concatenate {
        count: usize,
        #[serde(rename = "hasSeparator", default)]
        has_separator: bool,
        #[serde(rename = "respectingBlanks")]
        respecting_blanks: bool,
    },
    #[serde(rename = "split")]
    Split {
        #[serde(rename = "targetCount")]
        target_count: usize,
        #[serde(rename = "intoTable")]
        into_table: bool,
    },
    #[serde(rename = "condense")]
    Condense {
        #[serde(rename = "noGaps")]
        no_gaps: bool,
    },
    #[serde(rename = "message")]
    Message,
    #[serde(rename = "append")]
    Append,
    #[serde(rename = "insert")]
    Insert,
    #[serde(rename = "read_table")]
    ReadTable {
        #[serde(rename = "tableName", skip_serializing_if = "Option::is_none")]
        table_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        #[serde(default)]
        index: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<Vec<String>>,
    },
    #[serde(rename = "delete_table")]
    DeleteTable {
        #[serde(default)]
        index: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<Vec<String>>,
    },
    #[serde(rename = "sort")]
    Sort { keys: Vec<SortKey> },
    #[serde(rename = "describe")]
    Describe { target: String },
    #[serde(rename = "select")]
    Select {
        source: String,
        fields: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        single: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        where_clause: Option<Expr>,
    },
    #[serde(rename = "call_function")]
    CallFunction { name: String, params: Vec<String> },
    #[serde(rename = "call_method")]
    CallMethod {
        receiver: String,
        method: String,
        argc: usize,
        named: Vec<String>,
        #[serde(rename = "outputBindings", default)]
        output_bindings: Vec<OutputBinding>,
        #[serde(rename = "staticCall", default)]
        static_call: bool,
    },
    #[serde(rename = "clear")]
    Clear,
    #[serde(rename = "clear_var")]
    ClearVar { name: String },
    #[serde(rename = "assert")]
    Assert,
    #[serde(rename = "drop")]
    Drop,
    #[serde(rename = "nop")]
    Nop,
    #[serde(rename = "continue")]
    Continue,
    #[serde(rename = "exit_loop")]
    ExitLoop,
    #[serde(rename = "return")]
    Return,
    #[serde(rename = "new_object")]
    NewObject {
        #[serde(rename = "className")]
        class_name: String,
        #[serde(rename = "positionalCount")]
        positional_count: usize,
        named: Vec<String>,
    },
    #[serde(rename = "call_method_expr")]
    CallMethodExpr {
        method: String,
        #[serde(rename = "positionalCount")]
        positional_count: usize,
        named: Vec<String>,
        #[serde(rename = "outputBindings", default)]
        output_bindings: Vec<OutputBinding>,
    },
    #[serde(rename = "eml_read")]
    EmlRead {
        entity: String,
        #[serde(rename = "filterRows")]
        filter_rows: Vec<EmlFilterRowNames>,
        #[serde(rename = "resultTarget")]
        result_target: String,
        #[serde(rename = "failedTarget", skip_serializing_if = "Option::is_none")]
        failed_target: Option<String>,
        #[serde(rename = "reportedTarget", skip_serializing_if = "Option::is_none")]
        reported_target: Option<String>,
    },
    #[serde(rename = "eml_modify")]
    EmlModify {
        entity: String,
        operation: String,
        rows: Vec<EmlModifyRowNames>,
        #[serde(rename = "mappedTarget", skip_serializing_if = "Option::is_none")]
        mapped_target: Option<String>,
        #[serde(rename = "failedTarget", skip_serializing_if = "Option::is_none")]
        failed_target: Option<String>,
        #[serde(rename = "reportedTarget", skip_serializing_if = "Option::is_none")]
        reported_target: Option<String>,
    },
    #[serde(rename = "eml_commit")]
    EmlCommit,
    #[serde(rename = "eml_rollback")]
    EmlRollback,
    #[serde(rename = "push_handler")]
    PushHandler {
        #[serde(rename = "catchType")]
        catch_type: String,
        target: i64,
    },
    #[serde(rename = "pop_handler")]
    PopHandler,
    #[serde(rename = "assign_alias")]
    AssignAlias {
        source: String,
        target: String,
    },
    #[serde(rename = "assign_table_alias")]
    AssignTableAlias {
        target: String,
        table_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarIR {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    pub constant: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obligatory: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkbox: Option<bool>,
    #[serde(rename = "selectionOptions", skip_serializing_if = "Option::is_none")]
    pub selection_options: Option<bool>,
    #[serde(rename = "noIntervals", skip_serializing_if = "Option::is_none")]
    pub no_intervals: Option<bool>,
    #[serde(rename = "noExtension", skip_serializing_if = "Option::is_none")]
    pub no_extension: Option<bool>,
    #[serde(rename = "forField", skip_serializing_if = "Option::is_none")]
    pub for_field: Option<String>,
    #[serde(rename = "radiobuttonGroup", skip_serializing_if = "Option::is_none")]
    pub radiobutton_group: Option<String>,
    #[serde(rename = "lowerCase", skip_serializing_if = "Option::is_none")]
    pub lower_case: Option<bool>,
    #[serde(rename = "noDisplay", skip_serializing_if = "Option::is_none")]
    pub no_display: Option<bool>,
    #[serde(rename = "valueHelp", skip_serializing_if = "Option::is_none")]
    pub value_help: Option<String>,
    #[serde(rename = "hasDefault", skip_serializing_if = "Option::is_none")]
    pub has_default: Option<bool>,
    #[serde(rename = "initialValue", skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodIR {
    pub name: String,
    pub params: Vec<MethodParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returning: Option<ReturningParam>,
    pub vars: Vec<VarIR>,
    pub ops: Vec<Op>,
    #[serde(rename = "isStatic", skip_serializing_if = "Option::is_none")]
    pub is_static: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassAttributeIR {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    #[serde(rename = "isStatic")]
    pub is_static: bool,
    pub visibility: Visibility,
    pub constant: bool,
    #[serde(rename = "initialValue", skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassIR {
    pub name: String,
    #[serde(rename = "superClass", skip_serializing_if = "Option::is_none")]
    pub super_class: Option<String>,
    #[serde(rename = "isFinal")]
    pub is_final: bool,
    #[serde(rename = "isAbstract")]
    pub is_abstract: bool,
    pub interfaces: Vec<String>,
    pub types: HashMap<String, TypeRef>,
    pub attributes: HashMap<String, ClassAttributeIR>,
    pub methods: HashMap<String, MethodIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramIR {
    pub vars: Vec<VarIR>,
    pub ops: Vec<Op>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(rename = "entryPoint", skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<EntryPoint>,
    pub classes: HashMap<String, ClassIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    #[serde(rename = "className")]
    pub class_name: String,
    #[serde(rename = "methodName")]
    pub method_name: String,
}

pub struct Lowerer {
    pub ops: Vec<Op>,
    pub vars: Vec<VarIR>,
    pub types: HashMap<String, TypeRef>,
    pub temp_counter: usize,
    pub loop_ends: Vec<i64>,
    pub loop_continues: Vec<i64>,
    pub continue_placeholders: Vec<Vec<usize>>,
    pub program: Option<Program>,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            vars: Vec::new(),
            types: HashMap::new(),
            temp_counter: 0,
            loop_ends: Vec::new(),
            loop_continues: Vec::new(),
            continue_placeholders: Vec::new(),
            program: None,
        }
    }

    pub fn lower(
        mut self,
        program: &Program,
        mode: &str,
        requested_event: Option<RequestedEvent>,
        external_types: HashMap<String, TypeRef>,
    ) -> ProgramIR {
        self.program = Some(program.clone());
        for (name, ty) in external_types {
            self.types.insert(name.to_ascii_uppercase(), ty);
        }
        for t in &program.types {
            self.types.insert(t.name.to_ascii_uppercase(), t.type_ref.clone());
        }

        let models = build_class_models(program);
        let mut class_irs = HashMap::new();

        for (key, model) in &models {
            let mut local_types = self.types.clone();
            for t in &model.types {
                local_types.insert(t.name.to_ascii_uppercase(), t.type_ref.clone());
            }

            let mut methods = HashMap::new();
            for (method_key, method) in &model.methods {
                if !method.implemented.unwrap_or(false) {
                    continue;
                }
                let mut compiler = Lowerer::new();
                compiler.program = Some(program.clone());
                methods.insert(
                    method_key.clone(),
                    compiler.lower_method(method, &local_types, &models),
                );
            }

            let mut attributes = HashMap::new();
            for (attribute_key, attribute) in &model.attributes {
                let init_val = if let Some(Expr::Literal { value, .. }) = &attribute.value {
                    Some(value.clone())
                } else {
                    None
                };
                attributes.insert(
                    attribute_key.clone(),
                    ClassAttributeIR {
                        name: attribute.name.clone(),
                        type_ref: self.resolve_type(&attribute.type_ref),
                        is_static: attribute.is_static,
                        visibility: attribute.visibility,
                        constant: attribute.constant,
                        initial_value: init_val,
                    },
                );
            }

            class_irs.insert(
                key.clone(),
                ClassIR {
                    name: model.name.clone(),
                    super_class: model.super_class.clone(),
                    is_final: model.is_final,
                    is_abstract: model.is_abstract,
                    interfaces: model.interfaces.clone(),
                    types: local_types,
                    attributes,
                    methods,
                },
            );
        }

        let class_run = models.values().find(|c| {
            c.interfaces
                .iter()
                .any(|i| i.eq_ignore_ascii_case("IF_OO_ADT_CLASSRUN"))
                && c.methods
                    .get("IF_OO_ADT_CLASSRUN~MAIN")
                    .map(|m| m.implemented.unwrap_or(false))
                    .unwrap_or(false)
        });

        if mode == "cloud" && class_run.is_some() {
            let cr = class_run.unwrap();
            self.ops.push(Op::NewObject {
                class_name: cr.name.clone(),
                positional_count: 0,
                named: Vec::new(),
            });
            self.ops.push(Op::CallMethodExpr {
                method: "IF_OO_ADT_CLASSRUN~MAIN".to_string(),
                positional_count: 0,
                named: Vec::new(),
                output_bindings: Vec::new(),
            });
            self.ops.push(Op::Drop);
        } else if let Some(req) = requested_event {
            if mode == "standard" {
                self.lower_selection_event(&program.statements, &req);
            } else {
                self.lower_report(&program.statements);
            }
        } else if mode == "standard" && program.report_name.is_none() && class_run.is_some() {
            let cr = class_run.unwrap();
            self.ops.push(Op::NewObject {
                class_name: cr.name.clone(),
                positional_count: 0,
                named: Vec::new(),
            });
            self.ops.push(Op::CallMethodExpr {
                method: "IF_OO_ADT_CLASSRUN~MAIN".to_string(),
                positional_count: 0,
                named: Vec::new(),
                output_bindings: Vec::new(),
            });
            self.ops.push(Op::Drop);
        } else {
            let stmts = program.statements.clone();
            self.lower_report(&stmts);
        }

        let entry = class_run.map(|cr| EntryPoint {
            class_name: cr.name.clone(),
            method_name: "IF_OO_ADT_CLASSRUN~MAIN".to_string(),
        });

        ProgramIR {
            vars: self.vars,
            ops: self.ops,
            mode: Some(mode.to_string()),
            entry_point: entry,
            classes: class_irs,
        }
    }

    pub fn lower_method(
        &mut self,
        method: &MethodDecl,
        class_types: &HashMap<String, TypeRef>,
        _models: &HashMap<String, ClassModel>,
    ) -> MethodIR {
        self.types = class_types.clone();
        for p in &method.params {
            self.vars.push(VarIR {
                name: p.name.clone(),
                type_ref: self.resolve_type(&p.type_ref),
                constant: false,
                parameter: Some(true),
                obligatory: None,
                checkbox: None,
                selection_options: None,
                no_intervals: None,
                no_extension: None,
                for_field: None,
                radiobutton_group: None,
                lower_case: None,
                no_display: None,
                value_help: None,
                has_default: None,
                initial_value: None,
            });
            self.ops.push(Op::Declare { name: p.name.clone() });
        }
        if let Some(ret) = &method.returning {
            self.vars.push(VarIR {
                name: ret.name.clone(),
                type_ref: self.resolve_type(&ret.type_ref),
                constant: false,
                parameter: None,
                obligatory: None,
                checkbox: None,
                selection_options: None,
                no_intervals: None,
                no_extension: None,
                for_field: None,
                radiobutton_group: None,
                lower_case: None,
                no_display: None,
                value_help: None,
                has_default: None,
                initial_value: None,
            });
            self.ops.push(Op::Declare { name: ret.name.clone() });
        }
        for s in &method.statements {
            self.statement(s);
        }
        MethodIR {
            name: method.name.clone(),
            params: method.params.clone(),
            returning: method.returning.clone(),
            vars: self.vars.clone(),
            ops: self.ops.clone(),
            is_static: method.is_static,
            visibility: method.visibility,
        }
    }

    fn lower_report(&mut self, input: &[Statement]) {
        let events: Vec<&Statement> = input.iter().filter(|s| matches!(s, Statement::EventBlock { .. })).collect();
        let ordinary: Vec<&Statement> = input.iter().filter(|s| !matches!(s, Statement::EventBlock { .. })).collect();

        let is_decl = |s: &&Statement| match s {
            Statement::DataDecl { .. }
            | Statement::ConstantDecl { .. }
            | Statement::TypeDecl(..)
            | Statement::ClassDecl(..)
            | Statement::EmptyStmt { .. } => true,
            _ => false,
        };

        let declarations: Vec<&Statement> = ordinary.iter().copied().filter(is_decl).collect();
        let executable: Vec<&Statement> = ordinary.iter().copied().filter(|s| !is_decl(s)).collect();

        let bodies = |kinds: &[&str]| -> Vec<Statement> {
            let mut res = Vec::new();
            for e in &events {
                if let Statement::EventBlock { event, statements, .. } = e {
                    if kinds.contains(&event.as_str()) {
                        res.extend(statements.clone());
                    }
                }
            }
            res
        };

        for s in declarations {
            self.statement(s);
        }
        for s in bodies(&["initialization"]) {
            self.statement(&s);
        }
        for s in bodies(&["at-selection-screen-output"]) {
            self.statement(&s);
        }
        self.ops.push(Op::ApplyParameters);
        for s in bodies(&["at-selection-screen", "at-selection-screen-on"]) {
            self.statement(&s);
        }
        for s in executable {
            self.statement(s);
        }
        for s in bodies(&["start-of-selection"]) {
            self.statement(&s);
        }
        for s in bodies(&["end-of-selection"]) {
            self.statement(&s);
        }
    }

    fn lower_selection_event(&mut self, input: &[Statement], requested: &RequestedEvent) {
        for s in input {
            match s {
                Statement::DataDecl { .. }
                | Statement::ConstantDecl { .. }
                | Statement::TypeDecl(..)
                | Statement::ClassDecl(..)
                | Statement::EmptyStmt { .. } => {
                    self.statement(s);
                }
                _ => {}
            }
        }
        self.ops.push(Op::ApplyParameters);
        let target = requested.target.as_deref().unwrap_or("").to_ascii_uppercase();

        let block = input.iter().find(|s| {
            if let Statement::EventBlock { event, target: block_target, .. } = s {
                if event == &requested.event {
                    if target.is_empty() {
                        return true;
                    }
                    let bt = block_target.as_deref().unwrap_or("").to_ascii_uppercase();
                    return bt == target || bt.starts_with(&format!("{}-", target));
                }
            }
            false
        });

        if let Some(Statement::EventBlock { statements, .. }) = block {
            for s in statements {
                self.statement(s);
            }
        }
    }

    fn statement(&mut self, s: &Statement) {
        match s {
            Statement::DataDecl {
                name,
                type_ref,
                value,
                parameter,
                obligatory,
                checkbox,
                selection_options,
                no_intervals,
                no_extension,
                for_field,
                radiobutton_group,
                lower_case,
                no_display,
                value_help,
                ..
            } => {
                let init_val = if let Some(Expr::Literal { value: val, .. }) = value {
                    Some(val.clone())
                } else {
                    None
                };
                self.vars.push(VarIR {
                    name: name.clone(),
                    type_ref: self.resolve_type(type_ref),
                    constant: false,
                    parameter: *parameter,
                    obligatory: *obligatory,
                    checkbox: *checkbox,
                    selection_options: *selection_options,
                    no_intervals: *no_intervals,
                    no_extension: *no_extension,
                    for_field: for_field.clone(),
                    radiobutton_group: radiobutton_group.clone(),
                    lower_case: *lower_case,
                    no_display: *no_display,
                    value_help: value_help.clone(),
                    has_default: Some(value.is_some()),
                    initial_value: init_val,
                });
                self.ops.push(Op::Declare { name: name.clone() });
                if let Some(v) = value {
                    self.expr(v);
                    self.ops.push(if parameter.unwrap_or(false) {
                        Op::StoreIfUnset { name: name.clone() }
                    } else {
                        Op::Store { name: name.clone() }
                    });
                }
            }
            Statement::InlineDataAssign { name, value, .. } => {
                let inferred = self.infer_expr_type(value);
                self.vars.push(VarIR {
                    name: name.clone(),
                    type_ref: self.resolve_type(&inferred),
                    constant: false,
                    parameter: None,
                    obligatory: None,
                    checkbox: None,
                    selection_options: None,
                    no_intervals: None,
                    no_extension: None,
                    for_field: None,
                    radiobutton_group: None,
                    lower_case: None,
                    no_display: None,
                    value_help: None,
                    has_default: None,
                    initial_value: None,
                });
                self.ops.push(Op::Declare { name: name.clone() });
                self.expr(value);
                self.ops.push(Op::Store { name: name.clone() });
            }
            Statement::ConstantDecl { name, type_ref, value, .. } => {
                self.vars.push(VarIR {
                    name: name.clone(),
                    type_ref: self.resolve_type(type_ref),
                    constant: true,
                    parameter: None,
                    obligatory: None,
                    checkbox: None,
                    selection_options: None,
                    no_intervals: None,
                    no_extension: None,
                    for_field: None,
                    radiobutton_group: None,
                    lower_case: None,
                    no_display: None,
                    value_help: None,
                    has_default: None,
                    initial_value: None,
                });
                self.expr(value);
                self.ops.push(Op::Store { name: name.clone() });
            }
            Statement::Assignment { target, value, .. } => {
                self.expr(value);
                match target {
                    Expr::Variable { name, .. } => {
                        self.ops.push(Op::Store { name: name.clone() });
                    }
                    Expr::Field { base, field, static_selector, .. } => {
                        if static_selector.unwrap_or(false) {
                            if let Expr::Variable { name: base_name, .. } = &**base {
                                self.ops.push(Op::StoreStatic {
                                    class_name: base_name.clone(),
                                    name: field.clone(),
                                });
                            }
                        } else {
                            self.expr(base);
                            self.ops.push(Op::StoreField { field: field.clone() });
                            if let Expr::Variable { name: base_name, .. } = &**base {
                                self.ops.push(Op::Store { name: base_name.clone() });
                            }
                        }
                    }
                    Expr::Index { base, index, .. } => {
                        self.expr(base);
                        self.expr(index);
                        self.ops.push(Op::IndexSet);
                        if let Expr::Variable { name: base_name, .. } = &**base {
                            self.ops.push(Op::Store { name: base_name.clone() });
                        }
                    }
                    _ => {}
                }
            }
            Statement::Assign { source, target, .. } => {
                let src_name = match source {
                    Expr::Variable { name, .. } => name.clone(),
                    Expr::Field { base, field, .. } => {
                        if let Expr::Variable { name: b, .. } = &**base {
                            format!("{}-{}", b, field)
                        } else {
                            String::new()
                        }
                    }
                    _ => String::new(),
                };
                self.ops.push(Op::AssignAlias {
                    source: src_name,
                    target: target.clone(),
                });
            }
            Statement::WriteStmt { values, groups, newline, .. } => {
                for v in values {
                    self.expr(v);
                }
                self.ops.push(Op::Write {
                    groups: groups.iter().map(|g| g.len()).collect(),
                    newline: *newline,
                });
            }
            Statement::IfStmt { branches, else_branch, .. } => {
                self.lower_if(branches, else_branch);
            }
            Statement::WhileStmt { condition, body, .. } => {
                let loop_top = self.ops.len() as i64;
                self.expr(condition);
                let br = self.ops.len();
                self.ops.push(Op::JumpIfFalse { target: -1 });
                self.loop_ends.push(-1);
                self.loop_continues.push(loop_top);
                for x in body {
                    self.statement(x);
                }
                let cont = self.loop_continues.pop().unwrap();
                self.ops.push(Op::Jump { target: cont });
                let end = self.ops.len() as i64;
                self.loop_ends.pop();
                self.patch(br, end);
                self.patch_exit_placeholders(end);
            }
            Statement::DoStmt { count, body, .. } => {
                if let Some(c) = count {
                    let ctr = format!("__abapc_do_{}", self.temp_counter);
                    self.temp_counter += 1;
                    self.vars.push(VarIR {
                        name: ctr.clone(),
                        type_ref: TypeRef::scalar("i"),
                        constant: false,
                        parameter: None,
                        obligatory: None,
                        checkbox: None,
                        selection_options: None,
                        no_intervals: None,
                        no_extension: None,
                        for_field: None,
                        radiobutton_group: None,
                        lower_case: None,
                        no_display: None,
                        value_help: None,
                        has_default: None,
                        initial_value: None,
                    });
                    self.ops.push(Op::Declare { name: ctr.clone() });
                    self.expr(c);
                    self.ops.push(Op::Store { name: ctr.clone() });

                    let top = self.ops.len() as i64;
                    self.ops.push(Op::Load { name: ctr.clone() });
                    self.ops.push(Op::Push { value: serde_json::json!(0) });
                    self.ops.push(Op::Binary { op: ">".to_string() });

                    let br = self.ops.len();
                    self.ops.push(Op::JumpIfFalse { target: -1 });
                    self.loop_ends.push(-1);
                    self.loop_continues.push(-1);

                    for x in body {
                        self.statement(x);
                    }
                    let cont = self.ops.len() as i64;
                    let last_idx = self.loop_continues.len() - 1;
                    self.loop_continues[last_idx] = cont;

                    self.ops.push(Op::Load { name: ctr.clone() });
                    self.ops.push(Op::Push { value: serde_json::json!(1) });
                    self.ops.push(Op::Binary { op: "-".to_string() });
                    self.ops.push(Op::Store { name: ctr });
                    self.ops.push(Op::Jump { target: top });

                    let end = self.ops.len() as i64;
                    self.loop_ends.pop();
                    self.loop_continues.pop();
                    self.patch(br, end);
                    self.patch_exit_placeholders(end);
                } else {
                    let top = self.ops.len() as i64;
                    self.loop_ends.push(-1);
                    self.loop_continues.push(top);
                    for x in body {
                        self.statement(x);
                    }
                    self.ops.push(Op::Jump { target: top });
                    let end = self.ops.len() as i64;
                    self.loop_ends.pop();
                    self.loop_continues.pop();
                    self.patch_exit_placeholders(end);
                }
            }
            Statement::CaseStmt { expression, whens, others, .. } => {
                let tmp = format!("__abapc_case_{}", self.temp_counter);
                self.temp_counter += 1;
                self.vars.push(VarIR {
                    name: tmp.clone(),
                    type_ref: TypeRef::any(),
                    constant: false,
                    parameter: None,
                    obligatory: None,
                    checkbox: None,
                    selection_options: None,
                    no_intervals: None,
                    no_extension: None,
                    for_field: None,
                    radiobutton_group: None,
                    lower_case: None,
                    no_display: None,
                    value_help: None,
                    has_default: None,
                    initial_value: None,
                });
                self.ops.push(Op::Declare { name: tmp.clone() });
                self.expr(expression);
                self.ops.push(Op::Store { name: tmp.clone() });

                let mut end_jumps = Vec::new();
                for w in whens {
                    for v in &w.values {
                        self.ops.push(Op::Load { name: tmp.clone() });
                        self.expr(v);
                        self.ops.push(Op::Binary { op: "=".to_string() });
                        let br = self.ops.len();
                        self.ops.push(Op::JumpIfFalse { target: -1 });
                        for x in &w.statements {
                            self.statement(x);
                        }
                        let j = self.ops.len();
                        self.ops.push(Op::Jump { target: -1 });
                        end_jumps.push(j);
                        self.patch(br, self.ops.len() as i64);
                    }
                }
                for x in others {
                    self.statement(x);
                }
                let end = self.ops.len() as i64;
                for j in end_jumps {
                    self.patch(j, end);
                }
            }
            Statement::LoopStmt { source, target, inline, body, .. } => {
                if let Expr::Variable { name, .. } = source {
                    self.ops.push(Op::Push {
                        value: Value::String(name.clone()),
                    });
                    if let Some(t) = target {
                        self.ops.push(Op::Push {
                            value: Value::String(t.clone()),
                        });
                        self.ops.push(Op::Call {
                            name: "__iter_start".to_string(),
                            argc: 2,
                        });
                    } else {
                        self.ops.push(Op::Call {
                            name: "__iter_start".to_string(),
                            argc: 1,
                        });
                    }
                } else {
                    self.expr(source);
                    self.ops.push(Op::Call {
                        name: "__iter_start".to_string(),
                        argc: 1,
                    });
                }
                self.ops.push(Op::Drop);

                let top = self.ops.len() as i64;
                self.ops.push(Op::Call {
                    name: "__iter_has_next".to_string(),
                    argc: 0,
                });
                let br = self.ops.len();
                self.ops.push(Op::JumpIfFalse { target: -1 });
                self.loop_ends.push(-1);
                self.loop_continues.push(-1);
                self.continue_placeholders.push(Vec::new());

                if let Some(t) = target {
                    self.ops.push(Op::Call {
                        name: "__iter_value".to_string(),
                        argc: 0,
                    });
                    if t.starts_with('<') && t.ends_with('>') {
                        if let Expr::Variable { name: ref tbl_name, .. } = source {
                            self.ops.push(Op::AssignTableAlias {
                                target: t.clone(),
                                table_name: tbl_name.clone(),
                            });
                        }
                    }
                    self.ops.push(Op::Store { name: t.clone() });
                    if inline.unwrap_or(false) {
                        self.vars.push(VarIR {
                            name: t.clone(),
                            type_ref: TypeRef::any(),
                            constant: false,
                            parameter: None,
                            obligatory: None,
                            checkbox: None,
                            selection_options: None,
                            no_intervals: None,
                            no_extension: None,
                            for_field: None,
                            radiobutton_group: None,
                            lower_case: None,
                            no_display: None,
                            value_help: None,
                            has_default: None,
                            initial_value: None,
                        });
                    }
                }

                for x in body {
                    self.statement(x);
                }
                let cont = self.ops.len() as i64;
                let last_idx = self.loop_continues.len() - 1;
                self.loop_continues[last_idx] = cont;
                if let Some(c_list) = self.continue_placeholders.pop() {
                    for idx in c_list {
                        self.patch(idx, cont);
                    }
                }

                self.ops.push(Op::Call {
                    name: "__iter_next".to_string(),
                    argc: 0,
                });
                self.ops.push(Op::Drop);
                self.ops.push(Op::Jump { target: top });

                let end = self.ops.len() as i64;
                self.loop_ends.pop();
                self.loop_continues.pop();
                self.patch(br, end);
                self.patch_exit_placeholders(end);
            }
            Statement::AppendStmt { value, table, .. } => {
                self.expr(value);
                self.expr(table);
                self.ops.push(Op::Append);
                if let Expr::Variable { name, .. } = table {
                    self.ops.push(Op::Store { name: name.clone() });
                }
            }
            Statement::InsertStmt { value, table, .. } => {
                self.expr(value);
                self.expr(table);
                self.ops.push(Op::Insert);
                if let Expr::Variable { name, .. } = table {
                    self.ops.push(Op::Store { name: name.clone() });
                }
            }
            Statement::ReadTableStmt { table, target, inline, index, key, .. } => {
                if let Some(t) = target {
                    if inline.unwrap_or(false) {
                        self.vars.push(VarIR {
                            name: t.clone(),
                            type_ref: TypeRef::any(),
                            constant: false,
                            parameter: None,
                            obligatory: None,
                            checkbox: None,
                            selection_options: None,
                            no_intervals: None,
                            no_extension: None,
                            for_field: None,
                            radiobutton_group: None,
                            lower_case: None,
                            no_display: None,
                            value_help: None,
                            has_default: None,
                            initial_value: None,
                        });
                    }
                }
                let tbl_name = match table {
                    Expr::Variable { name, .. } => Some(name.clone()),
                    _ => None,
                };
                if let Some(idx) = index {
                    self.expr(table);
                    self.expr(idx);
                    self.ops.push(Op::ReadTable {
                        table_name: tbl_name.clone(),
                        target: target.clone(),
                        index: true,
                        key: None,
                    });
                } else {
                    self.expr(table);
                    let keys: Vec<String> = key.as_ref().map(|k| k.keys().cloned().collect()).unwrap_or_default();
                    for k in &keys {
                        if let Some(ev) = key.as_ref().and_then(|m| m.get(k)) {
                            self.expr(ev);
                        } else {
                            self.ops.push(Op::Push { value: serde_json::Value::Null });
                        }
                    }
                    self.ops.push(Op::ReadTable {
                        table_name: tbl_name,
                        target: target.clone(),
                        index: false,
                        key: if !keys.is_empty() { Some(keys) } else { None },
                    });
                }
            }
            Statement::DeleteTableStmt { table, index, key, .. } => {
                if let Some(idx) = index {
                    self.expr(table);
                    self.expr(idx);
                    self.ops.push(Op::DeleteTable {
                        index: true,
                        key: None,
                    });
                } else {
                    self.expr(table);
                    let keys: Vec<String> = key.as_ref().map(|k| k.keys().cloned().collect()).unwrap_or_default();
                    for k in &keys {
                        if let Some(ev) = key.as_ref().and_then(|m| m.get(k)) {
                            self.expr(ev);
                        } else {
                            self.ops.push(Op::Push { value: serde_json::Value::Null });
                        }
                    }
                    self.ops.push(Op::DeleteTable {
                        index: false,
                        key: if !keys.is_empty() { Some(keys) } else { None },
                    });
                }
            }
            Statement::SortStmt { table, keys, .. } => {
                self.expr(table);
                self.ops.push(Op::Sort { keys: keys.clone() });
            }
            Statement::DescribeTableStmt { table, target, .. } => {
                self.expr(table);
                self.ops.push(Op::Describe { target: target.clone() });
            }
            Statement::ClearStmt { target, .. } => {
                if let Expr::Variable { name, .. } = target {
                    self.ops.push(Op::ClearVar { name: name.clone() });
                } else {
                    self.expr(target);
                    self.ops.push(Op::Clear);
                }
            }
            Statement::TranslateStmt { target, mode, mask, .. } => {
                self.expr(target);
                if let Some(m) = mask {
                    self.expr(m);
                }
                self.ops.push(Op::Translate {
                    mode: mode.clone(),
                    has_mask: mask.is_some(),
                });
                self.store_target(target);
            }
            Statement::ConcatenateStmt { sources, target, separator, respecting_blanks, .. } => {
                for e in sources {
                    self.expr(e);
                }
                if let Some(sep) = separator {
                    self.expr(sep);
                }
                self.ops.push(Op::Concatenate {
                    count: sources.len(),
                    has_separator: separator.is_some(),
                    respecting_blanks: *respecting_blanks,
                });
                self.store_target(target);
            }
            Statement::SplitStmt { source, separator, targets, table_target, .. } => {
                self.expr(source);
                self.expr(separator);
                self.ops.push(Op::Split {
                    target_count: targets.len(),
                    into_table: table_target.is_some(),
                });
                if let Some(tt) = table_target {
                    self.store_target(tt);
                } else {
                    let temp = format!("__abapc_split_{}", self.temp_counter);
                    self.temp_counter += 1;
                    self.vars.push(VarIR {
                        name: temp.clone(),
                        type_ref: TypeRef::Table {
                            line: Box::new(TypeRef::scalar("string")),
                            table_kind: "standard".to_string(),
                            unique: None,
                        },
                        constant: false,
                        parameter: None,
                        obligatory: None,
                        checkbox: None,
                        selection_options: None,
                        no_intervals: None,
                        no_extension: None,
                        for_field: None,
                        radiobutton_group: None,
                        lower_case: None,
                        no_display: None,
                        value_help: None,
                        has_default: None,
                        initial_value: None,
                    });
                    self.ops.push(Op::Declare { name: temp.clone() });
                    self.ops.push(Op::Store { name: temp.clone() });
                    for (i, t) in targets.iter().enumerate() {
                        self.ops.push(Op::Load { name: temp.clone() });
                        self.ops.push(Op::Push { value: serde_json::json!(i + 1) });
                        self.ops.push(Op::IndexGet);
                        self.store_target(t);
                    }
                }
            }
            Statement::CondenseStmt { target, no_gaps, .. } => {
                self.expr(target);
                self.ops.push(Op::Condense { no_gaps: *no_gaps });
                self.store_target(target);
            }
            Statement::MessageStmt { text, message_type, .. } => {
                self.expr(text);
                self.expr(message_type);
                self.ops.push(Op::Message);
            }
            Statement::EventBlock { statements, .. } => {
                for x in statements {
                    self.statement(x);
                }
            }
            Statement::FlowStmt { op, condition, .. } => {
                match op.as_str() {
                    "continue" => {
                        if let Some(c_list) = self.continue_placeholders.last_mut() {
                            let idx = self.ops.len();
                            self.ops.push(Op::Jump { target: -1 });
                            c_list.push(idx);
                        } else {
                            let target = self.loop_continues.last().copied().unwrap_or(self.ops.len() as i64);
                            self.ops.push(Op::Jump { target });
                        }
                    }
                    "exit" => {
                        let target = self.loop_ends.last().copied().unwrap_or(self.ops.len() as i64);
                        self.ops.push(Op::Jump { target });
                    }
                    "return" => {
                        self.ops.push(Op::Return);
                    }
                    "check" => {
                        if let Some(c) = condition {
                            self.expr(c);
                        } else {
                            self.ops.push(Op::Push { value: serde_json::json!(false) });
                        }
                        if let Some(c_list) = self.continue_placeholders.last_mut() {
                            let idx = self.ops.len();
                            self.ops.push(Op::JumpIfFalse { target: -1 });
                            c_list.push(idx);
                        } else {
                            let target = self.loop_continues.last().copied().unwrap_or(self.ops.len() as i64);
                            self.ops.push(Op::JumpIfFalse { target });
                        }
                    }
                    _ => {}
                }
            }
            Statement::AssertStmt { condition, .. } => {
                self.expr(condition);
                self.ops.push(Op::Assert);
            }
            Statement::CallFunction { name, parameters, .. } => {
                let params: Vec<String> = parameters.keys().cloned().collect();
                for k in &params {
                    self.expr(&parameters[k]);
                }
                self.ops.push(Op::CallFunction {
                    name: name.clone(),
                    params,
                });
            }
            Statement::MethodCallStmt {
                receiver,
                method,
                args,
                named,
                output_named,
                changing_named,
                static_call,
                ..
            }
            | Statement::MethodCall {
                receiver,
                method,
                args,
                named,
                output_named,
                changing_named,
                static_call,
                ..
            } => {
                for e in args {
                    self.expr(e);
                }
                for e in named.values() {
                    self.expr(e);
                }
                if let Some(cn) = changing_named {
                    for e in cn.values() {
                        self.expr(e);
                    }
                }
                let mut bindings = Vec::new();
                if let Some(on) = output_named {
                    for (formal, target_expr) in on {
                        if let Expr::Variable { name, .. } = target_expr {
                            bindings.push(OutputBinding {
                                formal: formal.clone(),
                                target: name.clone(),
                                changing: false,
                            });
                        }
                    }
                }
                if let Some(cn) = changing_named {
                    for (formal, target_expr) in cn {
                        if let Expr::Variable { name, .. } = target_expr {
                            bindings.push(OutputBinding {
                                formal: formal.clone(),
                                target: name.clone(),
                                changing: true,
                            });
                        }
                    }
                }
                let mut named_keys: Vec<String> = named.keys().cloned().collect();
                if let Some(cn) = changing_named {
                    named_keys.extend(cn.keys().cloned());
                }
                let argc = args.len() + named.len() + changing_named.as_ref().map(|m| m.len()).unwrap_or(0);

                self.ops.push(Op::CallMethod {
                    receiver: receiver.clone(),
                    method: method.clone(),
                    argc,
                    named: named_keys,
                    output_bindings: bindings,
                    static_call: static_call.unwrap_or(false),
                });
            }
            Statement::RaiseStmt { exception_type, .. } => {
                self.ops.push(Op::Raise {
                    exception_type: exception_type.clone(),
                });
            }
            Statement::EmlReadStmt {
                entity,
                filters,
                result_target,
                failed_target,
                reported_target,
                ..
            } => {
                self.ensure_eml_target(result_target);
                if let Some(ft) = failed_target {
                    self.ensure_eml_target(ft);
                }
                if let Some(rt) = reported_target {
                    self.ensure_eml_target(rt);
                }
                for row in filters {
                    for f in &row.fields {
                        self.expr(&f.value);
                    }
                }
                let filter_rows = filters
                    .iter()
                    .map(|r| EmlFilterRowNames {
                        names: r.fields.iter().map(|f| f.name.clone()).collect(),
                    })
                    .collect();
                self.ops.push(Op::EmlRead {
                    entity: entity.clone(),
                    filter_rows,
                    result_target: result_target.clone(),
                    failed_target: failed_target.clone(),
                    reported_target: reported_target.clone(),
                });
            }
            Statement::EmlModifyStmt {
                entity,
                operation,
                rows,
                mapped_target,
                failed_target,
                reported_target,
                ..
            } => {
                if let Some(mt) = mapped_target {
                    self.ensure_eml_target(mt);
                }
                if let Some(ft) = failed_target {
                    self.ensure_eml_target(ft);
                }
                if let Some(rt) = reported_target {
                    self.ensure_eml_target(rt);
                }
                for row in rows {
                    for f in &row.fields {
                        self.expr(&f.value);
                    }
                }
                let mod_rows = rows
                    .iter()
                    .map(|r| EmlModifyRowNames {
                        names: r.fields.iter().map(|f| f.name.clone()).collect(),
                    })
                    .collect();
                self.ops.push(Op::EmlModify {
                    entity: entity.clone(),
                    operation: operation.clone(),
                    rows: mod_rows,
                    mapped_target: mapped_target.clone(),
                    failed_target: failed_target.clone(),
                    reported_target: reported_target.clone(),
                });
            }
            Statement::EmlTransactionStmt { operation, .. } => {
                self.ops.push(if operation == "COMMIT" {
                    Op::EmlCommit
                } else {
                    Op::EmlRollback
                });
            }
            Statement::SelectStmt {
                source,
                fields,
                target,
                single,
                where_clause,
                ..
            } => {
                self.ops.push(Op::Select {
                    source: source.clone(),
                    fields: fields.clone(),
                    target: target.clone(),
                    single: *single,
                    where_clause: where_clause.clone(),
                });
                if let Some(t) = target {
                    self.vars.push(VarIR {
                        name: t.clone(),
                        type_ref: TypeRef::any(),
                        constant: false,
                        parameter: None,
                        obligatory: None,
                        checkbox: None,
                        selection_options: None,
                        no_intervals: None,
                        no_extension: None,
                        for_field: None,
                        radiobutton_group: None,
                        lower_case: None,
                        no_display: None,
                        value_help: None,
                        has_default: None,
                        initial_value: None,
                    });
                }
            }
            Statement::TryStmt { body, catches, .. } => {
                let mut handler_indices = Vec::new();
                for c in catches {
                    let idx = self.ops.len();
                    self.ops.push(Op::PushHandler {
                        catch_type: c.catch_type.clone(),
                        target: -1,
                    });
                    handler_indices.push((idx, c));
                }

                for x in body {
                    self.statement(x);
                }

                for _ in catches {
                    self.ops.push(Op::PopHandler);
                }

                let end_jump = self.ops.len();
                self.ops.push(Op::Jump { target: -1 });

                let mut catch_end_jumps = Vec::new();
                for (handler_idx, c) in handler_indices {
                    let catch_start = self.ops.len() as i64;
                    self.patch(handler_idx, catch_start);
                    if let Some(ref v) = c.variable {
                        self.ops.push(Op::Store { name: v.clone() });
                    }
                    for s in &c.body {
                        self.statement(s);
                    }
                    let ce_jump = self.ops.len();
                    self.ops.push(Op::Jump { target: -1 });
                    catch_end_jumps.push(ce_jump);
                }

                let end = self.ops.len() as i64;
                self.patch(end_jump, end);
                for j in catch_end_jumps {
                    self.patch(j, end);
                }
            }
            Statement::ClassDecl(..)
            | Statement::TypeDecl(..)
            | Statement::InterfaceDecl(..)
            | Statement::ClassDefStmt(..)
            | Statement::ClassImplStmt(..)
            | Statement::EmptyStmt { .. } => {
                self.ops.push(Op::Nop);
            }
            Statement::CreateObject(co) => {
                let class_name = if let Some(cls) = &co.class_name {
                    cls.clone()
                } else if let Some(v) = self.vars.iter().find(|x| x.name.eq_ignore_ascii_case(&co.target)) {
                    match &v.type_ref {
                        TypeRef::Object { class_name } => class_name.clone(),
                        _ => co.target.clone(),
                    }
                } else {
                    co.target.clone()
                };
                for a in co.exporting.values() {
                    self.expr(a);
                }
                self.ops.push(Op::NewObject {
                    class_name,
                    positional_count: 0,
                    named: co.exporting.keys().cloned().collect(),
                });
                self.ops.push(Op::Store {
                    name: co.target.clone(),
                });
            }
        }
    }

    fn lower_switch(&mut self, expr: &Expr) {
        if let Expr::Switch {
            result_type,
            infer_type: _,
            operand,
            branches,
            else_result,
            ..
        } = expr
        {
            let temp = format!("__abapc_switch_{}", self.temp_counter);
            self.temp_counter += 1;
            let res_ty = if let Some(rt) = result_type {
                self.resolve_type(rt)
            } else {
                self.infer_expr_type(expr)
            };

            self.vars.push(VarIR {
                name: temp.clone(),
                type_ref: TypeRef::any(),
                constant: false,
                parameter: None,
                obligatory: None,
                checkbox: None,
                selection_options: None,
                no_intervals: None,
                no_extension: None,
                for_field: None,
                radiobutton_group: None,
                lower_case: None,
                no_display: None,
                value_help: None,
                has_default: None,
                initial_value: None,
            });
            self.ops.push(Op::Declare { name: temp.clone() });
            self.expr(operand);
            self.ops.push(Op::Store { name: temp.clone() });

            let mut end_jumps = Vec::new();
            for branch in branches {
                for value in &branch.values {
                    self.ops.push(Op::Load { name: temp.clone() });
                    self.expr(value);
                    self.ops.push(Op::Binary { op: "=".to_string() });
                    let skip = self.ops.len();
                    self.ops.push(Op::JumpIfFalse { target: -1 });
                    self.expr(&branch.result);
                    let finish = self.ops.len();
                    self.ops.push(Op::Jump { target: -1 });
                    end_jumps.push(finish);
                    self.patch(skip, self.ops.len() as i64);
                }
            }

            if let Some(el) = else_result {
                self.expr(el);
            } else {
                self.ops.push(Op::Push {
                    value: initial_value_for_type(&res_ty),
                });
            }

            let end = self.ops.len() as i64;
            for j in end_jumps {
                self.patch(j, end);
            }
        }
    }

    fn ensure_eml_target(&mut self, name: &str) {
        if !self.vars.iter().any(|v| v.name.eq_ignore_ascii_case(name)) {
            self.vars.push(VarIR {
                name: name.to_string(),
                type_ref: TypeRef::Table {
                    line: Box::new(TypeRef::any()),
                    table_kind: "standard".to_string(),
                    unique: None,
                },
                constant: false,
                parameter: None,
                obligatory: None,
                checkbox: None,
                selection_options: None,
                no_intervals: None,
                no_extension: None,
                for_field: None,
                radiobutton_group: None,
                lower_case: None,
                no_display: None,
                value_help: None,
                has_default: None,
                initial_value: None,
            });
            self.ops.push(Op::Declare { name: name.to_string() });
        }
    }

    fn store_target(&mut self, target: &Expr) {
        match target {
            Expr::Variable { name, .. } => {
                self.ops.push(Op::Store { name: name.clone() });
            }
            Expr::Field { base, field, .. } => {
                self.expr(base);
                self.ops.push(Op::StoreField { field: field.clone() });
            }
            Expr::Index { base, index, .. } => {
                self.expr(base);
                self.expr(index);
                self.ops.push(Op::IndexSet);
            }
            _ => {}
        }
    }

    fn lower_if(&mut self, branches: &[IfBranch], else_branch: &[Statement]) {
        let mut end_jumps = Vec::new();
        for b in branches {
            self.expr(&b.condition);
            let br = self.ops.len();
            self.ops.push(Op::JumpIfFalse { target: -1 });
            for x in &b.statements {
                self.statement(x);
            }
            let j = self.ops.len();
            self.ops.push(Op::Jump { target: -1 });
            end_jumps.push(j);
            self.patch(br, self.ops.len() as i64);
        }
        for x in else_branch {
            self.statement(x);
        }
        let end = self.ops.len() as i64;
        for j in end_jumps {
            self.patch(j, end);
        }
    }

    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Literal { value, .. } => {
                self.ops.push(Op::Push { value: value.clone() });
            }
            Expr::Variable { name, .. } => {
                if let Some(prog) = &self.program {
                    if prog.classes.iter().any(|c| c.name.eq_ignore_ascii_case(name)) {
                        let mut map = serde_json::Map::new();
                        map.insert("__CLASS".to_string(), serde_json::json!(name.to_ascii_uppercase()));
                        map.insert("__STATIC_CONTEXT".to_string(), serde_json::json!(true));
                        self.ops.push(Op::Push { value: serde_json::Value::Object(map) });
                        return;
                    }
                }
                self.ops.push(Op::Load { name: name.clone() });
            }
            Expr::Field { base, field, static_selector, .. } => {
                if static_selector.unwrap_or(false) {
                    if let Expr::Variable { name: base_name, .. } = &**base {
                        self.ops.push(Op::LoadStatic {
                            class_name: base_name.clone(),
                            name: field.clone(),
                        });
                        return;
                    }
                }
                self.expr(base);
                self.ops.push(Op::LoadField { field: field.clone() });
            }
            Expr::Index { base, index, .. } => {
                self.expr(base);
                self.expr(index);
                self.ops.push(Op::IndexGet);
            }
            Expr::Unary { op, operand, .. } => {
                self.expr(operand);
                self.ops.push(if op == "NOT" { Op::Not } else { Op::Neg });
            }
            Expr::Binary { op, left, right, .. } => {
                self.expr(left);
                if op.eq_ignore_ascii_case("IS") {
                    if let Expr::Variable { name, .. } = &**right {
                        let u = name.to_ascii_uppercase();
                        if u == "ASSIGNED" || u == "INITIAL" || u == "BOUND" {
                            self.ops.push(Op::Push { value: serde_json::json!(u) });
                        } else {
                            self.expr(right);
                        }
                    } else {
                        self.expr(right);
                    }
                } else {
                    self.expr(right);
                }
                self.ops.push(Op::Binary { op: op.clone() });
            }
            Expr::Call { name, args, .. } => {
                for a in args {
                    self.expr(a);
                }
                self.ops.push(Op::Call {
                    name: name.clone(),
                    argc: args.len(),
                });
            }
            Expr::Constructor { assignments, .. } => {
                let fields: Vec<String> = assignments.iter().map(|a| a.name.clone()).collect();
                for a in assignments {
                    self.expr(&a.value);
                }
                self.ops.push(Op::Constructor { fields });
            }
            Expr::ValueConstructor { components, .. } => {
                use crate::compiler::ast::ValueComponent;
                let has_rows = components.iter().any(|c| matches!(c, ValueComponent::Row(_)));
                if has_rows {
                    let mut row_count = 0;
                    for comp in components {
                        if let ValueComponent::Row(row_comps) = comp {
                            let mut fields = Vec::new();
                            for rc in row_comps {
                                match rc {
                                    ValueComponent::Field { name, value } => {
                                        fields.push(name.clone());
                                        self.expr(value);
                                    }
                                    ValueComponent::Expression(expr) => {
                                        fields.push("VAL".to_string());
                                        self.expr(expr);
                                    }
                                    _ => {}
                                }
                            }
                            self.ops.push(Op::Constructor { fields });
                            row_count += 1;
                        }
                    }
                    self.ops.push(Op::TableConstructor { count: row_count });
                } else {
                    let mut fields = Vec::new();
                    for comp in components {
                        match comp {
                            ValueComponent::Field { name, value } => {
                                fields.push(name.clone());
                                self.expr(value);
                            }
                            ValueComponent::Expression(expr) => {
                                fields.push("VAL".to_string());
                                self.expr(expr);
                            }
                            _ => {}
                        }
                    }
                    self.ops.push(Op::Constructor { fields });
                }
            }
            Expr::NewObject { class_name, args, named, .. } => {
                for a in args {
                    self.expr(a);
                }
                for a in named.values() {
                    self.expr(a);
                }
                self.ops.push(Op::NewObject {
                    class_name: class_name.clone(),
                    positional_count: args.len(),
                    named: named.keys().cloned().collect(),
                });
            }
            Expr::MethodCallExpr {
                receiver,
                method,
                args,
                named,
                output_named,
                changing_named,
                ..
            } => {
                self.expr(receiver);
                for a in args {
                    self.expr(a);
                }
                for a in named.values() {
                    self.expr(a);
                }
                if let Some(cn) = changing_named {
                    for a in cn.values() {
                        self.expr(a);
                    }
                }
                let mut bindings = Vec::new();
                if let Some(on) = output_named {
                    for (formal, target_expr) in on {
                        if let Expr::Variable { name, .. } = target_expr {
                            bindings.push(OutputBinding {
                                formal: formal.clone(),
                                target: name.clone(),
                                changing: false,
                            });
                        }
                    }
                }
                if let Some(cn) = changing_named {
                    for (formal, target_expr) in cn {
                        if let Expr::Variable { name, .. } = target_expr {
                            bindings.push(OutputBinding {
                                formal: formal.clone(),
                                target: name.clone(),
                                changing: true,
                            });
                        }
                    }
                }
                let mut named_keys: Vec<String> = named.keys().cloned().collect();
                if let Some(cn) = changing_named {
                    named_keys.extend(cn.keys().cloned());
                }
                self.ops.push(Op::CallMethodExpr {
                    method: method.clone(),
                    positional_count: args.len(),
                    named: named_keys,
                    output_bindings: bindings,
                });
            }
            Expr::Switch { .. } => {
                self.lower_switch(e);
            }
        }
    }

    fn infer_expr_type(&self, e: &Expr) -> TypeRef {
        match e {
            Expr::Literal { type_ref, .. } => type_ref.clone(),
            Expr::NewObject { class_name, .. } => TypeRef::Object {
                class_name: class_name.clone(),
            },
            Expr::Binary { op, left, right, .. } => {
                if matches!(op.as_str(), "=" | "==" | "<>" | "<" | "<=" | ">" | ">=" | "AND" | "OR") {
                    return TypeRef::scalar("abap_bool");
                }
                if op == "&&" || op == "||" {
                    return TypeRef::scalar("string");
                }
                let l = self.infer_expr_type(left);
                if matches!(l, TypeRef::Scalar { .. }) {
                    l
                } else {
                    self.infer_expr_type(right)
                }
            }
            Expr::Variable { name, .. } => {
                let key = name.to_ascii_uppercase();
                if let Some(v) = self.vars.iter().find(|x| x.name.eq_ignore_ascii_case(&key)) {
                    v.type_ref.clone()
                } else {
                    TypeRef::any()
                }
            }
            Expr::Switch { result_type, branches, else_result, .. } => {
                if let Some(rt) = result_type {
                    return self.resolve_type(rt);
                }
                if let Some(first) = branches.first() {
                    self.infer_expr_type(&first.result)
                } else if let Some(el) = else_result {
                    self.infer_expr_type(el)
                } else {
                    TypeRef::any()
                }
            }
            Expr::Call { name, .. } => {
                let u = name.to_ascii_uppercase();
                match u.as_str() {
                    "STRLEN" | "XSTRLEN" | "LINES" | "ABS" | "FLOOR" | "CEIL" | "ROUND" => TypeRef::scalar("i"),
                    "TO_UPPER" | "TO_LOWER" => TypeRef::scalar("string"),
                    _ => TypeRef::any(),
                }
            }
            _ => TypeRef::any(),
        }
    }

    fn resolve_type(&self, t: &TypeRef) -> TypeRef {
        match t {
            TypeRef::Named { name } => {
                let key = name.to_ascii_uppercase();
                if let Some(found) = self.types.get(&key) {
                    return found.clone();
                }
                if let Some(prog) = &self.program {
                    if prog.classes.iter().any(|c| c.name.eq_ignore_ascii_case(name)) {
                        return TypeRef::Object {
                            class_name: name.clone(),
                        };
                    }
                }
                t.clone()
            }
            TypeRef::Table { line, table_kind, unique } => TypeRef::Table {
                line: Box::new(self.resolve_type(line)),
                table_kind: table_kind.clone(),
                unique: *unique,
            },
            TypeRef::Struct { name, fields } => {
                let mut resolved = std::collections::BTreeMap::new();
                for (k, v) in fields {
                    resolved.insert(k.clone(), self.resolve_type(v));
                }
                TypeRef::Struct {
                    name: name.clone(),
                    fields: resolved,
                }
            }
            _ => t.clone(),
        }
    }

    fn patch(&mut self, i: usize, target: i64) {
        if let Some(op) = self.ops.get_mut(i) {
            match op {
                Op::Jump { target: t } | Op::JumpIfFalse { target: t } => {
                    *t = target;
                }
                Op::PushHandler { target: t, .. } => {
                    *t = target;
                }
                _ => {}
            }
        }
    }

    fn patch_exit_placeholders(&mut self, target: i64) {
        let len = self.ops.len();
        for i in (0..len).rev() {
            if (i as i64) < target - 10000 {
                break;
            }
            if let Some(op) = self.ops.get_mut(i) {
                match op {
                    Op::Jump { target: t } if *t < 0 => {
                        *t = target;
                    }
                    Op::JumpIfFalse { target: t } if *t < 0 => {
                        *t = target;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn initial_value_for_type(type_ref: &TypeRef) -> Value {
    match type_ref {
        TypeRef::Table { .. } => Value::Array(Vec::new()),
        TypeRef::Struct { .. } => Value::Object(serde_json::Map::new()),
        TypeRef::Object { .. } => Value::Null,
        TypeRef::Scalar { name, .. } => {
            if matches!(
                name.as_str(),
                "string" | "xstring" | "c" | "n" | "d" | "t"
            ) {
                Value::String(String::new())
            } else if name == "abap_bool" {
                Value::Bool(false)
            } else {
                serde_json::json!(0)
            }
        }
        _ => Value::Null,
    }
}

#[derive(Debug, Clone)]
pub struct RequestedEvent {
    pub event: String,
    pub target: Option<String>,
}
