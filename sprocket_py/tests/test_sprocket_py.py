import sprocket_py


def test_parse_valid_wdl():
    result = sprocket_py.parse("version 1.1\nworkflow hello {}")
    assert result.version == "1.1"
    assert result.document_kind is None
    assert len(result.diagnostics) == 0


def test_parse_invalid_wdl():
    result = sprocket_py.parse("not valid wdl at all")
    assert len(result.diagnostics) > 0
    for diagnostic in result.diagnostics:
        assert diagnostic.severity in ("Error", "Warning", "Note")
        assert isinstance(diagnostic.message, str)


def test_parse_diagnostics_have_spans():
    result = sprocket_py.parse("version 1.1\ntask foo { invalid }")
    assert len(result.diagnostics) > 0
    for diagnostic in result.diagnostics:
        for label in diagnostic.labels:
            assert label.span.start >= 0
            assert label.span.end >= label.span.start


def test_parse_empty_string():
    result = sprocket_py.parse("")
    assert len(result.diagnostics) > 0


def test_structured_diagnostic_fields():
    result = sprocket_py.parse("bad")
    diagnostic = result.diagnostics[0]
    assert hasattr(diagnostic, "rule")
    assert hasattr(diagnostic, "severity")
    assert hasattr(diagnostic, "message")
    assert hasattr(diagnostic, "fix")
    assert hasattr(diagnostic, "labels")


def test_legacy_ast_api_still_returns_tuple():
    diagnostic_count, diagnostics = sprocket_py.parse_ast("version 1.1\nworkflow hello {}")
    assert diagnostic_count == 0
    assert diagnostics == []


def test_legacy_cst_api_still_returns_tuple():
    diagnostic_count, diagnostics = sprocket_py.parse_cst("bad")
    assert diagnostic_count == len(diagnostics)
    assert diagnostic_count > 0


def test_structured_cst_parse_returns_result():
    result = sprocket_py.parse_cst_structured("bad")
    assert result.version is None
    assert len(result.diagnostics) > 0
