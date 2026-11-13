use super::ast::*;
use super::class_model::{build_class_models, class_lineage, inherited_attributes, resolve_method, ClassModel};
use super::diagnostics::{DiagnosticBag, Position};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub type_ref: TypeRef,
    pub constant: bool,
    pub attribute: bool,
    pub is_static: bool,
    pub visibility: Option<Visibility>,
    pub owner_class: Option<String>,
}

pub struct SemanticAnalyzer {
    scopes: Vec<HashMap<String, SymbolInfo>>,
    classes: HashMap<String, ClassModel>,
    types: HashMap<String, TypeRef>,
    current_class: Option<ClassModel>,
    current_method: Option<MethodDecl>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            classes: HashMap::new(),
            types: HashMap::new(),
            current_class: None,
            current_method: None,
        }
    }

    pub fn analyze(
        mut self,
        program: &Program,
        external_types: HashMap<String, TypeRef>,
    ) -> DiagnosticBag {
        let mut diagnostics = DiagnosticBag::new();
        self.classes = build_class_models(program);
        self.validate_class_hierarchy(&mut diagnostics);

        self.types = external_types
            .into_iter()
            .map(|(k, v)| (k.to_ascii_uppercase(), v))
            .collect();
        for t in &program.types {
            self.types.insert(t.name.to_ascii_uppercase(), t.type_ref.clone());
        }

        self.scopes = vec![HashMap::new()];

        let mut sy_fields = std::collections::BTreeMap::new();
        sy_fields.insert("SUBRC".to_string(), TypeRef::scalar("i"));
        sy_fields.insert("TABIX".to_string(), TypeRef::scalar("i"));
        sy_fields.insert("DBCNT".to_string(), TypeRef::scalar("i"));
        let sy_type = TypeRef::Struct {
            name: "SY".to_string(),
            fields: sy_fields,
        };
        self.declare("SY", sy_type, false, None, &mut diagnostics);

        for statement in &program.statements {
            self.check_statement(statement, &mut diagnostics);
        }

        let class_keys: Vec<String> = self.classes.keys().cloned().collect();
        for class_key in class_keys {
            let model = self.classes.get(&class_key).unwrap().clone();
            let mut class_types = self.types.clone();
            for ty in &model.types {
                class_types.insert(ty.name.to_ascii_uppercase(), ty.type_ref.clone());
            }

            let method_keys: Vec<String> = model.methods.keys().cloned().collect();
            for method_key in method_keys {
                let method = model.methods.get(&method_key).unwrap().clone();
                if method.statements.is_empty() {
                    continue;
                }

                self.types = class_types.clone();
                self.current_class = Some(model.clone());
                self.current_method = Some(method.clone());
                self.scopes.push(HashMap::new());

                if !method.is_static.unwrap_or(false) {
                    self.declare(
                        "ME",
                        TypeRef::Object {
                            class_name: model.name.clone(),
                        },
                        false,
                        None,
                        &mut diagnostics,
                    );
                }

                for attribute in inherited_attributes(&self.classes, &model.name) {
                    let owner = class_lineage(&self.classes, &model.name)
                        .into_iter()
                        .find(|x| x.attributes.contains_key(&attribute.name.to_ascii_uppercase()));
                    let is_own = owner.as_ref().map(|o| o.name.eq_ignore_ascii_case(&model.name)).unwrap_or(false);

                    if !is_own && attribute.visibility == Visibility::Private {
                        continue;
                    }
                    if method.is_static.unwrap_or(false) && !attribute.is_static {
                        continue;
                    }

                    let resolved_ty = self.resolve_type(&attribute.type_ref);
                    self.declare(&attribute.name, resolved_ty, attribute.constant, None, &mut diagnostics);
                    if let Some(symbol) = self.current_scope_mut().get_mut(&attribute.name.to_ascii_uppercase()) {
                        symbol.attribute = true;
                        symbol.is_static = attribute.is_static;
                        symbol.visibility = Some(attribute.visibility);
                        symbol.owner_class = owner.map(|o| o.name);
                    }
                }

                for parameter in &method.params {
                    let resolved = self.resolve_type(&parameter.type_ref);
                    self.declare(&parameter.name, resolved, false, None, &mut diagnostics);
                }
                if let Some(ret) = &method.returning {
                    let resolved = self.resolve_type(&ret.type_ref);
                    self.declare(&ret.name, resolved, false, None, &mut diagnostics);
                }

                for statement in &method.statements {
                    self.check_statement(statement, &mut diagnostics);
                }

                self.scopes.pop();
            }
        }

        self.current_class = None;
        self.current_method = None;
        diagnostics
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, SymbolInfo> {
        let len = self.scopes.len();
        &mut self.scopes[len - 1]
    }

    fn lookup(&self, name: &str) -> Option<SymbolInfo> {
        let key = name.to_ascii_uppercase();
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(&key) {
                return Some(info.clone());
            }
        }
        None
    }

    fn declare(
        &mut self,
        name: &str,
        type_ref: TypeRef,
        constant: bool,
        node_pos: Option<(Position, Position)>,
        d: &mut DiagnosticBag,
    ) {
        let key = name.to_ascii_uppercase();
        if self.current_scope_mut().contains_key(&key) {
            if let Some((start, end)) = node_pos {
                d.error("SEM001", format!("Duplicate declaration '{}'.", name), start, end);
            }
            return;
        }
        let resolved = self.resolve_type(&type_ref);
        self.current_scope_mut().insert(
            key,
            SymbolInfo {
                type_ref: resolved,
                constant,
                attribute: false,
                is_static: false,
                visibility: None,
                owner_class: None,
            },
        );
    }

    fn resolve_type(&self, type_ref: &TypeRef) -> TypeRef {
        match type_ref {
            TypeRef::Named { name } => {
                let key = name.to_ascii_uppercase();
                if let Some(found) = self.types.get(&key) {
                    found.clone()
                } else if self.classes.contains_key(&key) {
                    TypeRef::Object {
                        class_name: name.clone(),
                    }
                } else {
                    type_ref.clone()
                }
            }
            TypeRef::Table {
                line,
                table_kind,
                unique,
            } => TypeRef::Table {
                line: Box::new(self.resolve_type(line)),
                table_kind: table_kind.clone(),
                unique: *unique,
            },
            TypeRef::Struct { name, fields } => {
                let mut resolved_fields = std::collections::BTreeMap::new();
                for (k, v) in fields {
                    resolved_fields.insert(k.to_ascii_uppercase(), self.resolve_type(v));
                }
                TypeRef::Struct {
                    name: name.clone(),
                    fields: resolved_fields,
                }
            }
            _ => type_ref.clone(),
        }
    }

    fn validate_class_hierarchy(&self, d: &mut DiagnosticBag) {
        for model in self.classes.values() {
            if let Some(super_name) = &model.super_class {
                let parent = self.classes.get(&super_name.to_ascii_uppercase());
                if parent.is_none() {
                    d.error(
                        "SEM450",
                        format!("Superclass '{}' of '{}' is not declared in this program.", super_name, model.name),
                        model.start,
                        model.end,
                    );
                } else if parent.unwrap().is_final {
                    d.error(
                        "SEM451",
                        format!("Class '{}' cannot inherit from FINAL class '{}'.", model.name, parent.unwrap().name),
                        model.start,
                        model.end,
                    );
                }
            }

            let lineage = class_lineage(&self.classes, &model.name);
            if lineage.len() > self.classes.len() + 1 {
                d.error(
                    "SEM452",
                    format!("Inheritance cycle detected for class '{}'.", model.name),
                    model.start,
                    model.end,
                );
            }

            for (method_name, method) in &model.methods {
                let inherited = if let Some(super_name) = &model.super_class {
                    resolve_method(&self.classes, super_name, method_name)
                } else {
                    None
                };

                if method.redefinition.unwrap_or(false) && inherited.is_none() {
                    d.error(
                        "SEM453",
                        format!("Method '{}' is marked REDEFINITION but no inherited method exists.", method.name),
                        method.start,
                        method.end,
                    );
                }
                if let Some((_, inherited_method)) = &inherited {
                    if method.redefinition.unwrap_or(false) {
                        if inherited_method.is_final.unwrap_or(false) {
                            d.error(
                                "SEM454",
                                format!("Final method '{}' cannot be redefined.", inherited_method.name),
                                method.start,
                                method.end,
                            );
                        }
                        if inherited_method.is_static != method.is_static {
                            d.error(
                                "SEM455",
                                format!("Redefinition of '{}' must preserve static/instance method kind.", method.name),
                                method.start,
                                method.end,
                            );
                        }
                    }
                }
            }
        }
    }

    fn check_statement(&mut self, statement: &Statement, d: &mut DiagnosticBag) {
        match statement {
            Statement::DataDecl {
                name,
                type_ref,
                value,
                start,
                end,
                ..
            } => {
                self.declare(name, type_ref.clone(), false, Some((*start, *end)), d);
                if let Some(v) = value {
                    self.check_expr(v, d);
                }
            }
            Statement::InlineDataAssign { name, value, start, end } => {
                let inferred = self.infer_expr_type(value);
                self.declare(name, inferred, false, Some((*start, *end)), d);
                self.check_expr(value, d);
            }
            Statement::ConstantDecl { name, type_ref, value, start, end } => {
                self.declare(name, type_ref.clone(), true, Some((*start, *end)), d);
                self.check_expr(value, d);
            }
            Statement::Assignment { target, value, start, end } => {
                self.check_expr(target, d);
                self.check_expr(value, d);
                if let Expr::Variable { name, .. } = target {
                    if let Some(info) = self.lookup(name) {
                        if info.constant {
                            d.error("SEM002", format!("Cannot assign to constant '{}'.", name), *start, *end);
                        }
                    }
                }
            }
            Statement::Assign { source, .. } => {
                self.check_expr(source, d);
            }
            Statement::WriteStmt { values, .. } => {
                for v in values {
                    self.check_expr(v, d);
                }
            }
            Statement::IfStmt { branches, else_branch, .. } => {
                for b in branches {
                    self.check_expr(&b.condition, d);
                    for s in &b.statements {
                        self.check_statement(s, d);
                    }
                }
                for s in else_branch {
                    self.check_statement(s, d);
                }
            }
            Statement::WhileStmt { condition, body, .. } => {
                self.check_expr(condition, d);
                for s in body {
                    self.check_statement(s, d);
                }
            }
            Statement::DoStmt { count, body, .. } => {
                if let Some(c) = count {
                    self.check_expr(c, d);
                }
                for s in body {
                    self.check_statement(s, d);
                }
            }
            Statement::CaseStmt { expression, whens, others, .. } => {
                self.check_expr(expression, d);
                for w in whens {
                    for v in &w.values {
                        self.check_expr(v, d);
                    }
                    for s in &w.statements {
                        self.check_statement(s, d);
                    }
                }
                for s in others {
                    self.check_statement(s, d);
                }
            }
            Statement::LoopStmt { source, target, inline, body, .. } => {
                self.check_expr(source, d);
                if let Some(t) = target {
                    if inline.unwrap_or(false) {
                        self.declare(t, TypeRef::any(), false, None, d);
                    }
                }
                for s in body {
                    self.check_statement(s, d);
                }
            }
            Statement::AppendStmt { value, table, .. } | Statement::InsertStmt { value, table, .. } => {
                self.check_expr(value, d);
                self.check_expr(table, d);
            }
            Statement::ReadTableStmt { table, target, inline, index, key, .. } => {
                self.check_expr(table, d);
                if let Some(t) = target {
                    if inline.unwrap_or(false) {
                        self.declare(t, TypeRef::any(), false, None, d);
                    }
                }
                if let Some(idx) = index {
                    self.check_expr(idx, d);
                }
                if let Some(kmap) = key {
                    for v in kmap.values() {
                        self.check_expr(v, d);
                    }
                }
            }
            Statement::DeleteTableStmt { table, index, key, .. } => {
                self.check_expr(table, d);
                if let Some(idx) = index {
                    self.check_expr(idx, d);
                }
                if let Some(kmap) = key {
                    for v in kmap.values() {
                        self.check_expr(v, d);
                    }
                }
            }
            Statement::SortStmt { table, .. } | Statement::DescribeTableStmt { table, .. } => {
                self.check_expr(table, d);
            }
            Statement::ClearStmt { target, .. } => {
                self.check_expr(target, d);
            }
            Statement::TranslateStmt { target, mask, .. } => {
                self.check_expr(target, d);
                if let Some(m) = mask {
                    self.check_expr(m, d);
                }
            }
            Statement::ConcatenateStmt { sources, target, separator, .. } => {
                for s in sources {
                    self.check_expr(s, d);
                }
                self.check_expr(target, d);
                if let Some(sep) = separator {
                    self.check_expr(sep, d);
                }
            }
            Statement::SplitStmt { source, separator, targets, table_target, .. } => {
                self.check_expr(source, d);
                self.check_expr(separator, d);
                for t in targets {
                    self.check_expr(t, d);
                }
                if let Some(tt) = table_target {
                    self.check_expr(tt, d);
                }
            }
            Statement::CondenseStmt { target, .. } => {
                self.check_expr(target, d);
            }
            Statement::MessageStmt { text, message_type, .. } => {
                self.check_expr(text, d);
                self.check_expr(message_type, d);
            }
            Statement::EventBlock { statements, .. } => {
                for s in statements {
                    self.check_statement(s, d);
                }
            }
            Statement::FlowStmt { condition, .. } => {
                if let Some(c) = condition {
                    self.check_expr(c, d);
                }
            }
            Statement::AssertStmt { condition, .. } => {
                self.check_expr(condition, d);
            }
            Statement::CallFunction { parameters, .. } => {
                for v in parameters.values() {
                    self.check_expr(v, d);
                }
            }
            Statement::MethodCallStmt { receiver, method, args, named, .. }
            | Statement::MethodCall { receiver, method, args, named, .. } => {
                if !receiver.eq_ignore_ascii_case("OUT") {
                    if let Some(info) = self.lookup(receiver) {
                        if let TypeRef::Object { class_name } = &info.type_ref {
                            if resolve_method(&self.classes, class_name, method).is_none() {
                                // method unresolved check
                            }
                        }
                    }
                }
                for a in args {
                    self.check_expr(a, d);
                }
                for a in named.values() {
                    self.check_expr(a, d);
                }
            }
            Statement::RaiseStmt { .. } => {}
            Statement::EmlReadStmt { filters, .. } => {
                for r in filters {
                    for f in &r.fields {
                        self.check_expr(&f.value, d);
                    }
                }
            }
            Statement::EmlModifyStmt { rows, .. } => {
                for r in rows {
                    for f in &r.fields {
                        self.check_expr(&f.value, d);
                    }
                }
            }
            Statement::SelectStmt { target, .. } => {
                if let Some(t) = target {
                    self.declare(t, TypeRef::any(), false, None, d);
                }
            }
            Statement::TryStmt { body, catches, .. } => {
                for s in body {
                    self.check_statement(s, d);
                }
                for c in catches {
                    for s in &c.body {
                        self.check_statement(s, d);
                    }
                }
            }
            _ => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr, d: &mut DiagnosticBag) {
        match expr {
            Expr::Variable { name, start, end } => {
                let upper = name.to_ascii_uppercase();
                if upper == "ASSIGNED" || upper == "INITIAL" || upper == "BOUND" || upper == "REQUESTED" {
                    return;
                }
                let base = upper.split('-').next().unwrap();
                if !self.classes.contains_key(&upper)
                    && self.lookup(name).is_none()
                    && self.lookup(&upper).is_none()
                    && self.lookup(base).is_none()
                {
                    d.error("SEM010", format!("Undeclared variable '{}'.", name), *start, *end);
                }
            }
            Expr::Field { base, .. } => {
                self.check_expr(base, d);
            }
            Expr::Index { base, index, .. } => {
                self.check_expr(base, d);
                self.check_expr(index, d);
            }
            Expr::Unary { operand, .. } => {
                self.check_expr(operand, d);
            }
            Expr::Binary { left, right, .. } => {
                self.check_expr(left, d);
                self.check_expr(right, d);
            }
            Expr::Call { args, .. } => {
                for a in args {
                    self.check_expr(a, d);
                }
            }
            Expr::Constructor { assignments, .. } => {
                for a in assignments {
                    self.check_expr(&a.value, d);
                }
            }
            Expr::ValueConstructor { components, .. } => {
                fn check_vc(comp: &crate::compiler::ast::ValueComponent, sem: &mut SemanticAnalyzer, d: &mut DiagnosticBag) {
                    match comp {
                        crate::compiler::ast::ValueComponent::Field { value, .. } => sem.check_expr(value, d),
                        crate::compiler::ast::ValueComponent::Expression(value) => sem.check_expr(value, d),
                        crate::compiler::ast::ValueComponent::Row(rows) => {
                            for r in rows {
                                check_vc(r, sem, d);
                            }
                        }
                    }
                }
                for c in components {
                    check_vc(c, self, d);
                }
            }
            Expr::NewObject { args, named, .. } => {
                for a in args {
                    self.check_expr(a, d);
                }
                for a in named.values() {
                    self.check_expr(a, d);
                }
            }
            Expr::MethodCallExpr { receiver, args, named, .. } => {
                self.check_expr(receiver, d);
                for a in args {
                    self.check_expr(a, d);
                }
                for a in named.values() {
                    self.check_expr(a, d);
                }
            }
            Expr::Switch { operand, branches, else_result, .. } => {
                self.check_expr(operand, d);
                for b in branches {
                    for v in &b.values {
                        self.check_expr(v, d);
                    }
                    self.check_expr(&b.result, d);
                }
                if let Some(el) = else_result {
                    self.check_expr(el, d);
                }
            }
            _ => {}
        }
    }

    fn infer_expr_type(&self, expr: &Expr) -> TypeRef {
        match expr {
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
                if let Some(info) = self.lookup(name) {
                    info.type_ref
                } else {
                    TypeRef::any()
                }
            }
            Expr::Switch { result_type, branches, else_result, .. } => {
                if let Some(rt) = result_type {
                    return rt.clone();
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
}
