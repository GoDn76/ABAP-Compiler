use super::diagnostics::Position;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeBase {
    pub start: Position,
    pub end: Position,
}

impl NodeBase {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

pub type ScalarType = String; // 'i'|'int8'|'decfloat16'|'decfloat34'|'string'|'xstring'|'c'|'n'|'d'|'t'|'abap_bool'|'any'

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TypeRef {
    #[serde(rename = "scalar")]
    Scalar {
        name: ScalarType,
        #[serde(skip_serializing_if = "Option::is_none")]
        length: Option<usize>,
    },
    #[serde(rename = "table")]
    Table {
        line: Box<TypeRef>,
        #[serde(rename = "tableKind")]
        table_kind: String, // 'standard'|'sorted'|'hashed'
        #[serde(skip_serializing_if = "Option::is_none")]
        unique: Option<bool>,
    },
    #[serde(rename = "struct")]
    Struct {
        name: String,
        fields: BTreeMap<String, TypeRef>,
    },
    #[serde(rename = "object")]
    Object {
        #[serde(rename = "className")]
        class_name: String,
    },
    #[serde(rename = "named")]
    Named {
        name: String,
    },
}

impl TypeRef {
    pub fn scalar(name: &str) -> Self {
        TypeRef::Scalar {
            name: name.to_string(),
            length: None,
        }
    }
    pub fn any() -> Self {
        Self::scalar("any")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub start: Position,
    pub end: Position,
    #[serde(rename = "reportName", skip_serializing_if = "Option::is_none")]
    pub report_name: Option<String>,
    pub statements: Vec<Statement>,
    pub types: Vec<TypeDecl>,
    pub classes: Vec<ClassDecl>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDecl {
    pub start: Position,
    pub end: Position,
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassAttributeDecl {
    pub start: Position,
    pub end: Position,
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    #[serde(rename = "isStatic")]
    pub is_static: bool,
    pub visibility: Visibility,
    pub constant: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    pub direction: String, // 'importing'|'exporting'|'changing'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Expr>,
    #[serde(rename = "passByValue", skip_serializing_if = "Option::is_none")]
    pub pass_by_value: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturningParam {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodDecl {
    pub start: Position,
    pub end: Position,
    pub name: String,
    pub statements: Vec<Statement>,
    pub params: Vec<MethodParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returning: Option<ReturningParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implemented: Option<bool>,
    #[serde(rename = "isStatic", skip_serializing_if = "Option::is_none")]
    pub is_static: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(rename = "isAbstract", skip_serializing_if = "Option::is_none")]
    pub is_abstract: Option<bool>,
    #[serde(rename = "isFinal", skip_serializing_if = "Option::is_none")]
    pub is_final: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redefinition: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDecl {
    pub start: Position,
    pub end: Position,
    pub name: String,
    pub methods: Vec<MethodDecl>,
    pub attributes: Vec<ClassAttributeDecl>,
    pub implementation: bool,
    pub interfaces: Vec<String>,
    pub types: Vec<TypeDecl>,
    #[serde(rename = "superClass", skip_serializing_if = "Option::is_none")]
    pub super_class: Option<String>,
    #[serde(rename = "isFinal", skip_serializing_if = "Option::is_none")]
    pub is_final: Option<bool>,
    #[serde(rename = "isAbstract", skip_serializing_if = "Option::is_none")]
    pub is_abstract: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Statement {
    DataDecl {
        start: Position,
        end: Position,
        name: String,
        #[serde(rename = "type")]
        type_ref: TypeRef,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inline: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameter: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        obligatory: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        checkbox: Option<bool>,
        #[serde(rename = "selectionOptions", skip_serializing_if = "Option::is_none")]
        selection_options: Option<bool>,
        #[serde(rename = "noIntervals", skip_serializing_if = "Option::is_none")]
        no_intervals: Option<bool>,
        #[serde(rename = "noExtension", skip_serializing_if = "Option::is_none")]
        no_extension: Option<bool>,
        #[serde(rename = "forField", skip_serializing_if = "Option::is_none")]
        for_field: Option<String>,
        #[serde(rename = "radiobuttonGroup", skip_serializing_if = "Option::is_none")]
        radiobutton_group: Option<String>,
        #[serde(rename = "lowerCase", skip_serializing_if = "Option::is_none")]
        lower_case: Option<bool>,
        #[serde(rename = "noDisplay", skip_serializing_if = "Option::is_none")]
        no_display: Option<bool>,
        #[serde(rename = "valueHelp", skip_serializing_if = "Option::is_none")]
        value_help: Option<String>,
    },
    InlineDataAssign {
        start: Position,
        end: Position,
        name: String,
        value: Expr,
    },
    ConstantDecl {
        start: Position,
        end: Position,
        name: String,
        #[serde(rename = "type")]
        type_ref: TypeRef,
        value: Expr,
    },
    Assignment {
        start: Position,
        end: Position,
        target: Expr,
        value: Expr,
    },
    WriteStmt {
        start: Position,
        end: Position,
        values: Vec<Expr>,
        groups: Vec<Vec<Expr>>,
    },
    IfStmt {
        start: Position,
        end: Position,
        branches: Vec<IfBranch>,
        #[serde(rename = "elseBranch")]
        else_branch: Vec<Statement>,
    },
    WhileStmt {
        start: Position,
        end: Position,
        condition: Expr,
        body: Vec<Statement>,
    },
    DoStmt {
        start: Position,
        end: Position,
        #[serde(skip_serializing_if = "Option::is_none")]
        count: Option<Expr>,
        body: Vec<Statement>,
    },
    CaseStmt {
        start: Position,
        end: Position,
        expression: Expr,
        whens: Vec<CaseWhen>,
        others: Vec<Statement>,
    },
    LoopStmt {
        start: Position,
        end: Position,
        source: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inline: Option<bool>,
        body: Vec<Statement>,
    },
    AppendStmt {
        start: Position,
        end: Position,
        value: Expr,
        table: Expr,
    },
    InsertStmt {
        start: Position,
        end: Position,
        value: Expr,
        table: Expr,
    },
    ReadTableStmt {
        start: Position,
        end: Position,
        table: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<BTreeMap<String, Expr>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inline: Option<bool>,
    },
    DeleteTableStmt {
        start: Position,
        end: Position,
        table: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<BTreeMap<String, Expr>>,
    },
    SortStmt {
        start: Position,
        end: Position,
        table: Expr,
        keys: Vec<SortKey>,
    },
    DescribeTableStmt {
        start: Position,
        end: Position,
        table: Expr,
        target: String,
    },
    ClearStmt {
        start: Position,
        end: Position,
        target: Expr,
    },
    TranslateStmt {
        start: Position,
        end: Position,
        target: Expr,
        mode: String, // 'upper'|'lower'|'using'
        #[serde(skip_serializing_if = "Option::is_none")]
        mask: Option<Expr>,
    },
    ConcatenateStmt {
        start: Position,
        end: Position,
        sources: Vec<Expr>,
        target: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        separator: Option<Expr>,
        #[serde(rename = "respectingBlanks")]
        respecting_blanks: bool,
    },
    SplitStmt {
        start: Position,
        end: Position,
        source: Expr,
        separator: Expr,
        targets: Vec<Expr>,
        #[serde(rename = "tableTarget", skip_serializing_if = "Option::is_none")]
        table_target: Option<Expr>,
    },
    CondenseStmt {
        start: Position,
        end: Position,
        target: Expr,
        #[serde(rename = "noGaps")]
        no_gaps: bool,
    },
    MessageStmt {
        start: Position,
        end: Position,
        text: Expr,
        #[serde(rename = "type")]
        message_type: Expr,
    },
    EventBlock {
        start: Position,
        end: Position,
        event: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        statements: Vec<Statement>,
    },
    FlowStmt {
        start: Position,
        end: Position,
        op: String, // 'continue'|'exit'|'return'|'check'
        #[serde(skip_serializing_if = "Option::is_none")]
        condition: Option<Expr>,
    },
    AssertStmt {
        start: Position,
        end: Position,
        condition: Expr,
    },
    CallFunction {
        start: Position,
        end: Position,
        name: String,
        parameters: BTreeMap<String, Expr>,
    },
    MethodCallStmt {
        start: Position,
        end: Position,
        receiver: String,
        method: String,
        args: Vec<Expr>,
        named: BTreeMap<String, Expr>,
        #[serde(rename = "outputNamed", skip_serializing_if = "Option::is_none")]
        output_named: Option<BTreeMap<String, Expr>>,
        #[serde(rename = "changingNamed", skip_serializing_if = "Option::is_none")]
        changing_named: Option<BTreeMap<String, Expr>>,
        #[serde(rename = "staticCall", skip_serializing_if = "Option::is_none")]
        static_call: Option<bool>,
    },
    EmlReadStmt {
        start: Position,
        end: Position,
        #[serde(rename = "businessObject")]
        business_object: String,
        entity: String,
        filters: Vec<EmlFilterRow>,
        #[serde(rename = "resultTarget")]
        result_target: String,
        #[serde(rename = "failedTarget", skip_serializing_if = "Option::is_none")]
        failed_target: Option<String>,
        #[serde(rename = "reportedTarget", skip_serializing_if = "Option::is_none")]
        reported_target: Option<String>,
    },
    EmlModifyStmt {
        start: Position,
        end: Position,
        #[serde(rename = "businessObject")]
        business_object: String,
        entity: String,
        operation: String, // 'CREATE'|'UPDATE'|'DELETE'
        rows: Vec<EmlModifyRow>,
        #[serde(rename = "mappedTarget", skip_serializing_if = "Option::is_none")]
        mapped_target: Option<String>,
        #[serde(rename = "failedTarget", skip_serializing_if = "Option::is_none")]
        failed_target: Option<String>,
        #[serde(rename = "reportedTarget", skip_serializing_if = "Option::is_none")]
        reported_target: Option<String>,
    },
    EmlTransactionStmt {
        start: Position,
        end: Position,
        operation: String, // 'COMMIT'|'ROLLBACK'
    },
    SelectStmt {
        start: Position,
        end: Position,
        fields: Vec<String>,
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        single: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        where_clause: Option<Expr>,
    },
    TryStmt {
        start: Position,
        end: Position,
        body: Vec<Statement>,
        catches: Vec<TryCatch>,
    },
    EmptyStmt {
        start: Position,
        end: Position,
    },
    ClassDecl(ClassDecl),
    TypeDecl(TypeDecl),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfBranch {
    pub condition: Expr,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseWhen {
    pub values: Vec<Expr>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SortKey {
    pub name: String,
    pub descending: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmlFieldBinding {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmlFilterRow {
    pub fields: Vec<EmlFieldBinding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmlModifyRow {
    pub fields: Vec<EmlFieldBinding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TryCatch {
    #[serde(rename = "type")]
    pub catch_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Expr {
    Literal {
        start: Position,
        end: Position,
        value: serde_json::Value,
        #[serde(rename = "type")]
        type_ref: TypeRef,
    },
    Variable {
        start: Position,
        end: Position,
        name: String,
    },
    Field {
        start: Position,
        end: Position,
        base: Box<Expr>,
        field: String,
        #[serde(rename = "staticSelector", skip_serializing_if = "Option::is_none")]
        static_selector: Option<bool>,
    },
    Unary {
        start: Position,
        end: Position,
        op: String,
        operand: Box<Expr>,
    },
    Binary {
        start: Position,
        end: Position,
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        start: Position,
        end: Position,
        name: String,
        args: Vec<Expr>,
    },
    Index {
        start: Position,
        end: Position,
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Constructor {
        start: Position,
        end: Position,
        assignments: Vec<ConstructorAssignment>,
    },
    NewObject {
        start: Position,
        end: Position,
        #[serde(rename = "className")]
        class_name: String,
        args: Vec<Expr>,
        named: BTreeMap<String, Expr>,
    },
    MethodCallExpr {
        start: Position,
        end: Position,
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        named: BTreeMap<String, Expr>,
        #[serde(rename = "outputNamed", skip_serializing_if = "Option::is_none")]
        output_named: Option<BTreeMap<String, Expr>>,
        #[serde(rename = "changingNamed", skip_serializing_if = "Option::is_none")]
        changing_named: Option<BTreeMap<String, Expr>>,
        #[serde(rename = "staticCall", skip_serializing_if = "Option::is_none")]
        static_call: Option<bool>,
    },
    Switch {
        start: Position,
        end: Position,
        #[serde(rename = "resultType", skip_serializing_if = "Option::is_none")]
        result_type: Option<TypeRef>,
        #[serde(rename = "inferType")]
        infer_type: bool,
        operand: Box<Expr>,
        branches: Vec<SwitchBranch>,
        #[serde(rename = "elseResult", skip_serializing_if = "Option::is_none")]
        else_result: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructorAssignment {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchBranch {
    pub values: Vec<Expr>,
    pub result: Expr,
}
