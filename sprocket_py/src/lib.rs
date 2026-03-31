use std::fmt::Debug;

use pyo3::{prelude::*, wrap_pyfunction};
use wdl_ast::{AstToken, Document};
use wdl_grammar::{Diagnostic, Severity, SyntaxTree};

#[pyclass(frozen)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PySpan {
    #[pyo3(get)]
    pub start: usize,
    #[pyo3(get)]
    pub end: usize,
}

#[pyclass(frozen)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyLabel {
    #[pyo3(get)]
    pub message: String,
    span: PySpan,
}

#[pymethods]
impl PyLabel {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<Py<PySpan>> {
        Py::new(py, self.span.clone())
    }
}

#[pyclass(frozen)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyDiagnostic {
    #[pyo3(get)]
    pub rule: Option<String>,
    #[pyo3(get)]
    pub severity: String,
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub fix: Option<String>,
    labels: Vec<PyLabel>,
}

#[pymethods]
impl PyDiagnostic {
    #[getter]
    fn labels(&self, py: Python<'_>) -> PyResult<Vec<Py<PyLabel>>> {
        self.labels
            .iter()
            .cloned()
            .map(|label| Py::new(py, label))
            .collect()
    }
}

#[pyclass(frozen)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseResult {
    diagnostics: Vec<PyDiagnostic>,
    #[pyo3(get)]
    pub version: Option<String>,
    #[pyo3(get)]
    pub document_kind: Option<String>,
}

#[pymethods]
impl ParseResult {
    #[getter]
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Vec<Py<PyDiagnostic>>> {
        self.diagnostics
            .iter()
            .cloned()
            .map(|diagnostic| Py::new(py, diagnostic))
            .collect()
    }
}

fn render_diagnostics<T>(diagnostics: &[T]) -> Vec<String>
where
    T: Debug,
{
    diagnostics
        .iter()
        .map(|diagnostic| format!("{diagnostic:?}"))
        .collect()
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "Error",
        Severity::Warning => "Warning",
        Severity::Note => "Note",
    }
}

fn map_diagnostic(diagnostic: &Diagnostic) -> PyDiagnostic {
    let labels = diagnostic
        .labels()
        .map(|label| {
            let span = label.span();
            PyLabel {
                message: label.message().to_string(),
                span: PySpan {
                    start: span.start(),
                    end: span.end(),
                },
            }
        })
        .collect();

    PyDiagnostic {
        rule: diagnostic.rule().map(str::to_owned),
        severity: severity_name(diagnostic.severity()).to_string(),
        message: diagnostic.message().to_string(),
        fix: diagnostic.fix().map(str::to_owned),
        labels,
    }
}

fn parse_result_with_version(version: Option<String>, diagnostics: &[Diagnostic]) -> ParseResult {
    ParseResult {
        diagnostics: diagnostics.iter().map(map_diagnostic).collect(),
        version,
        document_kind: None,
    }
}

fn parse_ast_inner(source: &str) -> (usize, Vec<String>) {
    let (_document, diagnostics) = Document::parse(source);
    let diagnostics = render_diagnostics(&diagnostics);
    (diagnostics.len(), diagnostics)
}

fn parse_cst_inner(source: &str) -> (usize, Vec<String>) {
    let (_tree, diagnostics) = SyntaxTree::parse(source);
    let diagnostics = render_diagnostics(&diagnostics);
    (diagnostics.len(), diagnostics)
}

fn parse_inner(source: &str) -> ParseResult {
    let (document, diagnostics) = Document::parse(source);
    let version = document
        .version_statement()
        .map(|statement| statement.version().text().to_string());

    parse_result_with_version(version, &diagnostics)
}

fn parse_cst_structured_inner(source: &str) -> ParseResult {
    let (_tree, diagnostics) = SyntaxTree::parse(source);
    parse_result_with_version(None, &diagnostics)
}

/// Parse WDL source through the typed AST entry point and return structured data.
#[pyfunction]
fn parse(py: Python<'_>, source: &str) -> PyResult<Py<ParseResult>> {
    Py::new(py, parse_inner(source))
}

/// Parse WDL source through the lower-level CST entry point and return structured data.
#[pyfunction]
fn parse_cst_structured(py: Python<'_>, source: &str) -> PyResult<Py<ParseResult>> {
    Py::new(py, parse_cst_structured_inner(source))
}

/// Parse WDL source through the typed AST entry point.
///
/// This legacy wrapper preserves the original benchmark contract.
#[pyfunction]
fn parse_ast(source: &str) -> (usize, Vec<String>) {
    parse_ast_inner(source)
}

/// Parse WDL source through the lower-level CST entry point.
///
/// This legacy wrapper preserves the original benchmark contract.
#[pyfunction]
fn parse_cst(source: &str) -> (usize, Vec<String>) {
    parse_cst_inner(source)
}

/// Python bindings for Sprocket's WDL parser crates.
#[pymodule]
fn sprocket_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySpan>()?;
    m.add_class::<PyLabel>()?;
    m.add_class::<PyDiagnostic>()?;
    m.add_class::<ParseResult>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_cst_structured, m)?)?;
    m.add_function(wrap_pyfunction!(parse_ast, m)?)?;
    m.add_function(wrap_pyfunction!(parse_cst, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_ast_inner, parse_cst_inner, parse_cst_structured_inner, parse_inner};

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

    #[test]
    fn structured_ast_parse_exposes_version() {
        let result = parse_inner(MINIMAL_WDL);
        assert_eq!(result.version.as_deref(), Some("1.1"));
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.document_kind, None);
    }

    #[test]
    fn structured_cst_parse_collects_diagnostics() {
        let result = parse_cst_structured_inner("not valid wdl");
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.version, None);
    }
}
