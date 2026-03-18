use std::fmt::Debug;

use pyo3::{prelude::*, wrap_pyfunction};
use wdl_ast::Document;
use wdl_grammar::SyntaxTree;

fn render_diagnostics<T>(diagnostics: Vec<T>) -> Vec<String>
where
    T: Debug,
{
    diagnostics
        .into_iter()
        .map(|diagnostic| format!("{diagnostic:?}"))
        .collect()
}

fn parse_ast_inner(source: &str) -> (usize, Vec<String>) {
    let (_document, diagnostics) = Document::parse(source);
    let diagnostics = render_diagnostics(diagnostics);
    (diagnostics.len(), diagnostics)
}

fn parse_cst_inner(source: &str) -> (usize, Vec<String>) {
    let (_tree, diagnostics) = SyntaxTree::parse(source);
    let diagnostics = render_diagnostics(diagnostics);
    (diagnostics.len(), diagnostics)
}

/// Parse WDL source through the typed AST entry point.
#[pyfunction]
fn parse_ast(source: &str) -> (usize, Vec<String>) {
    parse_ast_inner(source)
}

/// Parse WDL source through the lower-level CST entry point.
#[pyfunction]
fn parse_cst(source: &str) -> (usize, Vec<String>) {
    parse_cst_inner(source)
}

/// Python bindings for Sprocket's WDL parser crates.
#[pymodule]
fn sprocket_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_ast, m)?)?;
    m.add_function(wrap_pyfunction!(parse_cst, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_ast_inner, parse_cst_inner};

    const MINIMAL_WDL: &str = "version 1.1\nworkflow hello {}";

    #[test]
    fn parses_minimal_document_via_ast() {
        let (diagnostic_count, diagnostics) = parse_ast_inner(MINIMAL_WDL);
        assert_eq!(diagnostic_count, 0, "{diagnostics:?}");
    }

    #[test]
    fn parses_minimal_document_via_cst() {
        let (diagnostic_count, diagnostics) = parse_cst_inner(MINIMAL_WDL);
        assert_eq!(diagnostic_count, 0, "{diagnostics:?}");
    }
}
