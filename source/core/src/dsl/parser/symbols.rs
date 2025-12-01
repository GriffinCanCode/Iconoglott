//! Symbol table and resolution pass for the iconoglott DSL
//!
//! Provides separate variable resolution with proper scoping and error reporting.

use super::ast::*;
use super::super::lexer::TokenValue;
use std::collections::HashMap;

#[allow(dead_code)] // Will be used for future scope features

/// A symbol in the symbol table
#[derive(Clone, Debug)]
pub struct Symbol {
    #[allow(dead_code)] // Used for error messages in future
    pub name: String,
    pub value: TokenValue,
    pub line: usize,
    pub col: usize,
}

/// A scope containing symbols
#[derive(Clone, Debug, Default)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self { Self::default() }

    pub fn with_parent(parent: Scope) -> Self {
        Self { symbols: HashMap::new(), parent: Some(Box::new(parent)) }
    }

    /// Define a symbol in current scope, returns previous definition if exists
    pub fn define(&mut self, name: String, value: TokenValue, line: usize, col: usize) -> Option<Symbol> {
        let symbol = Symbol { name: name.clone(), value, line, col };
        self.symbols.insert(name, symbol)
    }

    /// Look up a symbol, searching parent scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name).or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    /// Check if symbol exists in current scope only (not parents)
    #[allow(dead_code)] // Will be used for shadowing detection
    pub fn exists_local(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
}

/// Symbol table managing scopes and resolution
#[derive(Debug)]
pub struct SymbolTable {
    current: Scope,
}

impl Default for SymbolTable {
    fn default() -> Self { Self::new() }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { current: Scope::new() }
    }

    /// Define a variable in current scope
    pub fn define(&mut self, name: String, value: TokenValue, line: usize, col: usize) -> Option<Symbol> {
        self.current.define(name, value, line, col)
    }

    /// Look up a variable
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.current.lookup(name)
    }

    /// Enter a new nested scope
    pub fn push_scope(&mut self) {
        let parent = std::mem::take(&mut self.current);
        self.current = Scope::with_parent(parent);
    }

    /// Exit current scope, returning to parent
    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.current.parent.take() {
            self.current = *parent;
        }
    }
}

/// Resolution pass result
pub struct ResolveResult {
    pub ast: AstNode,
    pub errors: Vec<ParseError>,
}

/// Resolve variables in an AST, returning resolved AST and any errors
pub fn resolve(ast: AstNode) -> ResolveResult {
    let mut resolver = Resolver::new();
    let resolved = resolver.resolve_node(ast);
    ResolveResult { ast: resolved, errors: resolver.errors }
}

/// Resolver walks AST collecting definitions and resolving references
struct Resolver {
    symbols: SymbolTable,
    errors: Vec<ParseError>,
}

impl Resolver {
    fn new() -> Self {
        Self { symbols: SymbolTable::new(), errors: Vec::new() }
    }

    fn resolve_node(&mut self, node: AstNode) -> AstNode {
        match node {
            AstNode::Scene(children) => {
                // First pass: collect all variable definitions at scene level
                for child in &children {
                    if let AstNode::Variable { name, value } = child {
                        if let Some(val) = value {
                            // Check for duplicate in current scope
                            if let Some(prev) = self.symbols.define(name.clone(), val.clone(), 0, 0) {
                                self.errors.push(
                                    ParseError::new(
                                        format!("Variable '{}' already defined at line {}", name, prev.line),
                                        ErrorKind::DuplicateVariable, 0, 0
                                    ).with_suggestion(&format!("Previous definition was at {}:{}", prev.line, prev.col))
                                );
                            }
                        }
                    }
                }
                // Second pass: resolve all references
                let resolved: Vec<_> = children.into_iter().map(|c| self.resolve_node(c)).collect();
                AstNode::Scene(resolved)
            }
            AstNode::Shape(mut shape) => {
                // Resolve props that may have VarRefs
                shape.props = self.resolve_props(shape.props);
                // Resolve style colors
                shape.style = self.resolve_style(shape.style);
                // Recursively resolve children
                shape.children = shape.children.into_iter().map(|c| self.resolve_shape(c)).collect();
                AstNode::Shape(shape)
            }
            AstNode::Graph(mut graph) => {
                // Resolve node styles
                graph.nodes = graph.nodes.into_iter().map(|n| self.resolve_graph_node(n)).collect();
                // Resolve edge styles
                graph.edges = graph.edges.into_iter().map(|e| self.resolve_graph_edge(e)).collect();
                AstNode::Graph(graph)
            }
            AstNode::Variable { name, value } => AstNode::Variable { name, value },
            AstNode::Canvas(c) => AstNode::Canvas(self.resolve_canvas(c)),
            AstNode::Symbol(mut symbol) => {
                // Resolve children in symbol
                symbol.children = symbol.children.into_iter().map(|c| self.resolve_shape(c)).collect();
                AstNode::Symbol(symbol)
            }
            AstNode::Use(mut use_ref) => {
                // Resolve style in use reference
                use_ref.style = self.resolve_style(use_ref.style);
                AstNode::Use(use_ref)
            }
        }
    }

    fn resolve_shape(&mut self, mut shape: AstShape) -> AstShape {
        shape.props = self.resolve_props(shape.props);
        shape.style = self.resolve_style(shape.style);
        shape.children = shape.children.into_iter().map(|c| self.resolve_shape(c)).collect();
        shape
    }

    fn resolve_props(&mut self, props: HashMap<String, PropValue>) -> HashMap<String, PropValue> {
        props.into_iter().map(|(k, v)| (k, self.resolve_prop_value(v))).collect()
    }

    fn resolve_prop_value(&mut self, value: PropValue) -> PropValue {
        match value {
            PropValue::VarRef(name, line, col) => {
                if let Some(symbol) = self.symbols.lookup(&name) {
                    match &symbol.value {
                        TokenValue::Str(s) => PropValue::Str(s.clone()),
                        TokenValue::Num(n) => PropValue::Num(*n),
                        TokenValue::Pair(a, b) | TokenValue::PercentPair(a, b) => PropValue::Pair(*a, *b),
                        TokenValue::None => PropValue::None,
                    }
                } else {
                    self.errors.push(
                        ParseError::new(
                            format!("Undefined variable '{}'", name),
                            ErrorKind::UndefinedVariable, line, col
                        ).with_suggestion(&format!("Variable '{}' was used but never defined", name))
                    );
                    PropValue::None
                }
            }
            PropValue::Str(s) if s.starts_with("$VAR:") => {
                let name = &s[5..]; // strip "$VAR:"
                if let Some(symbol) = self.symbols.lookup(name) {
                    match &symbol.value {
                        TokenValue::Str(v) => PropValue::Str(v.clone()),
                        TokenValue::Num(n) => PropValue::Num(*n),
                        TokenValue::Pair(a, b) | TokenValue::PercentPair(a, b) => PropValue::Pair(*a, *b),
                        TokenValue::None => PropValue::None,
                    }
                } else {
                    self.errors.push(
                        ParseError::new(
                            format!("Undefined variable '{}'", name),
                            ErrorKind::UndefinedVariable, 0, 0
                        ).with_suggestion(&format!("Variable '{}' was used but never defined", name))
                    );
                    PropValue::None
                }
            }
            other => other,
        }
    }

    fn resolve_style(&mut self, mut style: AstStyle) -> AstStyle {
        // Resolve fill if it's a variable reference (marker format: $VAR:name)
        if let Some(ref fill) = style.fill {
            if let Some(name) = fill.strip_prefix("$VAR:") {
                if let Some(symbol) = self.symbols.lookup(name) {
                    if let TokenValue::Str(s) = &symbol.value {
                        style.fill = Some(s.clone());
                    }
                } else {
                    self.errors.push(
                        ParseError::new(
                            format!("Undefined variable '{}'", name),
                            ErrorKind::UndefinedVariable, 0, 0
                        ).with_suggestion(&format!("Variable '{}' was used but never defined. Define it with: ${} = #color", name, name))
                    );
                    style.fill = None;
                }
            }
        }
        // Resolve stroke if it's a variable reference
        if let Some(ref stroke) = style.stroke {
            if let Some(name) = stroke.strip_prefix("$VAR:") {
                if let Some(symbol) = self.symbols.lookup(name) {
                    if let TokenValue::Str(s) = &symbol.value {
                        style.stroke = Some(s.clone());
                    }
                } else {
                    self.errors.push(
                        ParseError::new(
                            format!("Undefined variable '{}'", name),
                            ErrorKind::UndefinedVariable, 0, 0
                        ).with_suggestion(&format!("Variable '{}' was used but never defined. Define it with: ${} = #color", name, name))
                    );
                    style.stroke = None;
                }
            }
        }
        style
    }

    fn resolve_canvas(&mut self, mut canvas: AstCanvas) -> AstCanvas {
        // Resolve fill if it's a variable reference
        if let Some(name) = canvas.fill.strip_prefix("$VAR:") {
            if let Some(symbol) = self.symbols.lookup(name) {
                if let TokenValue::Str(s) = &symbol.value {
                    canvas.fill = s.clone();
                }
            } else {
                self.errors.push(
                    ParseError::new(
                        format!("Undefined variable '{}'", name),
                        ErrorKind::UndefinedVariable, 0, 0
                    ).with_suggestion(&format!("Variable '{}' was used but never defined", name))
                );
                canvas.fill = "#fff".to_string(); // default
            }
        }
        canvas
    }

    fn resolve_graph_node(&mut self, mut node: GraphNode) -> GraphNode {
        node.style = self.resolve_style(node.style);
        node
    }

    fn resolve_graph_edge(&mut self, mut edge: GraphEdge) -> GraphEdge {
        // Resolve stroke if it's a variable reference
        if let Some(ref stroke) = edge.stroke {
            if let Some(name) = stroke.strip_prefix("$VAR:") {
                if let Some(symbol) = self.symbols.lookup(name) {
                    if let TokenValue::Str(s) = &symbol.value {
                        edge.stroke = Some(s.clone());
                    }
                } else {
                    self.errors.push(
                        ParseError::new(
                            format!("Undefined variable '{}'", name),
                            ErrorKind::UndefinedVariable, 0, 0
                        ).with_suggestion(&format!("Variable '{}' was used but never defined", name))
                    );
                    edge.stroke = None;
                }
            }
        }
        edge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_define_lookup() {
        let mut scope = Scope::new();
        scope.define("x".into(), TokenValue::Num(42.0), 1, 0);
        
        let sym = scope.lookup("x");
        assert!(sym.is_some());
        assert!(matches!(&sym.unwrap().value, TokenValue::Num(n) if (*n - 42.0).abs() < 0.001));
    }

    #[test]
    fn test_scope_parent_lookup() {
        let mut parent = Scope::new();
        parent.define("x".into(), TokenValue::Num(1.0), 1, 0);
        
        let child = Scope::with_parent(parent);
        let sym = child.lookup("x");
        assert!(sym.is_some());
    }

    #[test]
    fn test_symbol_table_scopes() {
        let mut table = SymbolTable::new();
        table.define("global".into(), TokenValue::Str("#fff".into()), 0, 0);
        
        table.push_scope();
        table.define("local".into(), TokenValue::Str("#000".into()), 1, 0);
        
        // Both visible in child scope
        assert!(table.lookup("global").is_some());
        assert!(table.lookup("local").is_some());
        
        table.pop_scope();
        
        // Only global visible after pop
        assert!(table.lookup("global").is_some());
        assert!(table.lookup("local").is_none());
    }

    #[test]
    fn test_resolve_undefined_variable() {
        let ast = AstNode::Scene(vec![
            AstNode::Shape(AstShape {
                kind: "rect".into(),
                props: [("fill".into(), PropValue::VarRef("undefined".into(), 1, 5))].into_iter().collect(),
                ..AstShape::new("rect")
            })
        ]);
        
        let result = resolve(ast);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ErrorKind::UndefinedVariable);
        assert!(result.errors[0].message.contains("undefined"));
    }

    #[test]
    fn test_resolve_defined_variable() {
        let ast = AstNode::Scene(vec![
            AstNode::Variable { 
                name: "accent".into(), 
                value: Some(TokenValue::Str("#ff0".into())) 
            },
            AstNode::Shape(AstShape {
                kind: "rect".into(),
                props: [("fill".into(), PropValue::VarRef("accent".into(), 2, 5))].into_iter().collect(),
                ..AstShape::new("rect")
            })
        ]);
        
        let result = resolve(ast);
        assert!(result.errors.is_empty());
        
        if let AstNode::Scene(children) = result.ast {
            if let AstNode::Shape(shape) = &children[1] {
                assert!(matches!(shape.props.get("fill"), Some(PropValue::Str(s)) if s == "#ff0"));
            }
        }
    }
}

