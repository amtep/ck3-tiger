use ansi_term::{ANSIString, ANSIStrings};
use unicode_width::UnicodeWidthChar;

use crate::fileset::FileKind;
use crate::output_style::Styled;
use crate::report::errors::Errors;
use crate::report::{LogReport, PointedMessage, Severity};

/// Log the report.
pub fn log_report(errors: &mut Errors, report: &LogReport) {
    // Log error lvl and message:
    log_line_title(errors, &report);
    // Log the primary pointer:
    log_pointer_primary(
        errors,
        report.primary(),
        report.indentation(),
        report.lvl.severity,
    );
    // Log the other pointers:
    report.pointers.windows(2).for_each(|pointers| {
        log_pointer(
            errors,
            pointers.get(0).expect("Must exist."),
            pointers.get(1).expect("Must exist."),
            report.indentation(),
            report.lvl.severity,
        );
    });
    if let Some(info) = report.info {
        let line_info: &[ANSIString<'static>] = &[
            errors.styles.style(&Styled::Default).paint(format!(
                "{:width$}",
                "",
                width = report.indentation()
            )),
            errors.styles.style(&Styled::Default).paint(" "),
            errors.styles.style(&Styled::Location).paint("="),
            errors.styles.style(&Styled::Default).paint(" "),
            errors.styles.style(&Styled::InfoTag).paint("Info:"),
            errors.styles.style(&Styled::Default).paint(" "),
            errors.styles.style(&Styled::Info).paint(format!("{info}")),
        ];
        println!("{}", ANSIStrings(line_info));
    }

    // if let Some(link) = &loc.link {
    //     self.log(link, level, key, "from here", None);
    // }

    // Write a blank line to visually separate reports:
    println!();
}

fn log_pointer_primary(
    errors: &mut Errors,
    pointer: &PointedMessage,
    indentation: usize,
    severity: Severity,
) {
    log_line_file_location(errors, pointer, indentation);
    if pointer.location.line == 0 {
        // Zero-length line means the location is an entire file,
        // not any particular location within the file.
        return;
    }
    if let Some(line) = errors.get_line(&pointer.location) {
        log_line_from_source(errors, pointer, indentation, &line);
        log_line_carets(errors, pointer, &line, indentation, severity);
    }
}

fn log_pointer(
    errors: &mut Errors,
    previous: &PointedMessage,
    pointer: &PointedMessage,
    indentation: usize,
    severity: Severity,
) {
    if previous.location.pathname != pointer.location.pathname
        || previous.location.kind != pointer.location.kind
    {
        // This pointer is not the same as the previous pointer. Print file location as well:
        log_line_file_location(errors, pointer, indentation);
    } else {
        log_line_blank(errors, indentation);
    }
    if pointer.location.line == 0 {
        // Zero-length line means the location is an entire file,
        // not any particular location within the file.
        return;
    }
    if let Some(line) = errors.get_line(&pointer.location) {
        log_line_from_source(errors, pointer, indentation, &line);
        log_line_carets(errors, pointer, &line, indentation, severity);
    }
}

/// Log the first line of a report, containing the severity level and the error message.
fn log_line_title(errors: &Errors, report: &LogReport) {
    let line: &[ANSIString<'static>] = &[
        errors
            .styles
            .style(&Styled::Tag(report.lvl.severity, true))
            .paint(format!("{}", report.lvl.severity)),
        errors
            .styles
            .style(&Styled::Tag(report.lvl.severity, false))
            .paint("("),
        errors
            .styles
            .style(&Styled::Tag(report.lvl.severity, false))
            .paint(format!("{}", report.key)),
        errors
            .styles
            .style(&Styled::Tag(report.lvl.severity, false))
            .paint(")"),
        errors.styles.style(&Styled::Default).paint(": "),
        errors
            .styles
            .style(&Styled::ErrorMessage)
            .paint(format!("{}", report.msg)),
    ];
    println!("{}", ANSIStrings(line));
}

/// Log the line containing the location's mod name and filename.
fn log_line_file_location(errors: &Errors, pointer: &PointedMessage, indentation: usize) {
    let line_filename: &[ANSIString<'static>] = &[
        errors
            .styles
            .style(&Styled::Default)
            .paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(&Styled::Location).paint("-->"),
        errors.styles.style(&Styled::Default).paint(" "),
        errors.styles.style(&Styled::Location).paint("["),
        errors
            .styles
            .style(&Styled::Location)
            .paint(format!("{}", kind_tag(errors, pointer.location.kind))),
        errors.styles.style(&Styled::Location).paint("]"),
        errors.styles.style(&Styled::Default).paint(" "),
        errors
            .styles
            .style(&Styled::Location)
            .paint(format!("{}", pointer.location.pathname.display())),
    ];
    println!("{}", ANSIStrings(line_filename));
}

/// Print a line from the source file.
fn log_line_from_source(errors: &Errors, pointer: &PointedMessage, indentation: usize, line: &str) {
    let line_from_source: &[ANSIString<'static>] = &[
        errors.styles.style(&Styled::Location).paint(format!(
            "{:width$}",
            pointer.location.line,
            width = indentation
        )),
        errors.styles.style(&Styled::Default).paint(" "),
        errors.styles.style(&Styled::Location).paint("|"),
        errors.styles.style(&Styled::Default).paint(" "),
        errors
            .styles
            .style(&Styled::SourceText)
            .paint(format!("{line}")),
    ];
    println!("{}", ANSIStrings(line_from_source));
}

fn log_line_carets(
    errors: &Errors,
    pointer: &PointedMessage,
    line: &str,
    indentation: usize,
    severity: Severity,
) {
    let mut spacing = String::new();
    for c in line.chars().take(pointer.location.column.saturating_sub(1)) {
        if c == '\t' {
            // spacing.push_str("  ");
            spacing.push('\t');
        } else {
            for _ in 0..c.width().unwrap_or(0) {
                spacing.push(' ');
            }
        }
    }
    // A line containing the carets that point upwards at the source line.
    let line_carets: &[ANSIString<'static>] = &[
        errors
            .styles
            .style(&Styled::Default)
            .paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(&Styled::Default).paint(" "),
        errors.styles.style(&Styled::Location).paint("|"),
        errors.styles.style(&Styled::Default).paint(" "),
        errors
            .styles
            .style(&Styled::Default)
            .paint(format!("{spacing}")),
        errors
            .styles
            .style(&Styled::Tag(severity, true))
            .paint(format!("{:^^width$}", "", width = pointer.length)),
        errors.styles.style(&Styled::Default).paint(" "),
        errors
            .styles
            .style(&Styled::Tag(severity, true))
            .paint(format!(
                "{}",
                pointer.msg.as_ref().map(|_| "<-- ").unwrap_or(&"")
            )),
        errors
            .styles
            .style(&Styled::Tag(severity, true))
            .paint(format!("{}", pointer.msg.as_ref().unwrap_or(&""))),
    ];
    println!("{}", ANSIStrings(line_carets));
}

/// Print a blank line to represent space between two lines in the same file.
fn log_line_blank(errors: &Errors, indentation: usize) {
    let line_blank: &[ANSIString<'static>] = &[
        errors
            .styles
            .style(&Styled::Location)
            .paint("-".repeat(indentation)),
        errors.styles.style(&Styled::Default).paint("   "),
    ];
    println!("{}", ANSIStrings(line_blank));
}

fn kind_tag(errors: &Errors, kind: FileKind) -> &str {
    match kind {
        FileKind::Vanilla => "CK3",
        FileKind::LoadedMod(idx) => &errors.loaded_mods_labels[idx as usize],
        FileKind::Mod => "MOD",
    }
}
