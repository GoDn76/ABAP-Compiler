use super::ast::{Program, Statement};
use super::diagnostics::DiagnosticBag;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageMode {
    Standard,
    Cloud,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleasedApiManifest {
    #[serde(default)]
    pub functions: Vec<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub interfaces: Vec<String>,
    #[serde(default)]
    pub tables: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompilerPolicy {
    pub mode: LanguageMode,
    pub released_apis: ReleasedApiManifest,
    pub strict_cloud_syntax: bool,
}

impl Default for CompilerPolicy {
    fn default() -> Self {
        Self {
            mode: LanguageMode::Standard,
            released_apis: ReleasedApiManifest::default(),
            strict_cloud_syntax: true,
        }
    }
}

pub fn validate_policy(program: &Program, policy: &CompilerPolicy, d: &mut DiagnosticBag) {
    if policy.mode == LanguageMode::Standard {
        return;
    }

    if program.report_name.is_some() {
        d.error(
            "CLD001",
            "REPORT programs are not accepted by the ABAP Cloud profile in ABAPC; use IF_OO_ADT_CLASSRUN~MAIN as the executable Cloud entry point.",
            program.start,
            program.end,
        );
    }

    let classes = &program.classes;
    let has_interface = classes.iter().any(|c| {
        c.interfaces
            .iter()
            .any(|i| i.eq_ignore_ascii_case("IF_OO_ADT_CLASSRUN"))
    });
    let entry = classes.iter().find(|c| {
        c.implementation
            && c.methods
                .iter()
                .any(|m| m.name.eq_ignore_ascii_case("IF_OO_ADT_CLASSRUN~MAIN"))
    });

    if !has_interface {
        d.error(
            "CLD003",
            "ABAP Cloud executable class must implement IF_OO_ADT_CLASSRUN.",
            program.start,
            program.end,
        );
    }
    if entry.is_none() {
        d.error(
            "CLD004",
            "ABAP Cloud executable class must implement IF_OO_ADT_CLASSRUN~MAIN.",
            program.start,
            program.end,
        );
    }
    if let Some(e) = entry {
        let implements = classes.iter().any(|c| {
            c.name.eq_ignore_ascii_case(&e.name)
                && c.interfaces
                    .iter()
                    .any(|i| i.eq_ignore_ascii_case("IF_OO_ADT_CLASSRUN"))
        });
        if !implements {
            d.error(
                "CLD005",
                format!(
                    "Class '{}' contains IF_OO_ADT_CLASSRUN~MAIN but does not declare IF_OO_ADT_CLASSRUN.",
                    e.name
                ),
                e.start,
                e.end,
            );
        }
    }

    visit(&program.statements, policy, d);
}

fn visit(statements: &[Statement], policy: &CompilerPolicy, d: &mut DiagnosticBag) {
    for s in statements {
        match s {
            Statement::WriteStmt { start, end, .. } => {
                d.error(
                    "CLD006",
                    "WRITE is not a Cloud output mechanism in ABAPC; use out->write( ... ) inside IF_OO_ADT_CLASSRUN~MAIN.",
                    *start,
                    *end,
                );
            }
            Statement::DataDecl {
                start,
                end,
                name,
                parameter,
                selection_options,
                ..
            } => {
                if parameter.unwrap_or(false) || selection_options.unwrap_or(false) {
                    d.error(
                        "CLD007",
                        format!(
                            "Classic selection-screen declaration '{}' is not available in ABAP Cloud mode.",
                            name
                        ),
                        *start,
                        *end,
                    );
                }
            }
            Statement::EventBlock {
                start,
                end,
                event,
                statements,
                ..
            } => {
                d.error(
                    "CLD008",
                    format!("Classic report event '{}' is not supported in ABAP Cloud mode.", event),
                    *start,
                    *end,
                );
                visit(statements, policy, d);
            }
            Statement::CallFunction { start, end, name, .. } => {
                let allowed: HashSet<String> = policy
                    .released_apis
                    .functions
                    .iter()
                    .map(|x| x.to_ascii_uppercase())
                    .collect();
                if !allowed.contains(&name.to_ascii_uppercase()) {
                    d.error(
                        "CLD002",
                        format!("Function '{}' is not present in the configured released-API manifest.", name),
                        *start,
                        *end,
                    );
                }
            }
            Statement::IfStmt {
                branches,
                else_branch,
                ..
            } => {
                for b in branches {
                    visit(&b.statements, policy, d);
                }
                visit(else_branch, policy, d);
            }
            Statement::WhileStmt { body, .. }
            | Statement::DoStmt { body, .. }
            | Statement::LoopStmt { body, .. } => {
                visit(body, policy, d);
            }
            Statement::CaseStmt { whens, others, .. } => {
                for w in whens {
                    visit(&w.statements, policy, d);
                }
                visit(others, policy, d);
            }
            Statement::TryStmt { body, catches, .. } => {
                visit(body, policy, d);
                for c in catches {
                    visit(&c.body, policy, d);
                }
            }
            Statement::ClassDecl(class_decl) => {
                for m in &class_decl.methods {
                    visit(&m.statements, policy, d);
                }
            }
            _ => {}
        }
    }
}
