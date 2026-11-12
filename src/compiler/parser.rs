use super::ast::*;
use super::diagnostics::DiagnosticBag;
use super::tokens::{Token, TokenKind};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::LazyLock;

static PREC: LazyLock<HashMap<&'static str, i32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("OR", 1);
    m.insert("||", 1);
    m.insert("AND", 2);
    m.insert("&&", 2);
    m.insert("=", 3);
    m.insert("==", 3);
    m.insert("<>", 3);
    m.insert("<", 3);
    m.insert("<=", 3);
    m.insert(">", 3);
    m.insert(">=", 3);
    m.insert("IS", 3);
    m.insert("IN", 3);
    m.insert("+", 4);
    m.insert("-", 4);
    m.insert("MOD", 5);
    m.insert("DIV", 5);
    m.insert("*", 5);
    m.insert("/", 5);
    m
});

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
    report_name: Option<String>,
    types: Vec<TypeDecl>,
    classes: Vec<ClassDecl>,
    diagnostics: DiagnosticBag,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            i: 0,
            report_name: None,
            types: Vec::new>,
            classes: Vec::new(),
            diagnostics: DiagnosticBag::new(),
        }
    }

    pub fn parse(mut self) -> (Program, DiagnosticBag) {
        let first = self.current().clone();
        let mut statements = Vec::new();

        while !self.at(TokenKind::EOF) {
            let empty_set = HashSet::new();
            if let Some(ss) = self.parse_statement(&empty_set) {
                statements.extend(ss);
            } else if !self.at(TokenKind::EOF) {
                self.recover_statement();
            }
        }

        let end_pos = statements
            .last()
            .map(|s| match s {
                Statement::DataDecl { end, .. }
                | Statement::InlineDataAssign { end, .. }
                | Statement::ConstantDecl { end, .. }
                | Statement::Assignment { end, .. }
                | Statement::WriteStmt { end, .. }
                | Statement::IfStmt { end, .. }
                | Statement::WhileStmt { end, .. }
                | Statement::DoStmt { end, .. }
                | Statement::CaseStmt { end, .. }
                | Statement::LoopStmt { end, .. }
                | Statement::AppendStmt { end, .. }
                | Statement::InsertStmt { end, .. }
                | Statement::ReadTableStmt { end, .. }
                | Statement::DeleteTableStmt { end, .. }
                | Statement::SortStmt { end, .. }
                | Statement::DescribeTableStmt { end, .. }
                | Statement::ClearStmt { end, .. }
                | Statement::TranslateStmt { end, .. }
                | Statement::ConcatenateStmt { end, .. }
                | Statement::SplitStmt { end, .. }
                | Statement::CondenseStmt { end, .. }
                | Statement::MessageStmt { end, .. }
                | Statement::EventBlock { end, .. }
                | Statement::FlowStmt { end, .. }
                | Statement::AssertStmt { end, .. }
                | Statement::CallFunction { end, .. }
                | Statement::MethodCallStmt { end, .. }
                | Statement::EmlReadStmt { end, .. }
                | Statement::EmlModifyStmt { end, .. }
                | Statement::EmlTransactionStmt { end, .. }
                | Statement::SelectStmt { end, .. }
                | Statement::TryStmt { end, .. }
                | Statement::EmptyStmt { end, .. } => *end,
                Statement::ClassDecl(c) => c.end,
                Statement::TypeDecl(t) => t.end,
            })
            .unwrap_or(first.end);

        let program = Program {
            start: first.start,
            end: end_pos,
            report_name: self.report_name,
            statements,
            types: self.types,
            classes: self.classes,
        };

        (program, self.diagnostics)
    }

    fn parse_statement(&mut self, end_words: &HashSet<String>) -> Option<Vec<Statement>> {
        if self.at(TokenKind::EOF) {
            return Some(Vec::new());
        }
        let t = self.current().clone();
        if t.kind == TokenKind::Dot {
            self.consume();
            return Some(vec![Statement::EmptyStmt {
                start: t.start,
                end: t.end,
            }]);
        }
        if self.is_word_in(end_words) {
            return Some(Vec::new());
        }

        if self.is_keyword("REPORT") {
            return self.parse_report().map(|s| vec![s]);
        }
        if self.is_keyword("PROGRAM") {
            return self.parse_simple_header().map(|s| vec![s]);
        }
        if self.is_keyword("DATA") && self.peek(1).kind == TokenKind::LParen {
            return self.parse_inline_data_assignment().map(|s| vec![s]);
        }
        if self.is_keyword("DATA") || self.is_keyword("STATICS") {
            return self.parse_data();
        }
        if self.is_keyword("PARAMETERS") {
            return self.parse_parameters();
        }
        if self.is_keyword("SELECT-OPTIONS") {
            return Some(self.parse_select_options());
        }
        if self.is_keyword("SELECTION-SCREEN") {
            return Some(vec![self.parse_selection_screen_decl()]);
        }
        if self.is_event_header_ahead() {
            return Some(vec![self.parse_event_block()]);
        }
        if self.is_keyword("TRANSLATE") {
            return Some(vec![self.parse_translate()]);
        }
        if self.is_keyword("CONCATENATE") {
            return Some(vec![self.parse_concatenate()]);
        }
        if self.is_keyword("SPLIT") {
            return Some(vec![self.parse_split()]);
        }
        if self.is_keyword("CONDENSE") {
            return Some(vec![self.parse_condense()]);
        }
        if self.is_keyword("MESSAGE") {
            return Some(vec![self.parse_message()]);
        }
        if self.is_keyword("CONSTANTS") {
            return self.parse_constants();
        }
        if self.is_keyword("TYPES") {
            return self.parse_types();
        }
        if self.is_keyword("IF") {
            return Some(vec![self.parse_if()]);
        }
        if self.is_keyword("WHILE") {
            return Some(vec![self.parse_while()]);
        }
        if self.is_keyword("DO") {
            return Some(vec![self.parse_do()]);
        }
        if self.is_keyword("CASE") {
            return Some(vec![self.parse_case()]);
        }
        if self.is_keyword("LOOP") {
            return Some(vec![self.parse_loop()]);
        }
        if self.is_keyword("APPEND") {
            return Some(vec![self.parse_append()]);
        }
        if self.is_keyword("INSERT") {
            return Some(vec![self.parse_insert()]);
        }
        if self.is_keyword("READ") && self.peek(1).upper == "ENTITIES" {
            return Some(vec![self.parse_eml_read()]);
        }
        if self.is_keyword("READ") {
            return Some(vec![self.parse_read_table()]);
        }
        if self.is_keyword("MODIFY") && self.peek(1).upper == "ENTITIES" {
            return Some(vec![self.parse_eml_modify()]);
        }
        if self.is_keyword("COMMIT") || self.is_keyword("ROLLBACK") {
            return Some(vec![self.parse_eml_transaction()]);
        }
        if self.is_keyword("DELETE") {
            return Some(vec![self.parse_delete_table()]);
        }
        if self.is_keyword("SORT") {
            return Some(vec![self.parse_sort()]);
        }
        if self.is_keyword("DESCRIBE") {
            return Some(vec![self.parse_describe()]);
        }
        if self.is_keyword("CLEAR") {
            return Some(vec![self.parse_clear()]);
        }
        if self.is_keyword("CHECK")
            || self.is_keyword("CONTINUE")
            || self.is_keyword("EXIT")
            || self.is_keyword("RETURN")
        {
            return Some(vec![self.parse_flow()]);
        }
        if self.is_keyword("ASSERT") {
            return Some(vec![self.parse_assert()]);
        }
        if self.is_keyword("WRITE") {
            return Some(vec![self.parse_write()]);
        }
        if self.is_keyword("CALL") {
            return Some(vec![self.parse_call_function()]);
        }
        if self.is_keyword("SELECT") {
            return Some(vec![self.parse_select()]);
        }
        if self.is_keyword("TRY") {
            return Some(vec![self.parse_try()]);
        }
        if self.is_keyword("CLASS") {
            return Some(vec![Statement::ClassDecl(self.parse_class())]);
        }
        if (self.at(TokenKind::Identifier) || self.at(TokenKind::Keyword))
            && self.peek(1).kind == TokenKind::Arrow
            && self.peek(3).kind == TokenKind::LParen
        {
            return Some(vec![self.parse_method_call()]);
        }
        if self.at(TokenKind::Identifier) || self.at(TokenKind::Keyword) || self.at(TokenKind::LParen) {
            return Some(vec![self.parse_assignment()]);
        }

        self.diagnostics.error(
            "PAR001",
            format!("Unexpected token '{}'.", if !t.lexeme.is_empty() { &t.lexeme } else { &t.upper }),
            t.start,
            t.end,
        );
        None
    }

    fn parse_report(&mut self) -> Option<Statement> {
        let start = self.consume();
        let name = self.expect_identifier("Expected report name after REPORT.");
        self.expect_dot("Expected \".\" after REPORT.");
        if let Some(n) = name {
            self.report_name = Some(n.lexeme);
        }
        Some(Statement::EmptyStmt {
            start: start.start,
            end: self.previous().end,
        })
    }

    fn parse_simple_header(&mut self) -> Option<Statement> {
        let s = self.consume();
        self.collect_until_dot();
        Some(Statement::EmptyStmt {
            start: s.start,
            end: self.previous().end,
        })
    }

    fn parse_inline_data_assignment(&mut self) -> Option<Statement> {
        let start = self.consume();
        self.expect(TokenKind::LParen, "Expected ( after inline DATA.");
        let name = self.expect_identifier("Expected inline data object name.");
        self.expect(TokenKind::RParen, "Expected ) after inline data object name.");
        self.expect_operator("=", "Expected = after inline DATA declaration.");
        let value = self.parse_expression(0);
        let end = self.expect_dot("Expected period after inline DATA assignment.");
        let name_tok = name?;
        Some(Statement::InlineDataAssign {
            name: name_tok.lexeme,
            value: value.clone(),
            start: start.start,
            end: end.map(|e| e.end).unwrap_or(value.end_pos()),
        })
    }

    fn parse_data(&mut self) -> Option<Vec<Statement>> {
        let start = self.consume();
        let mut out = Vec::new();
        if self.at(TokenKind::Colon) {
            self.consume();
        }

        while !self.at(TokenKind::EOF) {
            if self.is_keyword("BEGIN") || self.is_keyword("END") {
                break;
            }
            let name = self.expect_identifier("Expected data object name.");
            let name_tok = match name {
                Some(n) => n,
                None => {
                    self.recover_statement();
                    break;
                }
            };
            self.expect_keyword("TYPE", "Expected TYPE in DATA declaration.");
            let type_ref = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
            let mut value = None;
            if self.match_keyword("VALUE") {
                value = Some(self.parse_expression(0));
            }

            let end_token = if self.match_kind(TokenKind::Comma) {
                Some(self.previous().clone())
            } else {
                None
            };

            let end_pos = if let Some(e) = end_token {
                e.end
            } else if self.at(TokenKind::Dot) {
                self.current().end
            } else if let Some(v) = &value {
                v.end_pos()
            } else {
                name_tok.end
            };

            out.push(Statement::DataDecl {
                start: start.start,
                end: end_pos,
                name: name_tok.lexeme,
                type_ref,
                value,
                inline: Some(false),
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
            });

            if self.at(TokenKind::Dot) {
                self.consume();
                break;
            }
            if !self.at(TokenKind::Comma) && !self.match_kind(TokenKind::Comma) {
                self.diagnostics.error(
                    "PAR002",
                    "Expected comma or period in DATA declaration.",
                    self.current().start,
                    self.current().end,
                );
                self.recover_statement();
                break;
            }
        }
        Some(out)
    }

    fn parse_parameters(&mut self) -> Option<Vec<Statement>> {
        let start = self.consume();
        let mut out = Vec::new();
        self.match_kind(TokenKind::Colon);

        while !self.at(TokenKind::EOF) {
            let name = self.expect_identifier("Expected parameter name after PARAMETERS.");
            let name_tok = match name {
                Some(n) => n,
                None => {
                    self.recover_statement();
                    break;
                }
            };

            let mut type_ref = TypeRef::scalar("any");
            let mut type_specified = false;

            if self.match_keyword("TYPE") {
                type_ref = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
                type_specified = true;
            } else if self.match_keyword("LIKE") {
                let like = self.expect_identifier("Expected data object after LIKE.");
                type_ref = TypeRef::Named {
                    name: like.map(|l| l.lexeme).unwrap_or_else(|| "any".to_string()),
                };
                type_specified = true;
            }

            let mut value = None;
            let mut obligatory = false;
            let mut checkbox = false;
            let mut lower_case = false;
            let mut no_display = false;
            let mut radiobutton_group = None;
            let mut value_help = None;

            while !self.at(TokenKind::EOF) && !self.at(TokenKind::Comma) && !self.at(TokenKind::Dot) {
                if self.match_keyword("DEFAULT") {
                    value = Some(self.parse_expression(0));
                    continue;
                }
                if self.match_keyword("OBLIGATORY") {
                    obligatory = true;
                    continue;
                }
                if self.match_keyword("RADIOBUTTON") {
                    self.expect_keyword("GROUP", "Expected GROUP after RADIOBUTTON.");
                    let g = self.expect_identifier("Expected radio-button group name.");
                    radiobutton_group = g.map(|tok| tok.lexeme);
                    type_ref = TypeRef::scalar("abap_bool");
                    continue;
                }
                if self.match_keyword("AS") {
                    if self.match_keyword("CHECKBOX") {
                        checkbox = true;
                        type_ref = TypeRef::scalar("abap_bool");
                        continue;
                    }
                    if self.match_keyword("RADIOBUTTON") {
                        self.expect_keyword("GROUP", "Expected GROUP after AS RADIOBUTTON.");
                        let g = self.expect_identifier("Expected radio-button group name.");
                        radiobutton_group = g.map(|tok| tok.lexeme);
                        type_ref = TypeRef::scalar("abap_bool");
                        continue;
                    }
                    self.diagnostics.error(
                        "PAR004",
                        "Supported PARAMETERS additions after AS are CHECKBOX and RADIOBUTTON GROUP.",
                        self.current().start,
                        self.current().end,
                    );
                    if !self.at(TokenKind::EOF) {
                        self.consume();
                    }
                    continue;
                }
                if self.match_keyword("LOWER") {
                    self.match_keyword("CASE");
                    lower_case = true;
                    continue;
                }
                if self.match_keyword("NO-DISPLAY") {
                    no_display = true;
                    continue;
                }
                if self.match_keyword("MATCHCODE") {
                    self.match_keyword("OBJECT");
                    value_help = self.expect_identifier("Expected matchcode object name.").map(|t| t.lexeme);
                    continue;
                }
                if self.match_keyword("USER-COMMAND") {
                    self.expect_identifier("Expected user command code.");
                    continue;
                }
                self.diagnostics.error(
                    "PAR005",
                    format!(
                        "Unsupported PARAMETERS addition '{}'.",
                        if !self.current().lexeme.is_empty() { &self.current().lexeme } else { &self.current().upper }
                    ),
                    self.current().start,
                    self.current().end,
                );
                self.consume();
            }

            if !type_specified && !checkbox && radiobutton_group.is_none() {
                self.diagnostics.error(
                    "PAR003",
                    "PARAMETERS requires TYPE or LIKE unless AS CHECKBOX or RADIOBUTTON GROUP is specified.",
                    name_tok.start,
                    name_tok.end,
                );
            }

            out.push(Statement::DataDecl {
                start: start.start,
                end: self.previous().end,
                name: name_tok.lexeme,
                type_ref,
                value,
                inline: Some(false),
                parameter: Some(true),
                obligatory: Some(obligatory),
                checkbox: Some(checkbox),
                selection_options: Some(false),
                no_intervals: None,
                no_extension: None,
                for_field: None,
                radiobutton_group,
                lower_case: Some(lower_case),
                no_display: Some(no_display),
                value_help,
            });

            if self.at(TokenKind::Dot) {
                self.consume();
                break;
            }
            if !self.match_kind(TokenKind::Comma) {
                self.diagnostics.error(
                    "PAR002",
                    "Expected comma or period in PARAMETERS declaration.",
                    self.current().start,
                    self.current().end,
                );
                self.recover_statement();
                break;
            }
        }
        Some(out)
    }

    fn parse_select_options(&mut self) -> Vec<Statement> {
        let start = self.consume();
        let mut out = Vec::new();
        self.match_kind(TokenKind::Colon);

        while !self.at(TokenKind::EOF) {
            let name = self.expect_identifier("Expected range-table name after SELECT-OPTIONS.");
            let name_tok = match name {
                Some(n) => n,
                None => {
                    self.recover_statement();
                    break;
                }
            };

            self.expect_keyword("FOR", "Expected FOR in SELECT-OPTIONS.");
            let mut ref_tokens = Vec::new();
            let additions: HashSet<&str> = [
                "NO-INTERVALS",
                "NO-EXTENSION",
                "OBLIGATORY",
                "MATCHCODE",
                "MEMORY",
                "MODIF",
                "NO-DISPLAY",
                "DEFAULT",
                "USER-COMMAND",
                "LOWER",
            ]
            .into_iter()
            .collect();

            while !self.at(TokenKind::EOF)
                && !self.at(TokenKind::Comma)
                && !self.at(TokenKind::Dot)
                && !additions.contains(self.current().upper.as_str())
                && !(self.current().upper == "NO"
                    && matches!(self.peek(1).upper.as_str(), "INTERVALS" | "EXTENSION" | "DISPLAY"))
            {
                ref_tokens.push(self.consume());
            }
            let for_field = ref_tokens.into_iter().map(|t| t.lexeme).collect::<Vec<_>>().join("");

            let mut no_intervals = false;
            let mut no_extension = false;
            let mut obligatory = false;
            let mut no_display = false;
            let mut value_help = None;
            let mut selection_default = None;

            while !self.at(TokenKind::EOF) && !self.at(TokenKind::Comma) && !self.at(TokenKind::Dot) {
                if self.match_keyword("NO-INTERVALS")
                    || (self.match_keyword("NO") && self.match_keyword("INTERVALS"))
                {
                    no_intervals = true;
                    continue;
                }
                if self.match_keyword("NO-EXTENSION")
                    || (self.match_keyword("NO") && self.match_keyword("EXTENSION"))
                {
                    no_extension = true;
                    continue;
                }
                if self.match_keyword("NO-DISPLAY")
                    || (self.match_keyword("NO") && self.match_keyword("DISPLAY"))
                {
                    no_display = true;
                    continue;
                }
                if self.match_keyword("OBLIGATORY") {
                    obligatory = true;
                    continue;
                }
                if self.match_keyword("MATCHCODE") {
                    self.match_keyword("OBJECT");
                    value_help = self.expect_identifier("Expected matchcode object name.").map(|t| t.lexeme);
                    continue;
                }
                if self.match_keyword("MEMORY") {
                    self.match_keyword("ID");
                    self.expect_identifier("Expected memory ID.");
                    continue;
                }
                if self.match_keyword("MODIF") {
                    self.match_keyword("ID");
                    self.expect_identifier("Expected modification group ID.");
                    continue;
                }
                if self.match_keyword("USER-COMMAND") {
                    self.expect_identifier("Expected user command code.");
                    continue;
                }
                if self.match_keyword("LOWER") {
                    self.match_keyword("CASE");
                    continue;
                }
                if self.match_keyword("DEFAULT") {
                    let low = self.parse_expression(0);
                    let mut high = None;
                    if self.match_keyword("TO") {
                        high = Some(self.parse_expression(0));
                    }
                    let lit = |val: &str, start, end| Expr::Literal {
                        start,
                        end,
                        value: serde_json::Value::String(val.to_string()),
                        type_ref: TypeRef::scalar("string"),
                    };

                    let low_start = low.start_pos();
                    let low_end = low.end_pos();
                    let mut assignments = vec![
                        ConstructorAssignment {
                            name: "SIGN".to_string(),
                            value: lit("I", low_start, low_end),
                        },
                        ConstructorAssignment {
                            name: "OPTION".to_string(),
                            value: lit(if high.is_some() { "BT" } else { "EQ" }, low_start, low_end),
                        },
                        ConstructorAssignment {
                            name: "LOW".to_string(),
                            value: low,
                        },
                    ];
                    let end_pos = if let Some(h) = high {
                        let h_end = h.end_pos();
                        assignments.push(ConstructorAssignment {
                            name: "HIGH".to_string(),
                            value: h,
                        });
                        h_end
                    } else {
                        low_end
                    };
                    selection_default = Some(Expr::Constructor {
                        start: low_start,
                        end: end_pos,
                        assignments,
                    });
                    continue;
                }

                self.diagnostics.error(
                    "PAR006",
                    format!(
                        "Unsupported SELECT-OPTIONS addition '{}'.",
                        if !self.current().lexeme.is_empty() { &self.current().lexeme } else { &self.current().upper }
                    ),
                    self.current().start,
                    self.current().end,
                );
                self.consume();
            }

            let mut range_fields = BTreeMap::new();
            range_fields.insert("SIGN".to_string(), TypeRef::scalar("c"));
            range_fields.insert("OPTION".to_string(), TypeRef::scalar("c"));
            range_fields.insert("LOW".to_string(), TypeRef::scalar("string"));
            range_fields.insert("HIGH".to_string(), TypeRef::scalar("string"));

            let range_type = TypeRef::Table {
                line: Box::new(TypeRef::Struct {
                    name: format!("{}_RANGE", name_tok.lexeme),
                    fields: range_fields,
                }),
                table_kind: "standard".to_string(),
                unique: None,
            };

            out.push(Statement::DataDecl {
                start: start.start,
                end: self.previous().end,
                name: name_tok.lexeme,
                type_ref: range_type,
                value: selection_default,
                inline: Some(false),
                parameter: Some(true),
                obligatory: Some(obligatory),
                checkbox: None,
                selection_options: Some(true),
                no_intervals: Some(no_intervals),
                no_extension: Some(no_extension),
                for_field: Some(for_field),
                radiobutton_group: None,
                lower_case: None,
                no_display: Some(no_display),
                value_help,
            });

            if self.at(TokenKind::Dot) {
                self.consume();
                break;
            }
            if !self.match_kind(TokenKind::Comma) {
                self.diagnostics.error(
                    "PAR002",
                    "Expected comma or period in SELECT-OPTIONS declaration.",
                    self.current().start,
                    self.current().end,
                );
                self.recover_statement();
                break;
            }
        }
        out
    }

    fn parse_selection_screen_decl(&mut self) -> Statement {
        let s = self.consume();
        self.collect_until_dot();
        Statement::EmptyStmt {
            start: s.start,
            end: self.previous().end,
        }
    }

    fn is_event_header_ahead(&self) -> bool {
        matches!(
            self.current().upper.as_str(),
            "INITIALIZATION" | "START-OF-SELECTION" | "END-OF-SELECTION"
        ) || (self.is_keyword("AT") && self.peek(1).upper == "SELECTION-SCREEN")
    }

    fn parse_event_block(&mut self) -> Statement {
        let s = self.current().clone();
        let mut event = "start-of-selection".to_string();
        let mut target = None;

        if self.match_keyword("INITIALIZATION") {
            event = "initialization".to_string();
        } else if self.match_keyword("START-OF-SELECTION") {
            event = "start-of-selection".to_string();
        } else if self.match_keyword("END-OF-SELECTION") {
            event = "end-of-selection".to_string();
        } else {
            self.expect_keyword("AT", "Expected AT in selection-screen event.");
            self.expect_keyword("SELECTION-SCREEN", "Expected SELECTION-SCREEN after AT.");
            if self.match_keyword("OUTPUT") {
                event = "at-selection-screen-output".to_string();
            } else if self.match_keyword("ON") {
                if self.match_keyword("VALUE-REQUEST") {
                    event = "at-selection-screen-value-request".to_string();
                    self.match_keyword("FOR");
                    target = self.parse_selection_field_reference("Expected field name after VALUE-REQUEST FOR.");
                } else if self.match_keyword("HELP-REQUEST") {
                    event = "at-selection-screen-help-request".to_string();
                    self.match_keyword("FOR");
                    target = self.parse_selection_field_reference("Expected field name after HELP-REQUEST FOR.");
                } else {
                    event = "at-selection-screen-on".to_string();
                    if self.match_keyword("BLOCK") {
                        target = self.expect_identifier("Expected block name.").map(|t| t.lexeme);
                    } else {
                        target = self.parse_selection_field_reference("Expected field name after ON.");
                    }
                }
            } else {
                event = "at-selection-screen".to_string();
            }
        }
        self.expect_dot("Expected period after selection event header.");
        let empty_set = HashSet::new();
        let statements = self.parse_block(&empty_set);
        let end_pos = statements.last().map(|st| st.end_pos()).unwrap_or(self.previous().end);

        Statement::EventBlock {
            start: s.start,
            end: end_pos,
            event,
            target,
            statements,
        }
    }

    fn parse_selection_field_reference(&mut self, msg: &str) -> Option<String> {
        let first = self.expect_identifier(msg)?;
        let mut name = first.lexeme;
        while self.current().lexeme == "-" && self.current().start.offset == self.previous().end.offset {
            self.consume();
            if let Some(part) = self.expect_identifier("Expected field suffix after -.") {
                name.push('-');
                name.push_str(&part.lexeme);
            } else {
                break;
            }
        }
        Some(name)
    }

    fn parse_translate(&mut self) -> Statement {
        let s = self.consume();
        let target = self.parse_postfix();
        let mut mode = "upper".to_string();
        let mut mask = None;

        if self.match_keyword("TO") {
            if self.match_keyword("LOWER") {
                mode = "lower".to_string();
            } else {
                self.expect_keyword("UPPER", "Expected UPPER or LOWER after TRANSLATE TO.");
                mode = "upper".to_string();
            }
            self.expect_keyword("CASE", "Expected CASE after TRANSLATE TO UPPER/LOWER.");
        } else if self.match_keyword("USING") {
            mode = "using".to_string();
            mask = Some(self.parse_expression(0));
        } else {
            self.diagnostics.error(
                "PAR050",
                "Expected TO UPPER CASE, TO LOWER CASE, or USING after TRANSLATE target.",
                self.current().start,
                self.current().end,
            );
        }

        let e = self.expect_dot("Expected period after TRANSLATE.");
        Statement::TranslateStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            target,
            mode,
            mask,
        }
    }

    fn parse_concatenate(&mut self) -> Statement {
        let s = self.consume();
        let mut sources = Vec::new();
        while !self.at(TokenKind::EOF) && !self.is_keyword("INTO") && !self.at(TokenKind::Dot) {
            let before = self.i;
            sources.push(self.parse_expression(0));
            if self.i == before {
                break;
            }
        }
        self.expect_keyword("INTO", "Expected INTO in CONCATENATE.");
        let target = self.parse_postfix();
        let mut separator = None;
        let mut respecting_blanks = false;

        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
            if self.match_keyword("SEPARATED") {
                self.expect_keyword("BY", "Expected BY after SEPARATED.");
                separator = Some(self.parse_expression(0));
                continue;
            }
            if self.match_keyword("RESPECTING") {
                self.expect_keyword("BLANKS", "Expected BLANKS after RESPECTING.");
                respecting_blanks = true;
                continue;
            }
            if self.match_keyword("IN") {
                self.match_keyword("CHARACTER");
                self.match_keyword("BYTE");
                self.match_keyword("MODE");
                continue;
            }
            self.diagnostics.error(
                "PAR051",
                format!(
                    "Unsupported CONCATENATE addition '{}'.",
                    if !self.current().lexeme.is_empty() { &self.current().lexeme } else { &self.current().upper }
                ),
                self.current().start,
                self.current().end,
            );
            self.consume();
        }

        let e = self.expect_dot("Expected period after CONCATENATE.");
        Statement::ConcatenateStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            sources,
            target,
            separator,
            respecting_blanks,
        }
    }

    fn parse_split(&mut self) -> Statement {
        let s = self.consume();
        let source = self.parse_expression(0);
        self.expect_keyword("AT", "Expected AT in SPLIT.");
        let separator = self.parse_expression(0);
        self.expect_keyword("INTO", "Expected INTO in SPLIT.");

        let mut targets = Vec::new();
        let mut table_target = None;

        if self.match_keyword("TABLE") {
            table_target = Some(self.parse_postfix());
        } else {
            while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
                if self.match_keyword("IN") {
                    self.match_keyword("CHARACTER");
                    self.match_keyword("BYTE");
                    self.match_keyword("MODE");
                    continue;
                }
                let n = self.parse_postfix();
                targets.push(n);
            }
        }

        let e = self.expect_dot("Expected period after SPLIT.");
        Statement::SplitStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            source,
            separator,
            targets,
            table_target,
        }
    }

    fn parse_condense(&mut self) -> Statement {
        let s = self.consume();
        let target = self.parse_postfix();
        let no_gaps = self.match_keyword("NO-GAPS") || (self.match_keyword("NO") && self.match_keyword("GAPS"));
        let e = self.expect_dot("Expected period after CONDENSE.");
        Statement::CondenseStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            target,
            no_gaps,
        }
    }

    fn parse_message(&mut self) -> Statement {
        let s = self.consume();
        let text = self.parse_expression(0);
        self.expect_keyword("TYPE", "Expected TYPE in MESSAGE statement.");
        let message_type = self.parse_expression(0);
        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
            self.consume();
        }
        let e = self.expect_dot("Expected period after MESSAGE.");
        Statement::MessageStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            text,
            message_type,
        }
    }

    fn parse_constants(&mut self) -> Option<Vec<Statement>> {
        let start = self.consume();
        let mut out = Vec::new();
        self.match_kind(TokenKind::Colon);

        while !self.at(TokenKind::EOF) {
            let name = self.expect_identifier("Expected constant name.");
            let name_tok = match name {
                Some(n) => n,
                None => break,
            };
            self.expect_keyword("TYPE", "Expected TYPE in CONSTANTS declaration.");
            let type_ref = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
            self.expect_keyword("VALUE", "Expected VALUE in CONSTANTS declaration.");
            let value = self.parse_expression(0);
            let val_end = value.end_pos();

            out.push(Statement::ConstantDecl {
                start: start.start,
                end: val_end,
                name: name_tok.lexeme,
                type_ref,
                value,
            });

            if self.at(TokenKind::Dot) {
                self.consume();
                break;
            }
            self.expect(TokenKind::Comma, "Expected comma between CONSTANTS declarations.");
        }
        Some(out)
    }

    fn parse_types(&mut self) -> Option<Vec<Statement>> {
        let start = self.consume();
        let mut out = Vec::new();
        self.match_kind(TokenKind::Colon);

        while !self.at(TokenKind::EOF) {
            if self.match_keyword("BEGIN") && self.match_keyword("OF") {
                let type_name = self.expect_identifier("Expected structure type name.");
                self.match_kind(TokenKind::Comma);
                let mut fields = BTreeMap::new();
                while !self.at(TokenKind::EOF) && !(self.is_keyword("END") && self.is_next_keyword("OF")) {
                    let field = self.expect_identifier("Expected structure component.");
                    if let Some(f) = field {
                        self.expect_keyword("TYPE", "Expected TYPE for structure component.");
                        let ty = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
                        fields.insert(f.lexeme.to_ascii_uppercase(), ty);
                        if self.at(TokenKind::Comma) {
                            self.consume();
                        } else if !self.is_keyword("END") {
                            self.recover_to_comma_or_dot();
                        }
                    } else {
                        self.recover_to_comma_or_dot();
                    }
                }
                self.expect_keyword("END", "Expected END OF in structure type.");
                self.expect_keyword("OF", "Expected OF in structure type.");
                if type_name.is_some() && self.at(TokenKind::Identifier) {
                    self.consume();
                }
                let dot = self.expect_dot("Expected \".\" after END OF type.");
                if let Some(tn) = type_name {
                    let node = TypeDecl {
                        name: tn.lexeme.clone(),
                        type_ref: TypeRef::Struct {
                            name: tn.lexeme,
                            fields,
                        },
                        start: start.start,
                        end: dot.map(|d| d.end).unwrap_or(start.end),
                    };
                    self.types.push(node.clone());
                    out.push(Statement::TypeDecl(node));
                }
                break;
            }

            let name = self.expect_identifier("Expected type name.");
            let name_tok = match name {
                Some(n) => n,
                None => break,
            };
            self.expect_keyword("TYPE", "Expected TYPE in TYPES declaration.");
            let type_ref = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
            let node = TypeDecl {
                name: name_tok.lexeme,
                type_ref,
                start: start.start,
                end: self.previous().end,
            };
            self.types.push(node.clone());
            out.push(Statement::TypeDecl(node));

            if self.at(TokenKind::Dot) {
                self.consume();
                break;
            }
            self.expect(TokenKind::Comma, "Expected comma or period in TYPES declaration.");
        }
        Some(out)
    }

    fn parse_type(&mut self) -> Option<TypeRef> {
        if self.match_keyword("TABLE") {
            let mut table_kind = "standard".to_string();
            if self.match_keyword("SORTED") {
                table_kind = "sorted".to_string();
            } else if self.match_keyword("HASHED") {
                table_kind = "hashed".to_string();
            }
            self.expect_keyword("OF", "Expected OF in TABLE type.");
            let line = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
            let mut unique = false;
            if self.match_keyword("WITH") {
                if self.match_keyword("UNIQUE") {
                    unique = true;
                }
                self.match_keyword("EMPTY");
                self.match_keyword("KEY");
            }
            return Some(TypeRef::Table {
                line: Box::new(line),
                table_kind,
                unique: Some(unique),
            });
        }
        if self.match_keyword("REF") {
            self.match_keyword("TO");
            let n = self.expect_identifier("Expected referenced type name.");
            return Some(TypeRef::Named {
                name: n.map(|tok| tok.lexeme).unwrap_or_else(|| "any".to_string()),
            });
        }

        let t = self.current().clone();
        if t.kind == TokenKind::Keyword || t.kind == TokenKind::Identifier {
            let u = t.upper.to_ascii_lowercase();
            let scalar_name = match u.as_str() {
                "i" | "int4" | "int2" => Some("i"),
                "int8" => Some("int8"),
                "decfloat16" => Some("decfloat16"),
                "decfloat34" | "p" | "packed" => Some("decfloat34"),
                "string" => Some("string"),
                "xstring" => Some("xstring"),
                "c" | "char" => Some("c"),
                "n" | "numc" => Some("n"),
                "d" | "dats" => Some("d"),
                "t" | "tims" => Some("t"),
                "abap_bool" | "boolean" => Some("abap_bool"),
                _ => None,
            };

            if let Some(sname) = scalar_name {
                self.consume();
                let mut length = None;
                if self.at(TokenKind::LParen) {
                    self.consume();
                    if self.at(TokenKind::Integer) {
                        length = self.consume().lexeme.parse::<usize>().ok();
                    }
                    self.expect(TokenKind::RParen, "Expected ) after type length.");
                }
                return Some(TypeRef::Scalar {
                    name: sname.to_string(),
                    length,
                });
            }

            self.consume();
            return Some(TypeRef::Named { name: t.lexeme });
        }
        None
    }

    fn parse_if(&mut self) -> Statement {
        let start = self.consume();
        let mut branches = Vec::new();
        let condition = self.parse_expression(0);
        self.expect_dot("Expected \".\" after IF condition.");

        let mut end_set = HashSet::new();
        end_set.insert("ELSEIF".to_string());
        end_set.insert("ELSE".to_string());
        end_set.insert("ENDIF".to_string());

        let mut body = self.parse_block(&end_set);
        branches.push(IfBranch {
            condition,
            statements: body,
        });

        while self.match_keyword("ELSEIF") {
            let c = self.parse_expression(0);
            self.expect_dot("Expected \".\" after ELSEIF condition.");
            body = self.parse_block(&end_set);
            branches.push(IfBranch {
                condition: c,
                statements: body,
            });
        }

        let mut else_branch = Vec::new();
        if self.match_keyword("ELSE") {
            self.expect_dot("Expected \".\" after ELSE.");
            let mut endif_set = HashSet::new();
            endif_set.insert("ENDIF".to_string());
            else_branch = self.parse_block(&endif_set);
        }

        self.expect_keyword("ENDIF", "Expected ENDIF.");
        let e = self.expect_dot("Expected \".\" after ENDIF.");
        Statement::IfStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            branches,
            else_branch,
        }
    }

    fn parse_while(&mut self) -> Statement {
        let start = self.consume();
        let condition = self.parse_expression(0);
        self.expect_dot("Expected \".\" after WHILE.");
        let mut end_set = HashSet::new();
        end_set.insert("ENDWHILE".to_string());
        let body = self.parse_block(&end_set);
        self.expect_keyword("ENDWHILE", "Expected ENDWHILE.");
        let e = self.expect_dot("Expected \".\" after ENDWHILE.");
        Statement::WhileStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            condition,
            body,
        }
    }

    fn parse_do(&mut self) -> Statement {
        let start = self.consume();
        let mut count = None;
        if !self.at(TokenKind::Dot) {
            count = Some(self.parse_expression(0));
            self.match_keyword("TIMES");
        }
        self.expect_dot("Expected \".\" after DO.");
        let mut end_set = HashSet::new();
        end_set.insert("ENDDO".to_string());
        let body = self.parse_block(&end_set);
        self.expect_keyword("ENDDO", "Expected ENDDO.");
        let e = self.expect_dot("Expected \".\" after ENDDO.");
        Statement::DoStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            count,
            body,
        }
    }

    fn parse_case(&mut self) -> Statement {
        let start = self.consume();
        let expression = self.parse_expression(0);
        self.expect_dot("Expected \".\" after CASE.");
        let mut whens = Vec::new();
        let mut others = Vec::new();

        while !self.at(TokenKind::EOF) && !self.is_keyword("ENDCASE") {
            if self.match_keyword("WHEN") {
                if self.match_keyword("OTHERS") {
                    self.expect_dot("Expected \".\" after WHEN OTHERS.");
                    let mut end_set = HashSet::new();
                    end_set.insert("ENDCASE".to_string());
                    others = self.parse_block(&end_set);
                    continue;
                }
                let mut values = Vec::new();
                loop {
                    values.push(self.parse_expression(0));
                    if !self.match_kind(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect_dot("Expected \".\" after WHEN.");
                let mut when_set = HashSet::new();
                when_set.insert("WHEN".to_string());
                when_set.insert("ENDCASE".to_string());
                let statements = self.parse_block(&when_set);
                whens.push(CaseWhen { values, statements });
            } else {
                self.diagnostics.error(
                    "PAR010",
                    "Expected WHEN or ENDCASE.",
                    self.current().start,
                    self.current().end,
                );
                self.recover_statement();
            }
        }
        self.expect_keyword("ENDCASE", "Expected ENDCASE.");
        let e = self.expect_dot("Expected \".\" after ENDCASE.");
        Statement::CaseStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            expression,
            whens,
            others,
        }
    }

    fn parse_loop(&mut self) -> Statement {
        let start = self.consume();
        self.expect_keyword("AT", "Expected AT after LOOP.");
        let source = self.parse_expression(0);
        let mut target = None;
        let mut inline = false;

        if self.match_keyword("INTO") || self.match_keyword("ASSIGNING") {
            self.match_keyword("FIELD-SYMBOL");
            self.match_kind(TokenKind::At);
            if self.match_keyword("DATA") {
                self.match_kind(TokenKind::LParen);
                let n = self.expect_identifier("Expected inline variable name.");
                self.expect(TokenKind::RParen, "Expected ).");
                target = n.map(|t| t.lexeme);
                inline = true;
            } else if self.at(TokenKind::LParen) {
                self.consume();
                let n = self.expect_identifier("Expected inline variable name.");
                self.expect(TokenKind::RParen, "Expected ).");
                target = n.map(|t| t.lexeme);
                inline = true;
            } else {
                let n = self.expect_identifier("Expected loop target.");
                target = n.map(|t| t.lexeme);
            }
        }
        self.expect_dot("Expected \".\" after LOOP header.");
        let mut end_set = HashSet::new();
        end_set.insert("ENDLOOP".to_string());
        let body = self.parse_block(&end_set);
        self.expect_keyword("ENDLOOP", "Expected ENDLOOP.");
        let e = self.expect_dot("Expected \".\" after ENDLOOP.");

        Statement::LoopStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            source,
            target,
            inline: Some(inline),
            body,
        }
    }

    fn parse_append(&mut self) -> Statement {
        let start = self.consume();
        let value = self.parse_expression(0);
        self.expect_keyword("TO", "Expected TO in APPEND.");
        let table = self.parse_expression(0);
        let e = self.expect_dot("Expected \".\" after APPEND.");
        Statement::AppendStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            value,
            table,
        }
    }

    fn parse_insert(&mut self) -> Statement {
        let start = self.consume();
        let value = self.parse_expression(0);
        self.expect_keyword("INTO", "Expected INTO in INSERT.");
        self.match_keyword("TABLE");
        let table = self.parse_expression(0);
        let e = self.expect_dot("Expected \".\" after INSERT.");
        Statement::InsertStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            value,
            table,
        }
    }

    fn parse_read_table(&mut self) -> Statement {
        let start = self.consume();
        self.expect_keyword("TABLE", "Expected TABLE after READ.");
        let table = self.parse_expression(0);
        let mut target = None;
        let mut index = None;
        let mut inline = false;
        let mut key = BTreeMap::new();

        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
            if self.match_keyword("INTO") {
                self.match_kind(TokenKind::At);
                if self.match_keyword("DATA") {
                    self.match_kind(TokenKind::LParen);
                    let n = self.expect_identifier("Expected inline variable.");
                    self.match_kind(TokenKind::RParen);
                    target = n.map(|t| t.lexeme);
                    inline = true;
                } else {
                    target = self.expect_identifier("Expected target variable.").map(|t| t.lexeme);
                }
            } else if self.match_keyword("INDEX") {
                index = Some(self.parse_expression(0));
            } else if self.match_keyword("WITH") {
                self.match_keyword("KEY");
                while !self.at(TokenKind::Dot) && !self.is_keyword("INDEX") && !self.is_keyword("INTO") {
                    let k = self.expect_identifier("Expected key field.");
                    self.expect_operator("=", "Expected = in key.");
                    let v = self.parse_expression(0);
                    if let Some(tok) = k {
                        key.insert(tok.lexeme.to_ascii_uppercase(), v);
                    }
                    if self.at(TokenKind::Comma) {
                        self.consume();
                    } else if !self.at(TokenKind::Dot) {
                        break;
                    }
                }
            } else {
                self.consume();
            }
        }
        let e = self.expect_dot("Expected \".\" after READ TABLE.");
        Statement::ReadTableStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            table,
            target,
            index,
            key: if !key.is_empty() { Some(key) } else { None },
            inline: Some(inline),
        }
    }

    fn parse_delete_table(&mut self) -> Statement {
        let start = self.consume();
        let table = self.parse_expression(0);
        let mut index = None;
        let mut key = BTreeMap::new();

        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
            if self.match_keyword("INDEX") {
                index = Some(self.parse_expression(0));
            } else if self.match_keyword("WHERE") {
                while !self.at(TokenKind::Dot) {
                    let k = self.expect_identifier("Expected key field.");
                    self.expect_operator("=", "Expected = in WHERE.");
                    let v = self.parse_expression(0);
                    if let Some(tok) = k {
                        key.insert(tok.lexeme.to_ascii_uppercase(), v);
                    }
                    if self.at(TokenKind::Keyword) && self.current().upper == "AND" {
                        self.consume();
                    } else if !self.at(TokenKind::Dot) {
                        break;
                    }
                }
            } else {
                self.consume();
            }
        }
        let e = self.expect_dot("Expected \".\" after DELETE.");
        Statement::DeleteTableStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            table,
            index,
            key: if !key.is_empty() { Some(key) } else { None },
        }
    }

    fn parse_sort(&mut self) -> Statement {
        let start = self.consume();
        let table = self.parse_expression(0);
        let mut keys = Vec::new();
        if self.match_keyword("BY") {
            while !self.at(TokenKind::Dot) {
                let n = self.expect_identifier("Expected sort field.");
                let mut descending = false;
                if self.match_keyword("DESCENDING") {
                    descending = true;
                } else {
                    self.match_keyword("ASCENDING");
                }
                if let Some(tok) = n {
                    keys.push(SortKey {
                        name: tok.lexeme,
                        descending,
                    });
                }
                if self.at(TokenKind::Comma) {
                    self.consume();
                } else if !self.at(TokenKind::Dot) {
                    break;
                }
            }
        }
        let e = self.expect_dot("Expected \".\" after SORT.");
        Statement::SortStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            table,
            keys,
        }
    }

    fn parse_describe(&mut self) -> Statement {
        let start = self.consume();
        self.expect_keyword("TABLE", "Expected TABLE after DESCRIBE.");
        let table = self.parse_expression(0);
        self.expect_keyword("LINES", "Expected LINES after table.");
        let target = self
            .expect_identifier("Expected target variable.")
            .map(|t| t.lexeme)
            .unwrap_or_default();
        let e = self.expect_dot("Expected \".\" after DESCRIBE TABLE.");
        Statement::DescribeTableStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            table,
            target,
        }
    }

    fn parse_clear(&mut self) -> Statement {
        let start = self.consume();
        let target = self.parse_expression(0);
        let e = self.expect_dot("Expected \".\" after CLEAR.");
        Statement::ClearStmt {
            start: start.start,
            end: e.map(|tok| tok.end).unwrap_or(start.end),
            target,
        }
    }

    fn parse_flow(&mut self) -> Statement {
        let t = self.consume();
        let op = t.upper.to_ascii_lowercase();
        let mut condition = None;
        if op == "check" && !self.at(TokenKind::Dot) {
            condition = Some(self.parse_expression(0));
        }
        let e = if self.at(TokenKind::Dot) {
            self.consume()
        } else {
            self.expect_dot("Expected \".\" after flow statement.").unwrap_or(t.clone())
        };
        Statement::FlowStmt {
            start: t.start,
            end: e.end,
            op,
            condition,
        }
    }

    fn parse_assert(&mut self) -> Statement {
        let s = self.consume();
        let condition = self.parse_expression(0);
        let e = self.expect_dot("Expected \".\" after ASSERT.");
        Statement::AssertStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            condition,
        }
    }

    fn parse_write(&mut self) -> Statement {
        let s = self.consume();
        self.match_kind(TokenKind::Colon);
        let mut groups: Vec<Vec<Expr>> = vec![Vec::new()];
        let mut values = Vec::new();

        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
            if self.at(TokenKind::Comma) {
                self.consume();
                continue;
            }
            if self.current().lexeme == "/" {
                self.consume();
                if let Some(last) = groups.last() {
                    if !last.is_empty() {
                        groups.push(Vec::new());
                    }
                }
                continue;
            }
            let value = self.parse_expression(0);
            values.push(value.clone());
            if let Some(last) = groups.last_mut() {
                last.push(value);
            }
            if self.at(TokenKind::Comma) {
                self.consume();
            } else if !self.at(TokenKind::Dot) && self.current().lexeme != "/" {
                break;
            }
        }
        let e = self.expect_dot("Expected \".\" after WRITE.");
        groups.retain(|g| !g.is_empty());
        Statement::WriteStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            values,
            groups,
        }
    }

    fn parse_eml_data_target(&mut self, label: &str) -> Option<String> {
        self.expect_keyword(label, &format!("Expected {} in EML statement.", label));
        self.expect_keyword("DATA", &format!("Expected DATA after {}.", label));
        self.expect(TokenKind::LParen, &format!("Expected ( after {} DATA.", label));
        let target = self
            .expect_identifier(&format!("Expected target name for {}.", label))
            .map(|t| t.lexeme);
        self.expect(TokenKind::RParen, &format!("Expected ) after {} target.", label));
        target
    }

    fn parse_eml_rows(&mut self) -> Vec<Vec<EmlFieldBinding>> {
        self.expect_keyword("VALUE", "Expected VALUE after WITH.");
        self.expect(TokenKind::Hash, "Expected # in VALUE #(...) row constructor.");
        self.expect(TokenKind::LParen, "Expected ( after VALUE #.");
        let mut rows = Vec::new();

        while !self.at(TokenKind::EOF) && !self.at(TokenKind::RParen) && !self.at(TokenKind::Dot) {
            if !self.match_kind(TokenKind::LParen) {
                self.diagnostics.error(
                    "PAR080",
                    "Expected ( to begin an EML row.",
                    self.current().start,
                    self.current().end,
                );
                self.consume();
                continue;
            }
            let mut fields = Vec::new();
            while !self.at(TokenKind::EOF) && !self.at(TokenKind::RParen) && !self.at(TokenKind::Dot) {
                if self.match_kind(TokenKind::Comma) {
                    continue;
                }
                let name = self.expect_identifier("Expected EML component name.");
                let name_tok = match name {
                    Some(n) => n,
                    None => break,
                };
                self.expect_operator("=", "Expected = after EML component.");
                let value = self.parse_expression(0);
                fields.push(EmlFieldBinding {
                    name: name_tok.lexeme.to_ascii_uppercase(),
                    value,
                });
                if self.at(TokenKind::Comma) {
                    self.consume();
                }
            }
            self.expect(TokenKind::RParen, "Expected ) after EML row.");
            rows.push(fields);
        }
        self.expect(TokenKind::RParen, "Expected ) after EML row table.");
        rows
    }

    fn parse_eml_read(&mut self) -> Statement {
        let start = self.consume();
        self.expect_keyword("ENTITIES", "Expected ENTITIES after READ.");
        self.expect_keyword("OF", "Expected OF after READ ENTITIES.");
        let business_object = self
            .expect_identifier("Expected business object after READ ENTITIES OF.")
            .map(|t| t.lexeme)
            .unwrap_or_default();
        self.expect_keyword("ENTITY", "Expected ENTITY in READ ENTITIES.");
        let entity = self
            .expect_identifier("Expected entity name.")
            .map(|t| t.lexeme)
            .unwrap_or_default();
        self.match_keyword("ALL");
        self.match_keyword("FIELDS");

        let mut filters = Vec::new();
        if self.match_keyword("WITH") || self.match_keyword("FROM") {
            let row_fields = self.parse_eml_rows();
            for rf in row_fields {
                filters.push(EmlFilterRow { fields: rf });
            }
        }
        let result_target = self.parse_eml_data_target("RESULT").unwrap_or_else(|| "LT_RESULT".to_string());
        let mut failed_target = None;
        let mut reported_target = None;

        while !self.at(TokenKind::Dot) && !self.at(TokenKind::EOF) {
            if self.is_keyword("FAILED") {
                failed_target = self.parse_eml_data_target("FAILED");
                continue;
            }
            if self.is_keyword("REPORTED") {
                reported_target = self.parse_eml_data_target("REPORTED");
                continue;
            }
            self.consume();
        }
        let end = self.expect_dot("Expected period after READ ENTITIES.");
        Statement::EmlReadStmt {
            start: start.start,
            end: end.map(|tok| tok.end).unwrap_or(start.end),
            business_object,
            entity,
            filters,
            result_target,
            failed_target,
            reported_target,
        }
    }

    fn parse_eml_modify(&mut self) -> Statement {
        let start = self.consume();
        self.expect_keyword("ENTITIES", "Expected ENTITIES after MODIFY.");
        self.expect_keyword("OF", "Expected OF after MODIFY ENTITIES.");
        let business_object = self
            .expect_identifier("Expected business object after MODIFY ENTITIES OF.")
            .map(|t| t.lexeme)
            .unwrap_or_default();
        self.expect_keyword("ENTITY", "Expected ENTITY in MODIFY ENTITIES.");
        let entity = self
            .expect_identifier("Expected entity name.")
            .map(|t| t.lexeme)
            .unwrap_or_default();

        let mut operation = "CREATE".to_string();
        if self.match_keyword("CREATE") {
            operation = "CREATE".to_string();
        } else if self.match_keyword("UPDATE") {
            operation = "UPDATE".to_string();
        } else if self.match_keyword("DELETE") {
            operation = "DELETE".to_string();
        } else {
            self.diagnostics.error(
                "PAR081",
                "Expected CREATE, UPDATE, or DELETE after EML ENTITY.",
                self.current().start,
                self.current().end,
            );
        }

        if self.match_keyword("FIELDS") && self.match_kind(TokenKind::LParen) {
            while !self.at(TokenKind::EOF) && !self.at(TokenKind::RParen) {
                self.consume();
            }
            self.expect(TokenKind::RParen, "Expected ) after EML FIELDS list.");
        }

        if operation == "DELETE" {
            self.expect_keyword("FROM", "Expected FROM VALUE for EML DELETE.");
        } else {
            self.expect_keyword("WITH", "Expected WITH VALUE in MODIFY ENTITIES.");
        }

        let raw_rows = self.parse_eml_rows();
        let rows = raw_rows.into_iter().map(|rf| EmlModifyRow { fields: rf }).collect();

        let mut mapped_target = None;
        let mut failed_target = None;
        let mut reported_target = None;

        while !self.at(TokenKind::Dot) && !self.at(TokenKind::EOF) {
            if self.is_keyword("MAPPED") {
                mapped_target = self.parse_eml_data_target("MAPPED");
                continue;
            }
            if self.is_keyword("FAILED") {
                failed_target = self.parse_eml_data_target("FAILED");
                continue;
            }
            if self.is_keyword("REPORTED") {
                reported_target = self.parse_eml_data_target("REPORTED");
                continue;
            }
            self.consume();
        }
        let end = self.expect_dot("Expected period after MODIFY ENTITIES.");
        Statement::EmlModifyStmt {
            start: start.start,
            end: end.map(|tok| tok.end).unwrap_or(start.end),
            business_object,
            entity,
            operation,
            rows,
            mapped_target,
            failed_target,
            reported_target,
        }
    }

    fn parse_eml_transaction(&mut self) -> Statement {
        let start = self.consume();
        let operation = start.upper.clone();
        self.expect_keyword("ENTITIES", &format!("Expected ENTITIES after {}.", operation));
        let end = self.expect_dot(&format!("Expected period after {} ENTITIES.", operation));
        Statement::EmlTransactionStmt {
            start: start.start,
            end: end.map(|tok| tok.end).unwrap_or(start.end),
            operation,
        }
    }

    fn parse_call_function(&mut self) -> Statement {
        let s = self.consume();
        self.expect_keyword("FUNCTION", "Expected FUNCTION after CALL.");
        let name = if self.current().kind == TokenKind::String {
            self.consume().lexeme
        } else {
            self.expect_identifier("Expected function name.")
                .map(|t| t.lexeme)
                .unwrap_or_default()
        };
        let mut parameters = BTreeMap::new();
        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
            let key = self.expect_identifier("Expected parameter name.");
            let key_tok = match key {
                Some(k) => k,
                None => break,
            };
            self.expect_operator("=", "Expected = after parameter.");
            parameters.insert(key_tok.lexeme.to_ascii_uppercase(), self.parse_expression(0));
        }
        let e = self.expect_dot("Expected \".\" after CALL FUNCTION.");
        Statement::CallFunction {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            name,
            parameters,
        }
    }

    fn parse_select(&mut self) -> Statement {
        let s = self.consume();
        let single = self.match_keyword("SINGLE");
        let mut fields = Vec::new();

        if !self.match_keyword("FROM") {
            if self.match_kind(TokenKind::Operator) && self.previous().lexeme == "*" {
                fields = vec!["*".to_string()];
            } else {
                while !self.at(TokenKind::EOF) && !self.is_keyword("FROM") {
                    if self.at(TokenKind::Comma) {
                        self.consume();
                        continue;
                    }
                    if self.at(TokenKind::Identifier) || self.at(TokenKind::Keyword) {
                        fields.push(self.consume().lexeme);
                    } else {
                        break;
                    }
                }
                self.expect_keyword("FROM", "Expected FROM in SELECT.");
            }
        }

        let mut source = String::new();
        if self.at(TokenKind::Identifier) || self.at(TokenKind::Keyword) {
            source = self.consume().lexeme;
        } else {
            self.diagnostics.error(
                "PAR030",
                "Expected data source after FROM.",
                self.current().start,
                self.current().end,
            );
        }

        if fields.is_empty() && self.match_keyword("FIELDS") {
            while !self.at(TokenKind::EOF) && !self.is_keyword("INTO") && !self.is_keyword("WHERE") {
                if self.at(TokenKind::Comma) {
                    self.consume();
                    continue;
                }
                if self.at(TokenKind::Operator) && self.current().lexeme == "*" {
                    self.consume();
                    fields = vec!["*".to_string()];
                    continue;
                }
                if self.at(TokenKind::Identifier) || self.at(TokenKind::Keyword) {
                    fields.push(self.consume().lexeme);
                } else {
                    break;
                }
            }
        }

        let mut where_clause = None;
        if self.match_keyword("WHERE") {
            where_clause = Some(self.parse_expression(0));
        }

        let mut target = None;
        if self.match_keyword("INTO") {
            self.match_keyword("TABLE");
            self.match_kind(TokenKind::At);
            if self.match_keyword("DATA") {
                self.match_kind(TokenKind::LParen);
                target = self.expect_identifier("Expected inline target.").map(|t| t.lexeme);
                self.match_kind(TokenKind::RParen);
            } else {
                target = self.expect_identifier("Expected target after INTO.").map(|t| t.lexeme);
            }
        }
        let e = self.expect_dot("Expected \".\" after SELECT.");
        Statement::SelectStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            fields,
            source,
            target,
            single,
            where_clause,
        }
    }

    fn parse_try(&mut self) -> Statement {
        let s = self.consume();
        self.expect_dot("Expected \".\" after TRY.");
        let mut end_set = HashSet::new();
        end_set.insert("CATCH".to_string());
        end_set.insert("ENDTRY".to_string());
        let body = self.parse_block(&end_set);
        let mut catches = Vec::new();

        while self.match_keyword("CATCH") {
            let catch_type = self
                .expect_identifier("Expected exception type.")
                .map(|t| t.lexeme)
                .unwrap_or_else(|| "CX_ROOT".to_string());
            let cb = self.parse_block(&end_set);
            catches.push(TryCatch {
                catch_type,
                variable: None,
                body: cb,
            });
        }
        self.expect_keyword("ENDTRY", "Expected ENDTRY.");
        let e = self.expect_dot("Expected \".\" after ENDTRY.");
        Statement::TryStmt {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            body,
            catches,
        }
    }

    fn parse_class(&mut self) -> ClassDecl {
        let s = self.consume();
        let name = self
            .expect_identifier("Expected class name.")
            .map(|t| t.lexeme)
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let mut implementation = false;
        let mut super_class = None;
        let mut is_final = false;
        let mut is_abstract = false;

        if self.match_keyword("IMPLEMENTATION") {
            implementation = true;
        } else {
            self.match_keyword("DEFINITION");
            while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
                if self.match_keyword("INHERITING") {
                    self.expect_keyword("FROM", "Expected FROM after INHERITING.");
                    super_class = self.expect_identifier("Expected superclass name.").map(|t| t.lexeme);
                    continue;
                }
                if self.match_keyword("FINAL") {
                    is_final = true;
                    continue;
                }
                if self.match_keyword("ABSTRACT") {
                    is_abstract = true;
                    continue;
                }
                self.consume();
            }
        }
        if self.at(TokenKind::Dot) {
            self.consume();
        }

        let mut methods = Vec::new();
        let mut interfaces = Vec::new();
        let mut class_types = Vec::new();
        let mut attributes = Vec::new();
        let mut visibility = Visibility::Private;

        while !self.at(TokenKind::EOF) && !self.is_keyword("ENDCLASS") {
            if self.is_keyword("PUBLIC") || self.is_keyword("PROTECTED") || self.is_keyword("PRIVATE") {
                let v = self.consume().upper.to_ascii_lowercase();
                visibility = match v.as_str() {
                    "public" => Visibility::Public,
                    "protected" => Visibility::Protected,
                    _ => Visibility::Private,
                };
                self.match_keyword("SECTION");
                self.expect_dot("Expected period after visibility section.");
                continue;
            }
            if self.is_keyword("TYPES") {
                if let Some(parsed) = self.parse_types() {
                    for item in parsed {
                        if let Statement::TypeDecl(td) = item {
                            class_types.push(td);
                        }
                    }
                }
                continue;
            }
            if self.match_keyword("INTERFACES") {
                if let Some(iface) = self.expect_identifier("Expected interface name.") {
                    interfaces.push(iface.lexeme);
                }
                if self.at(TokenKind::Dot) {
                    self.consume();
                } else {
                    self.expect_dot("Expected \".\" after INTERFACES.");
                }
                continue;
            }
            if self.is_keyword("DATA") || self.is_keyword("CLASS-DATA") || self.is_keyword("CONSTANTS") {
                let decl = self.consume();
                let is_static = decl.upper == "CLASS-DATA";
                let constant = decl.upper == "CONSTANTS";
                self.match_kind(TokenKind::Colon);

                while !self.at(TokenKind::EOF) {
                    let n = self.expect_identifier("Expected class attribute name.");
                    let n_tok = match n {
                        Some(tok) => tok,
                        None => {
                            self.recover_to_comma_or_dot();
                            break;
                        }
                    };
                    self.expect_keyword("TYPE", "Expected TYPE in class attribute declaration.");
                    let type_ref = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
                    let mut value = None;
                    while !self.at(TokenKind::EOF) && !self.at(TokenKind::Comma) && !self.at(TokenKind::Dot) {
                        if self.match_keyword("VALUE") {
                            self.expect_operator("=", "Expected = after VALUE.");
                            value = Some(self.parse_expression(0));
                            continue;
                        }
                        if self.match_keyword("READ-ONLY") {
                            continue;
                        }
                        self.consume();
                    }
                    let end_pos = value.as_ref().map(|v| v.end_pos()).unwrap_or(n_tok.end);
                    attributes.push(ClassAttributeDecl {
                        start: decl.start,
                        end: end_pos,
                        name: n_tok.lexeme,
                        type_ref,
                        is_static,
                        visibility,
                        constant,
                        value,
                    });
                    if self.at(TokenKind::Dot) {
                        self.consume();
                        break;
                    }
                    if !self.match_kind(TokenKind::Comma) {
                        self.diagnostics.error(
                            "PAR067",
                            "Expected comma or period in class attribute declaration.",
                            self.current().start,
                            self.current().end,
                        );
                        self.recover_statement();
                        break;
                    }
                }
                continue;
            }
            if self.is_keyword("METHODS") || self.is_keyword("CLASS-METHODS") {
                let method_kw = self.consume();
                let is_static = method_kw.upper == "CLASS-METHODS";
                self.match_kind(TokenKind::Colon);

                while !self.at(TokenKind::Dot) && !self.at(TokenKind::EOF) {
                    let m = self.parse_method_name();
                    let m_info = match m {
                        Some(info) => info,
                        None => {
                            self.recover_to_comma_or_dot();
                            if self.at(TokenKind::Dot) {
                                break;
                            }
                            continue;
                        }
                    };
                    let (params, returning) = self.parse_method_signature();
                    let mut abstract_method = false;
                    let mut method_final = false;
                    let mut redefinition = false;

                    while !self.at(TokenKind::Comma) && !self.at(TokenKind::Dot) && !self.at(TokenKind::EOF) {
                        if self.match_keyword("ABSTRACT") {
                            abstract_method = true;
                            continue;
                        }
                        if self.match_keyword("FINAL") {
                            method_final = true;
                            continue;
                        }
                        if self.match_keyword("REDEFINITION") {
                            redefinition = true;
                            continue;
                        }
                        self.consume();
                    }
                    methods.push(MethodDecl {
                        start: m_info.start,
                        end: self.previous().end,
                        name: m_info.name,
                        statements: Vec::new(),
                        params,
                        returning,
                        implemented: Some(false),
                        is_static: Some(is_static),
                        visibility: Some(visibility),
                        is_abstract: Some(abstract_method),
                        is_final: Some(method_final),
                        redefinition: Some(redefinition),
                    });
                    if self.at(TokenKind::Comma) {
                        self.consume();
                        continue;
                    }
                    if !self.at(TokenKind::Dot) {
                        self.diagnostics.error(
                            "PAR060",
                            "Expected comma or period after method declaration.",
                            self.current().start,
                            self.current().end,
                        );
                        self.recover_to_comma_or_dot();
                        if self.at(TokenKind::Comma) {
                            self.consume();
                        } else {
                            break;
                        }
                    }
                }
                self.match_kind(TokenKind::Dot);
                continue;
            }
            if self.match_keyword("METHOD") {
                let m = self.parse_method_name();
                self.expect_dot("Expected \".\" after METHOD.");
                let mut end_set = HashSet::new();
                end_set.insert("ENDMETHOD".to_string());
                let body = self.parse_block(&end_set);
                self.expect_keyword("ENDMETHOD", "Expected ENDMETHOD.");
                let e = self.expect_dot("Expected \".\" after ENDMETHOD.");
                if let Some(info) = m {
                    methods.push(MethodDecl {
                        start: info.start,
                        end: e.map(|tok| tok.end).unwrap_or(info.end),
                        name: info.name,
                        statements: body,
                        params: Vec::new(),
                        returning: None,
                        implemented: Some(true),
                        is_static: None,
                        visibility: Some(visibility),
                        is_abstract: None,
                        is_final: None,
                        redefinition: None,
                    });
                }
                continue;
            }
            if self.match_keyword("CLASS_CONSTRUCTOR") {
                let m = self.previous().clone();
                self.expect_dot("Expected period after CLASS_CONSTRUCTOR.");
                let mut end_set = HashSet::new();
                end_set.insert("ENDMETHOD".to_string());
                let body = self.parse_block(&end_set);
                self.expect_keyword("ENDMETHOD", "Expected ENDMETHOD after CLASS_CONSTRUCTOR.");
                let e = self.expect_dot("Expected period after class constructor.");
                methods.push(MethodDecl {
                    start: m.start,
                    end: e.map(|tok| tok.end).unwrap_or(m.end),
                    name: "CLASS_CONSTRUCTOR".to_string(),
                    statements: body,
                    params: Vec::new(),
                    returning: None,
                    implemented: Some(true),
                    is_static: Some(true),
                    visibility: Some(Visibility::Private),
                    is_abstract: None,
                    is_final: None,
                    redefinition: None,
                });
                continue;
            }
            if self.is_keyword("FINAL")
                || self.is_keyword("CREATE")
                || self.is_keyword("ABSTRACT")
                || self.is_keyword("FRIENDS")
            {
                self.consume();
                continue;
            }
            self.consume();
        }

        self.expect_keyword("ENDCLASS", "Expected ENDCLASS.");
        let e = self.expect_dot("Expected \".\" after ENDCLASS.");
        let c = ClassDecl {
            start: s.start,
            end: e.map(|tok| tok.end).unwrap_or(s.end),
            name,
            methods,
            attributes,
            implementation,
            interfaces,
            types: class_types,
            super_class,
            is_final: Some(is_final),
            is_abstract: Some(is_abstract),
        };
        self.classes.push(c.clone());
        c
    }

    fn parse_method_signature(&mut self) -> (Vec<MethodParameter>, Option<ReturningParam>) {
        let mut params = Vec::new();
        let mut returning = None;
        let mut direction = "importing".to_string();

        let stop_keywords: HashSet<&str> = ["ABSTRACT", "FINAL", "REDEFINITION"].into_iter().collect();

        while !self.at(TokenKind::EOF)
            && !self.at(TokenKind::Dot)
            && !self.at(TokenKind::Comma)
            && !stop_keywords.contains(self.current().upper.as_str())
        {
            if self.match_keyword("IMPORTING") {
                direction = "importing".to_string();
                continue;
            }
            if self.match_keyword("EXPORTING") {
                direction = "exporting".to_string();
                continue;
            }
            if self.match_keyword("CHANGING") {
                direction = "changing".to_string();
                continue;
            }
            if self.match_keyword("RETURNING") {
                let mut by_val_name = None;
                if self.match_keyword("VALUE") {
                    self.expect(TokenKind::LParen, "Expected ( after VALUE in RETURNING.");
                    by_val_name = self.expect_identifier("Expected returning parameter name.").map(|t| t.lexeme);
                    self.expect(TokenKind::RParen, "Expected ) after returning parameter name.");
                } else {
                    by_val_name = self.expect_identifier("Expected returning parameter name.").map(|t| t.lexeme);
                }
                self.expect_keyword("TYPE", "Expected TYPE for returning parameter.");
                let ty = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
                if let Some(bvn) = by_val_name {
                    returning = Some(ReturningParam { name: bvn, type_ref: ty });
                }
                continue;
            }

            let mut param_name = None;
            let mut pass_by_value = false;
            if self.match_keyword("VALUE") {
                pass_by_value = true;
                self.expect(TokenKind::LParen, "Expected ( after VALUE.");
                param_name = self.expect_identifier("Expected parameter name.").map(|t| t.lexeme);
                self.expect(TokenKind::RParen, "Expected ) after parameter name.");
            } else {
                param_name = self.expect_identifier("Expected formal parameter name.").map(|t| t.lexeme);
            }

            let pname = match param_name {
                Some(p) => p,
                None => {
                    self.consume();
                    continue;
                }
            };

            let mut ty = TypeRef::scalar("any");
            if self.match_keyword("TYPE") {
                ty = self.parse_type().unwrap_or_else(|| TypeRef::scalar("any"));
            } else if self.match_keyword("LIKE") {
                let n = self.expect_identifier("Expected data object after LIKE.");
                ty = TypeRef::Named {
                    name: n.map(|t| t.lexeme).unwrap_or_else(|| "any".to_string()),
                };
            } else {
                self.diagnostics.error(
                    "PAR061",
                    format!("Expected TYPE for formal parameter '{}'.", pname),
                    self.current().start,
                    self.current().end,
                );
            }

            let mut optional = false;
            let mut default_value = None;
            let keywords_dir: HashSet<&str> =
                ["IMPORTING", "EXPORTING", "CHANGING", "RETURNING", "ABSTRACT", "FINAL", "REDEFINITION"]
                    .into_iter()
                    .collect();

            while !self.at(TokenKind::EOF)
                && !self.at(TokenKind::Comma)
                && !self.at(TokenKind::Dot)
                && !keywords_dir.contains(self.current().upper.as_str())
            {
                if self.match_keyword("OPTIONAL") {
                    optional = true;
                    continue;
                }
                if self.match_keyword("DEFAULT") {
                    default_value = Some(self.parse_expression(4));
                    continue;
                }
                if self.match_keyword("VALUE") {
                    pass_by_value = true;
                    continue;
                }
                break;
            }
            params.push(MethodParameter {
                name: pname,
                type_ref: ty,
                direction: direction.clone(),
                optional: Some(optional),
                default_value,
                pass_by_value: Some(pass_by_value),
            });
        }
        (params, returning)
    }

    fn parse_method_name(&mut self) -> Option<MethodNameInfo> {
        let first = self.expect_identifier("Expected method name.")?;
        let mut name = first.lexeme;
        let mut end = first.end;
        if self.at(TokenKind::Tilde) {
            self.consume();
            if let Some(second) = self.expect_identifier("Expected interface method name after ~.") {
                name = format!("{}~{}", name, second.lexeme);
                end = second.end;
            }
        }
        Some(MethodNameInfo {
            name,
            start: first.start,
            end,
        })
    }

    fn parse_method_call(&mut self) -> Statement {
        let start = self.current().start;
        let receiver = self.consume().lexeme;
        let selector = self.current().lexeme.clone();
        self.expect(TokenKind::Arrow, "Expected -> or => for method call.");
        let method = self
            .parse_method_name()
            .map(|m| m.name)
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let actuals = self.parse_method_actual_arguments();
        let end = self.expect_dot("Expected \".\" after method call.");

        Statement::MethodCallStmt {
            start,
            end: end.map(|tok| tok.end).unwrap_or(self.previous().end),
            receiver,
            method,
            args: actuals.args,
            named: actuals.named,
            output_named: if !actuals.output_named.is_empty() {
                Some(actuals.output_named)
            } else {
                None
            },
            changing_named: if !actuals.changing_named.is_empty() {
                Some(actuals.changing_named)
            } else {
                None
            },
            static_call: Some(selector == "=>"),
        }
    }

    fn parse_method_actual_arguments(&mut self) -> MethodActuals {
        self.expect(TokenKind::LParen, "Expected ( after method name.");
        let mut args = Vec::new();
        let mut named = BTreeMap::new();
        let mut output_named = BTreeMap::new();
        let mut changing_named = BTreeMap::new();
        let mut section = "exporting";

        while !self.at(TokenKind::EOF) && !self.at(TokenKind::RParen) {
            if self.at(TokenKind::Comma) {
                self.consume();
                continue;
            }
            if self.match_keyword("EXPORTING") {
                section = "exporting";
                continue;
            }
            if self.match_keyword("IMPORTING") {
                section = "importing";
                continue;
            }
            if self.match_keyword("CHANGING") {
                section = "changing";
                continue;
            }

            if (self.at(TokenKind::Identifier) || self.at(TokenKind::Keyword))
                && self.peek(1).kind == TokenKind::Operator
                && self.peek(1).lexeme == "="
            {
                let n = self.consume().lexeme.to_ascii_uppercase();
                self.consume();
                let actual = self.parse_expression(0);
                if section == "importing" {
                    output_named.insert(n, actual);
                } else if section == "changing" {
                    changing_named.insert(n, actual);
                } else {
                    named.insert(n, actual);
                }
            } else if section == "exporting" {
                args.push(self.parse_expression(0));
            } else {
                self.diagnostics.error(
                    "PAR063",
                    format!("Named actual parameter expected in {} section.", section.to_ascii_uppercase()),
                    self.current().start,
                    self.current().end,
                );
                self.parse_expression(0);
            }

            if self.at(TokenKind::Comma) {
                self.consume();
            } else if !self.at(TokenKind::RParen)
                && !self.is_keyword("EXPORTING")
                && !self.is_keyword("IMPORTING")
                && !self.is_keyword("CHANGING")
                && !((self.at(TokenKind::Identifier) || self.at(TokenKind::Keyword))
                    && self.peek(1).kind == TokenKind::Operator
                    && self.peek(1).lexeme == "=")
            {
                self.diagnostics.error(
                    "PAR062",
                    "Expected another named argument, section keyword, comma, or ) in method call.",
                    self.current().start,
                    self.current().end,
                );
                break;
            }
        }
        self.expect(TokenKind::RParen, "Expected ) after method arguments.");
        MethodActuals {
            args,
            named,
            output_named,
            changing_named,
        }
    }

    fn parse_assignment(&mut self) -> Statement {
        let start = self.current().start;
        let target = self.parse_postfix();
        self.expect_operator("=", "Expected = in assignment.");
        let value = self.parse_expression(0);
        let e = self.expect_dot("Expected \".\" after assignment.");
        let val_end = value.end_pos();
        Statement::Assignment {
            start,
            end: e.map(|tok| tok.end).unwrap_or(val_end),
            target,
            value,
        }
    }

    fn parse_expression(&mut self, min_prec: i32) -> Expr {
        let mut left = self.parse_prefix();
        while !self.at(TokenKind::EOF) {
            let op = self.current().upper.clone();
            let prec = PREC.get(op.as_str()).copied().unwrap_or(-1);
            if prec < min_prec {
                break;
            }
            let token = self.consume();
            let right = self.parse_expression(prec + 1);
            let left_start = left.start_pos();
            let right_end = right.end_pos();
            left = Expr::Binary {
                start: left_start,
                end: right_end,
                op: token.upper,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_prefix(&mut self) -> Expr {
        let t = self.current().clone();
        if t.upper == "NOT" || t.lexeme == "-" {
            self.consume();
            let operand = self.parse_expression(6);
            let op_end = operand.end_pos();
            return Expr::Unary {
                start: t.start,
                end: op_end,
                op: t.upper,
                operand: Box::new(operand),
            };
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut base = self.parse_primary();
        loop {
            if self.match_kind(TokenKind::LBracket) {
                let idx = self.parse_expression(0);
                let e = self.expect(TokenKind::RBracket, "Expected ] after index.");
                let b_start = base.start_pos();
                let end_pos = e.map(|tok| tok.end).unwrap_or_else(|| idx.end_pos());
                base = Expr::Index {
                    start: b_start,
                    end: end_pos,
                    base: Box::new(base),
                    index: Box::new(idx),
                };
                continue;
            }
            if self.current().lexeme == "-" && self.current().start.offset == base.end_pos().offset {
                self.consume();
                if let Some(f) = self.expect_identifier("Expected component name after -.") {
                    let b_start = base.start_pos();
                    base = Expr::Field {
                        start: b_start,
                        end: f.end,
                        base: Box::new(base),
                        field: f.lexeme,
                        static_selector: None,
                    };
                }
                continue;
            }
            if self.current().kind == TokenKind::Arrow {
                let selector = self.current().lexeme.clone();
                self.consume();
                let f = self.expect_identifier("Expected component or method name after ->/=>.");
                let f_tok = match f {
                    Some(tok) => tok,
                    None => break,
                };
                let b_start = base.start_pos();
                if self.at(TokenKind::LParen) {
                    let actuals = self.parse_method_actual_arguments();
                    base = Expr::MethodCallExpr {
                        start: b_start,
                        end: self.previous().end,
                        receiver: Box::new(base),
                        method: f_tok.lexeme,
                        args: actuals.args,
                        named: actuals.named,
                        output_named: if !actuals.output_named.is_empty() {
                            Some(actuals.output_named)
                        } else {
                            None
                        },
                        changing_named: if !actuals.changing_named.is_empty() {
                            Some(actuals.changing_named)
                        } else {
                            None
                        },
                        static_call: Some(selector == "=>"),
                    };
                } else {
                    base = Expr::Field {
                        start: b_start,
                        end: f_tok.end,
                        base: Box::new(base),
                        field: f_tok.lexeme,
                        static_selector: Some(selector == "=>"),
                    };
                }
                continue;
            }
            break;
        }
        base
    }

    fn parse_switch_expression(&mut self) -> Expr {
        let start = self.consume();
        let mut result_type = None;
        let mut infer_type = false;

        if self.match_kind(TokenKind::Hash) {
            infer_type = true;
        } else if let Some(type_token) = self.expect_identifier("Expected a result type or # after SWITCH.") {
            let u = type_token.upper.as_str();
            let sname = match u {
                "I" | "INT4" => Some("i"),
                "INT8" => Some("int8"),
                "DECFLOAT16" => Some("decfloat16"),
                "DECFLOAT34" => Some("decfloat34"),
                "STRING" => Some("string"),
                "XSTRING" => Some("xstring"),
                "C" | "CHAR" => Some("c"),
                "N" | "NUMC" => Some("n"),
                "D" | "DATS" => Some("d"),
                "T" | "TIMS" => Some("t"),
                "ABAP_BOOL" => Some("abap_bool"),
                _ => None,
            };
            result_type = Some(if let Some(s) = sname {
                TypeRef::scalar(s)
            } else {
                TypeRef::Named {
                    name: type_token.lexeme,
                }
            });
        }

        self.expect(TokenKind::LParen, "Expected ( after SWITCH result type.");
        let operand = self.parse_expression(0);
        let mut branches = Vec::new();

        while self.match_keyword("WHEN") {
            let mut values = vec![self.parse_expression(2)];
            while self.match_keyword("OR") {
                values.push(self.parse_expression(2));
            }
            self.expect_keyword("THEN", "Expected THEN after SWITCH WHEN value.");
            let result = self.parse_expression(0);
            branches.push(SwitchBranch { values, result });
        }

        let mut else_result = None;
        if self.match_keyword("ELSE") {
            else_result = Some(Box::new(self.parse_expression(0)));
        } else if self.match_keyword("OTHERS") {
            self.diagnostics.error(
                "PAR072",
                "Use ELSE in a SWITCH expression; OTHERS is used with the CASE statement.",
                self.previous().start,
                self.previous().end,
            );
            self.expect_keyword("THEN", "Expected THEN after OTHERS.");
            else_result = Some(Box::new(self.parse_expression(0)));
        }

        let end = self.expect(TokenKind::RParen, "Expected ) after SWITCH expression.");
        let end_pos = end.map(|tok| tok.end).unwrap_or(self.previous().end);

        if branches.is_empty() {
            self.diagnostics.error(
                "PAR071",
                "SWITCH requires at least one WHEN branch.",
                start.start,
                end_pos,
            );
        }

        Expr::Switch {
            start: start.start,
            end: end_pos,
            result_type,
            infer_type,
            operand: Box::new(operand),
            branches,
            else_result,
        }
    }

    fn parse_primary(&mut self) -> Expr {
        let t = self.current().clone();
        if t.kind == TokenKind::Integer {
            self.consume();
            let val = t.lexeme.parse::<i64>().unwrap_or(0);
            return Expr::Literal {
                start: t.start,
                end: t.end,
                value: serde_json::Value::Number(val.into()),
                type_ref: TypeRef::scalar("i"),
            };
        }
        if t.kind == TokenKind::Decimal {
            self.consume();
            let val = t.lexeme.parse::<f64>().unwrap_or(0.0);
            let num = serde_json::Number::from_f64(val).unwrap_or_else(|| serde_json::Number::from(0));
            return Expr::Literal {
                start: t.start,
                end: t.end,
                value: serde_json::Value::Number(num),
                type_ref: TypeRef::scalar("decfloat34"),
            };
        }
        if t.kind == TokenKind::String {
            self.consume();
            return Expr::Literal {
                start: t.start,
                end: t.end,
                value: serde_json::Value::String(t.lexeme),
                type_ref: TypeRef::scalar("string"),
            };
        }
        if t.upper == "ABAP_TRUE" || t.upper == "TRUE" {
            self.consume();
            return Expr::Literal {
                start: t.start,
                end: t.end,
                value: serde_json::Value::Bool(true),
                type_ref: TypeRef::scalar("abap_bool"),
            };
        }
        if t.upper == "ABAP_FALSE" || t.upper == "FALSE" {
            self.consume();
            return Expr::Literal {
                start: t.start,
                end: t.end,
                value: serde_json::Value::Bool(false),
                type_ref: TypeRef::scalar("abap_bool"),
            };
        }
        if t.upper == "SPACE" {
            self.consume();
            return Expr::Literal {
                start: t.start,
                end: t.end,
                value: serde_json::Value::String(" ".to_string()),
                type_ref: TypeRef::scalar("string"),
            };
        }
        if t.upper == "NEW" {
            let s = self.consume();
            let cls = self
                .expect_identifier("Expected class name after NEW.")
                .map(|tok| tok.lexeme)
                .unwrap_or_else(|| "UNKNOWN".to_string());
            let actuals = if self.at(TokenKind::LParen) {
                self.parse_method_actual_arguments()
            } else {
                MethodActuals::default()
            };
            return Expr::NewObject {
                start: s.start,
                end: self.previous().end,
                class_name: cls,
                args: actuals.args,
                named: actuals.named,
            };
        }
        if t.kind == TokenKind::LParen {
            self.consume();
            let e = self.parse_expression(0);
            self.expect(TokenKind::RParen, "Expected )");
            return e;
        }
        if t.upper == "VALUE" {
            let s = self.consume();
            if self.at(TokenKind::Hash) {
                self.consume();
            }
            self.match_kind(TokenKind::LParen);
            let mut assignments = Vec::new();
            while !self.at(TokenKind::EOF) && !self.at(TokenKind::RParen) {
                let n = self.expect_identifier("Expected component in VALUE #(...).");
                let n_tok = match n {
                    Some(tok) => tok,
                    None => break,
                };
                self.expect_operator("=", "Expected = in VALUE constructor.");
                let v = self.parse_expression(4);
                assignments.push(ConstructorAssignment {
                    name: n_tok.lexeme,
                    value: v,
                });
                if self.at(TokenKind::Comma) {
                    self.consume();
                }
            }
            let e = self.expect(TokenKind::RParen, "Expected ) after VALUE constructor.");
            return Expr::Constructor {
                start: s.start,
                end: e.map(|tok| tok.end).unwrap_or(s.end),
                assignments,
            };
        }
        if t.upper == "SWITCH" {
            return self.parse_switch_expression();
        }
        if t.kind == TokenKind::Identifier || t.kind == TokenKind::Keyword {
            let name = self.consume();
            if self.at(TokenKind::LParen) {
                self.consume();
                let mut args = Vec::new();
                while !self.at(TokenKind::EOF) && !self.at(TokenKind::RParen) {
                    if (self.at(TokenKind::Identifier) || self.at(TokenKind::Keyword))
                        && self.peek(1).kind == TokenKind::Operator
                        && self.peek(1).lexeme == "="
                    {
                        self.consume();
                        self.consume();
                        args.push(self.parse_expression(0));
                    } else {
                        args.push(self.parse_expression(0));
                    }
                    if self.at(TokenKind::Comma) {
                        self.consume();
                    } else if !self.at(TokenKind::RParen) {
                        break;
                    }
                }
                let e = self.expect(TokenKind::RParen, "Expected ) after function call.");
                return Expr::Call {
                    start: name.start,
                    end: e.map(|tok| tok.end).unwrap_or(name.end),
                    name: name.lexeme,
                    args,
                };
            }
            return Expr::Variable {
                start: name.start,
                end: name.end,
                name: name.lexeme,
            };
        }

        self.diagnostics.error(
            "PAR040",
            format!(
                "Expected expression, got '{}'.",
                if !t.lexeme.is_empty() { &t.lexeme } else { &t.upper }
            ),
            t.start,
            t.end,
        );
        self.consume();
        Expr::Literal {
            start: t.start,
            end: t.end,
            value: serde_json::Value::Number(0.into()),
            type_ref: TypeRef::scalar("i"),
        }
    }

    fn parse_block(&mut self, end_words: &HashSet<String>) -> Vec<Statement> {
        let mut out = Vec::new();
        while !self.at(TokenKind::EOF) && !self.is_word_in(end_words) && !self.is_event_header_ahead() {
            if let Some(ss) = self.parse_statement(end_words) {
                out.extend(ss);
            } else if !self.is_word_in(end_words) && !self.at(TokenKind::EOF) {
                self.recover_statement();
            }
        }
        out
    }

    fn collect_until_dot(&mut self) {
        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
            self.consume();
        }
        self.match_kind(TokenKind::Dot);
    }

    fn recover_statement(&mut self) {
        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Dot) {
            self.consume();
        }
        self.match_kind(TokenKind::Dot);
    }

    fn recover_to_comma_or_dot(&mut self) {
        while !self.at(TokenKind::EOF) && !self.at(TokenKind::Comma) && !self.at(TokenKind::Dot) {
            self.consume();
        }
        if self.at(TokenKind::Comma) {
            self.consume();
        }
    }

    fn is_word_in(&self, set: &HashSet<String>) -> bool {
        set.contains(&self.current().upper)
    }

    fn is_keyword(&self, word: &str) -> bool {
        self.current().upper == word
    }

    fn is_next_keyword(&self, word: &str) -> bool {
        self.peek(1).upper == word
    }

    fn current(&self) -> &Token {
        &self.tokens[self.i.min(self.tokens.len() - 1)]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.i.saturating_sub(1).min(self.tokens.len() - 1)]
    }

    fn peek(&self, n: usize) -> &Token {
        &self.tokens[(self.i + n).min(self.tokens.len() - 1)]
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    fn consume(&mut self) -> Token {
        let t = self.current().clone();
        if self.i < self.tokens.len() {
            self.i += 1;
        }
        t
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn match_keyword(&mut self, word: &str) -> bool {
        if self.is_keyword(word) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) -> Option<Token> {
        if self.at(kind) {
            Some(self.consume())
        } else {
            self.diagnostics.error("PAR999", msg, self.current().start, self.current().end);
            None
        }
    }

    fn expect_identifier(&mut self, msg: &str) -> Option<Token> {
        if self.current().kind == TokenKind::Identifier || self.current().kind == TokenKind::Keyword {
            Some(self.consume())
        } else {
            self.diagnostics.error("PAR999", msg, self.current().start, self.current().end);
            None
        }
    }

    fn expect_keyword(&mut self, word: &str, msg: &str) -> Option<Token> {
        if self.match_keyword(word) {
            Some(self.previous().clone())
        } else {
            self.diagnostics.error("PAR999", msg, self.current().start, self.current().end);
            None
        }
    }

    fn expect_operator(&mut self, op: &str, msg: &str) -> Option<Token> {
        if self.current().kind == TokenKind::Operator && self.current().lexeme == op {
            Some(self.consume())
        } else {
            self.diagnostics.error("PAR999", msg, self.current().start, self.current().end);
            None
        }
    }

    fn expect_dot(&mut self, msg: &str) -> Option<Token> {
        self.expect(TokenKind::Dot, msg)
    }
}

pub struct MethodNameInfo {
    pub name: String,
    pub start: Position,
    pub end: Position,
}

#[derive(Default)]
pub struct MethodActuals {
    pub args: Vec<Expr>,
    pub named: BTreeMap<String, Expr>,
    pub output_named: BTreeMap<String, Expr>,
    pub changing_named: BTreeMap<String, Expr>,
}

impl Expr {
    pub fn start_pos(&self) -> Position {
        match self {
            Expr::Literal { start, .. }
            | Expr::Variable { start, .. }
            | Expr::Field { start, .. }
            | Expr::Unary { start, .. }
            | Expr::Binary { start, .. }
            | Expr::Call { start, .. }
            | Expr::Index { start, .. }
            | Expr::Constructor { start, .. }
            | Expr::NewObject { start, .. }
            | Expr::MethodCallExpr { start, .. }
            | Expr::Switch { start, .. } => *start,
        }
    }

    pub fn end_pos(&self) -> Position {
        match self {
            Expr::Literal { end, .. }
            | Expr::Variable { end, .. }
            | Expr::Field { end, .. }
            | Expr::Unary { end, .. }
            | Expr::Binary { end, .. }
            | Expr::Call { end, .. }
            | Expr::Index { end, .. }
            | Expr::Constructor { end, .. }
            | Expr::NewObject { end, .. }
            | Expr::MethodCallExpr { end, .. }
            | Expr::Switch { end, .. } => *end,
        }
    }
}

impl Statement {
    pub fn end_pos(&self) -> Position {
        match self {
            Statement::DataDecl { end, .. }
            | Statement::InlineDataAssign { end, .. }
            | Statement::ConstantDecl { end, .. }
            | Statement::Assignment { end, .. }
            | Statement::WriteStmt { end, .. }
            | Statement::IfStmt { end, .. }
            | Statement::WhileStmt { end, .. }
            | Statement::DoStmt { end, .. }
            | Statement::CaseStmt { end, .. }
            | Statement::LoopStmt { end, .. }
            | Statement::AppendStmt { end, .. }
            | Statement::InsertStmt { end, .. }
            | Statement::ReadTableStmt { end, .. }
            | Statement::DeleteTableStmt { end, .. }
            | Statement::SortStmt { end, .. }
            | Statement::DescribeTableStmt { end, .. }
            | Statement::ClearStmt { end, .. }
            | Statement::TranslateStmt { end, .. }
            | Statement::ConcatenateStmt { end, .. }
            | Statement::SplitStmt { end, .. }
            | Statement::CondenseStmt { end, .. }
            | Statement::MessageStmt { end, .. }
            | Statement::EventBlock { end, .. }
            | Statement::FlowStmt { end, .. }
            | Statement::AssertStmt { end, .. }
            | Statement::CallFunction { end, .. }
            | Statement::MethodCallStmt { end, .. }
            | Statement::EmlReadStmt { end, .. }
            | Statement::EmlModifyStmt { end, .. }
            | Statement::EmlTransactionStmt { end, .. }
            | Statement::SelectStmt { end, .. }
            | Statement::TryStmt { end, .. }
            | Statement::EmptyStmt { end, .. } => *end,
            Statement::ClassDecl(c) => c.end,
            Statement::TypeDecl(t) => t.end,
        }
    }
}
