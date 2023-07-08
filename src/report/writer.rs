use ansi_term::{ANSIString, ANSIStrings};
use unicode_width::UnicodeWidthChar;

use crate::fileset::FileKind;
use crate::report::errors::Errors;
use crate::report::output_style::Styled;
use crate::report::{LogReport, PointedMessage, Severity};

/// Source lines printed in the output have tab characters replaced by this string.
const SPACES_PER_TAB: &str = " ";
/// Within a single report, if all printed source files have leading whitespace in excess of
/// this number of spaces, the whitespace will be truncated.
const MAX_IDLE_SPACE: usize = 4;

/// Log the report.
pub fn log_report(errors: &mut Errors, report: &LogReport) {
    // Log error lvl and message:
    log_line_title(errors, report);
    let lines = lines(errors, report);
    let skippable_ws = skippable_ws(&lines);

    // Log the primary pointer:
    log_pointer(
        errors,
        None,
        report.primary(),
        lines.first().unwrap_or(&None),
        skippable_ws,
        report.indentation(),
        report.severity,
    );
    // Log the other pointers:
    report.pointers.windows(2).enumerate().for_each(|(index, pointers)| {
        log_pointer(
            errors,
            pointers.get(0),
            pointers.get(1).expect("Must exist."),
            lines.get(index + 1).unwrap_or(&None),
            skippable_ws,
            report.indentation(),
            report.severity,
        );
    });
    // Log the info line, if one exists.
    if let Some(info) = &report.info {
        log_line_info(errors, report.indentation(), info);
    }
    // Write a blank line to visually separate reports:
    println!();
}

fn log_pointer(
    errors: &Errors,
    previous: Option<&PointedMessage>,
    pointer: &PointedMessage,
    line: &Option<String>,
    skippable_ws: usize,
    indentation: usize,
    severity: Severity,
) {
    if previous.is_none()
        || previous.unwrap().location.pathname != pointer.location.pathname
        || previous.unwrap().location.kind != pointer.location.kind
    {
        // This pointer is not the same as the previous pointer. Print file location as well:
        log_line_file_location(errors, pointer, indentation);
    }
    if pointer.location.line == 0 {
        // Line being zero means the location is an entire file,
        // not any particular location within the file.
        return;
    }
    // If a line exists, slice it to skip the given number of spaces.
    if let Some(line) = line.as_ref().map(|v| &v[skippable_ws..]) {
        log_line_from_source(errors, pointer, indentation, line);
        log_line_carets(errors, pointer, line, skippable_ws, indentation, severity);
    }
}

/// Log the first line of a report, containing the severity level and the error message.
fn log_line_title(errors: &Errors, report: &LogReport) {
    let line: &[ANSIString<'static>] = &[
        errors
            .styles
            .style(&Styled::Tag(report.severity, true))
            .paint(format!("{}", report.severity)),
        errors
            .styles
            .style(&Styled::Tag(report.severity, false))
            .paint(format!("({})", report.key)),
        errors.styles.style(&Styled::Default).paint(": "),
        errors.styles.style(&Styled::ErrorMessage).paint(report.msg.to_string()),
    ];
    println!("{}", ANSIStrings(line));
}

/// Log the optional info line that is part of the overall report.
fn log_line_info(errors: &Errors, indentation: usize, info: &str) {
    let line_info: &[ANSIString<'static>] = &[
        errors.styles.style(&Styled::Default).paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(&Styled::Default).paint(" "),
        errors.styles.style(&Styled::Location).paint("="),
        errors.styles.style(&Styled::Default).paint(" "),
        errors.styles.style(&Styled::InfoTag).paint("Info:"),
        errors.styles.style(&Styled::Default).paint(" "),
        errors.styles.style(&Styled::Info).paint(info.to_string()),
    ];
    println!("{}", ANSIStrings(line_info));
}

/// Log the line containing the location's mod name and filename.
fn log_line_file_location(errors: &Errors, pointer: &PointedMessage, indentation: usize) {
    let line_filename: &[ANSIString<'static>] = &[
        errors.styles.style(&Styled::Default).paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(&Styled::Location).paint("-->"),
        errors.styles.style(&Styled::Default).paint(" "),
        errors
            .styles
            .style(&Styled::Location)
            .paint(format!("[{}]", kind_tag(errors, pointer.location.kind))),
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
        errors.styles.style(&Styled::SourceText).paint(line.to_string()),
    ];
    println!("{}", ANSIStrings(line_from_source));
}

fn log_line_carets(
    errors: &Errors,
    pointer: &PointedMessage,
    line: &str,
    skippable_ws: usize,
    indentation: usize,
    severity: Severity,
) {
    let mut spacing = String::new();
    for c in line
        .chars()
        .skip(skippable_ws)
        .take(pointer.location.column.saturating_sub(skippable_ws + 1))
    {
        for _ in 0..c.width().unwrap_or(0) {
            spacing.push(' ');
        }
    }
    // A line containing the carets that point upwards at the source line.
    let line_carets: &[ANSIString] = &[
        errors.styles.style(&Styled::Default).paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(&Styled::Default).paint(" "),
        errors.styles.style(&Styled::Location).paint("|"),
        errors.styles.style(&Styled::Default).paint(" "),
        errors.styles.style(&Styled::Default).paint(spacing.to_string()),
        errors.styles.style(&Styled::Tag(severity, true)).paint(format!(
            "{:^^width$}",
            "",
            width = pointer.length
        )),
        errors.styles.style(&Styled::Default).paint(" "),
        errors
            .styles
            .style(&Styled::Tag(severity, true))
            .paint(pointer.msg.as_deref().map_or("", |_| "<-- ")),
        errors
            .styles
            .style(&Styled::Tag(severity, true))
            .paint(pointer.msg.as_deref().unwrap_or("")),
    ];
    println!("{}", ANSIStrings(line_carets));
}

fn kind_tag(errors: &Errors, kind: FileKind) -> &str {
    match kind {
        FileKind::Internal => "Internal",
        FileKind::Clausewitz => "Clausewitz",
        FileKind::Jomini => "Jomini",
        FileKind::Vanilla => "CK3",
        FileKind::LoadedMod(idx) => &errors.loaded_mods_labels[idx as usize],
        FileKind::Mod => "MOD",
    }
}

/// Gathers all printable source lines and gets rid of tab characters for consistency.
fn lines(errors: &mut Errors, report: &LogReport) -> Vec<Option<String>> {
    report
        .pointers
        .iter()
        .map(|p| errors.get_line(&p.location).map(|line| line.replace('\t', SPACES_PER_TAB)))
        .collect()
}

/// Calculates how many leading spaces to skip from each printed source line.
fn skippable_ws(lines: &[Option<String>]) -> usize {
    lines
        .iter()
        .flatten()
        .map(|line| line.chars().take_while(|ch| ch == &' ').count())
        .min()
        // If there are no lines, this value doesn't matter anyway, so just return a zero:
        .map_or(0, |smallest_whitespace| smallest_whitespace.saturating_sub(MAX_IDLE_SPACE))
}
