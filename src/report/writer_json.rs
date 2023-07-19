use serde_json::json;

use crate::report::errors::Errors;
use crate::report::writer::kind_tag;
use crate::report::LogReport;

/// Log the report in JSON format.
pub fn log_report_json(errors: &mut Errors, report: &LogReport) {
    let pointers: Vec<_> = report
        .pointers
        .iter()
        .map(|pointer| {
            let path = pointer.loc.pathname();
            json!({
                "path": path,
                "from": kind_tag(errors, pointer.loc.kind),
                "fullpath": errors.get_fullpath(pointer.loc.kind, path),
                "linenr": if pointer.loc.line == 0 { None } else { Some(pointer.loc.line) },
                "column": if pointer.loc.column == 0 { None } else { Some(pointer.loc.column) },
                "length": pointer.length,
                "line": errors.get_line(&pointer.loc),
                "tag": pointer.msg,
            })
        })
        .collect();
    let report = json!({
        "severity": report.severity,
        "confidence": report.confidence,
        "key": report.key,
        "message": &report.msg,
        "info": &report.info,
        "locations": pointers,
    });

    if let Err(e) = serde_json::to_writer_pretty(errors.output.get_mut(), &report) {
        eprintln!("JSON error: {e:#}");
    }
}
