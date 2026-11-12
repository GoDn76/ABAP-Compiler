use super::ast::{ClassAttributeDecl, MethodDecl, Program, TypeDecl};
use super::diagnostics::Position;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ClassModel {
    pub name: String,
    pub super_class: Option<String>,
    pub is_final: bool,
    pub is_abstract: bool,
    pub interfaces: Vec<String>,
    pub types: Vec<TypeDecl>,
    pub attributes: HashMap<String, ClassAttributeDecl>,
    pub methods: HashMap<String, MethodDecl>,
    pub start: Position,
    pub end: Position,
}

pub fn build_class_models(program: &Program) -> HashMap<String, ClassModel> {
    let mut classes: HashMap<String, ClassModel> = HashMap::new();

    for declaration in &program.classes {
        let key = declaration.name.to_ascii_uppercase();
        let model = classes.entry(key.clone()).or_insert_with(|| ClassModel {
            name: declaration.name.clone(),
            super_class: declaration.super_class.clone(),
            is_final: declaration.is_final.unwrap_or(false),
            is_abstract: declaration.is_abstract.unwrap_or(false),
            interfaces: Vec::new(),
            types: Vec::new(),
            attributes: HashMap::new(),
            methods: HashMap::new(),
            start: declaration.start,
            end: declaration.end,
        });

        if declaration.super_class.is_some() {
            model.super_class = declaration.super_class.clone();
        }
        model.is_final = model.is_final || declaration.is_final.unwrap_or(false);
        model.is_abstract = model.is_abstract || declaration.is_abstract.unwrap_or(false);

        for iface in &declaration.interfaces {
            if !model.interfaces.iter().any(|x| x.eq_ignore_ascii_case(iface)) {
                model.interfaces.push(iface.clone());
            }
        }

        for ty in &declaration.types {
            if let Some(pos) = model.types.iter().position(|x| x.name.eq_ignore_ascii_case(&ty.name)) {
                model.types[pos] = ty.clone();
            } else {
                model.types.push(ty.clone());
            }
        }

        for attr in &declaration.attributes {
            let attr_key = attr.name.to_ascii_uppercase();
            let previous = model.attributes.get(&attr_key);
            if previous.is_none() || !declaration.implementation {
                model.attributes.insert(attr_key, attr.clone());
            }
        }

        for method in &declaration.methods {
            let method_key = method.name.to_ascii_uppercase();
            if let Some(previous) = model.methods.get(&method_key).cloned() {
                if declaration.implementation {
                    let mut m = method.clone();
                    m.params = if !previous.params.is_empty() { previous.params } else { m.params };
                    m.returning = previous.returning.or(m.returning);
                    m.is_static = previous.is_static.or(m.is_static);
                    m.is_final = previous.is_final.or(m.is_final);
                    m.visibility = previous.visibility.or(m.visibility);
                    m.redefinition = previous.redefinition.or(m.redefinition);
                    m.implemented = Some(true);
                    model.methods.insert(method_key, m);
                } else {
                    let mut m = previous;
                    m.params = method.params.clone();
                    m.returning = method.returning.clone().or(m.returning);
                    m.is_static = method.is_static.or(m.is_static);
                    m.is_final = method.is_final.or(m.is_final);
                    m.visibility = method.visibility.or(m.visibility);
                    m.redefinition = method.redefinition.or(m.redefinition);
                    model.methods.insert(method_key, m);
                }
            } else {
                model.methods.insert(method_key, method.clone());
            }
        }

        if declaration.implementation {
            model.end = declaration.end;
        }
    }

    // REDEFINITION inherits original signature unless provided
    let class_keys: Vec<String> = classes.keys().cloned().collect();
    for class_key in class_keys {
        let (super_class_name, method_keys) = {
            let model = classes.get(&class_key).unwrap();
            let super_class = model.super_class.clone();
            let redefs: Vec<String> = model
                .methods
                .iter()
                .filter(|(_, m)| m.redefinition.unwrap_or(false))
                .map(|(k, _)| k.clone())
                .collect();
            (super_class, redefs)
        };

        if let Some(super_name) = super_class_name {
            for method_key in method_keys {
                if let Some((_, inherited_method)) = resolve_method(&classes, &super_name, &method_key) {
                    let model = classes.get_mut(&class_key).unwrap();
                    let m = model.methods.get_mut(&method_key).unwrap();
                    if m.params.is_empty() {
                        m.params = inherited_method.params.clone();
                    }
                    if m.returning.is_none() {
                        m.returning = inherited_method.returning.clone();
                    }
                    if m.is_static.is_none() {
                        m.is_static = inherited_method.is_static;
                    }
                }
            }
        }
    }

    classes
}

pub fn class_lineage(models: &HashMap<String, ClassModel>, class_name: &str) -> Vec<ClassModel> {
    let mut lineage = Vec::new();
    let mut visited = HashSet::new();
    let mut current_key = class_name.to_ascii_uppercase();

    while let Some(current) = models.get(&current_key) {
        let key = current.name.to_ascii_uppercase();
        if visited.contains(&key) {
            break;
        }
        visited.insert(key);
        lineage.push(current.clone());

        if let Some(super_name) = &current.super_class {
            current_key = super_name.to_ascii_uppercase();
        } else {
            break;
        }
    }

    lineage
}

pub fn resolve_method(
    models: &HashMap<String, ClassModel>,
    class_name: &str,
    method_name: &str,
) -> Option<(ClassModel, MethodDecl)> {
    let m_upper = method_name.to_ascii_uppercase();
    for model in class_lineage(models, class_name) {
        if let Some(method) = model.methods.get(&m_upper).cloned() {
            return Some((model, method));
        }
    }
    None
}

pub fn inherited_attributes(
    models: &HashMap<String, ClassModel>,
    class_name: &str,
) -> Vec<ClassAttributeDecl> {
    let mut lineage = class_lineage(models, class_name);
    lineage.reverse();
    let mut by_name: HashMap<String, ClassAttributeDecl> = HashMap::new();
    for model in lineage {
        for attr in model.attributes.values() {
            by_name.insert(attr.name.to_ascii_uppercase(), attr.clone());
        }
    }
    by_name.into_values().collect()
}
