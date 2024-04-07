use anyhow::{anyhow, Result};
use winnow::ascii::{alpha1, digit1, line_ending, space1};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, seq, terminated, trace};
use winnow::token::{take_till, take_while};
use winnow::{PResult, Parser};

use crate::report::{ErrorKey, Severity, SuppressionLocation, SuppressionReport};

/// Swallow ANSI color codes.
fn colorcode(input: &mut &str) -> PResult<()> {
    ("\x1b[", digit1, opt((';', digit1)), 'm').void().parse_next(input)
}

/// Swallow linear whitespace and ANSI color codes.
fn ws(input: &mut &str) -> PResult<()> {
    trace("ws", repeat(.., alt((colorcode, space1.void())))).parse_next(input)
}

fn severity(input: &mut &str) -> PResult<Severity> {
    alpha1.parse_to().parse_next(input)
}

fn errorkey(input: &mut &str) -> PResult<ErrorKey> {
    take_while(1.., ('a'..='z', '-')).parse_to().parse_next(input)
}

fn restofline(input: &mut &str) -> PResult<String> {
    terminated(take_till(1.., b"\x1b\r\n"), (opt(colorcode), line_ending))
        .map(str::to_owned)
        .parse_next(input)
}

/// Swallow the `[MOD]` etc marker, which is not needed for suppressions.
fn filekind(input: &mut &str) -> PResult<()> {
    delimited('[', take_till(1.., ']'), ']').void().parse_next(input)
}

/// Swallow the carets on the caret line, leaving only the tag (if any)
fn caret_line_start(input: &mut &str) -> PResult<()> {
    (ws, '|', ws, take_while(1.., '^'), ws).void().parse_next(input)
}

/// Parse the first line of an error report, containing the severity, errorkey, and error message.
fn top_line(input: &mut &str) -> PResult<(Severity, ErrorKey, String)> {
    seq!(_: ws, severity, delimited('(', errorkey, ')'), _: (opt(colorcode), ':', ws), restofline)
        .parse_next(input)
}

/// Parse the top line of an error location, containing the filekind and filename.
/// Return only the filename.
fn file_loc_line(input: &mut &str) -> PResult<String> {
    trace(
        "file_loc_line",
        preceded((delimited(ws, "-->", ws), delimited(ws, filekind, ws)), restofline),
    )
    .parse_next(input)
}

/// Parse the part of an error location that displays the line number and the line contents.
fn line_contents_line(input: &mut &str) -> PResult<(u32, String)> {
    trace("line_contents_line", seq!(_: ws, digit1.parse_to(), _: (ws, '|', ws), restofline))
        .parse_next(input)
}

/// Parse a line that uses carets to point at the error location column.
/// This line may have an optional "tag" with a tag message.
/// Return that tag if it's present.
fn caret_tag_line(input: &mut &str) -> PResult<Option<String>> {
    trace(
        "caret_tag_line",
        preceded(
            caret_line_start,
            alt((preceded(("<--", ws), restofline).map(|s| Some(s)), line_ending.map(|_| None))),
        ),
    )
    .parse_next(input)
}

/// Parse the optional "Info:" line at the end of a report
fn info_line(input: &mut &str) -> PResult<String> {
    trace("info_line", preceded((ws, '=', ws, "Info:", ws), restofline)).parse_next(input)
}

fn suppression_location(input: &mut &str) -> PResult<SuppressionLocation> {
    (file_loc_line, opt((line_contents_line, caret_tag_line)))
        .map(|(path, opt)| {
            if let Some(((_linenr, line), tag)) = opt {
                SuppressionLocation { path, line: Some(line), tag }
            } else {
                SuppressionLocation { path, line: None, tag: None }
            }
        })
        .parse_next(input)
}

fn report(input: &mut &str) -> PResult<SuppressionReport> {
    (top_line, repeat(1.., suppression_location), opt(info_line), line_ending)
        .map(|((_, key, message), locations, _, _)| SuppressionReport { key, message, locations })
        .parse_next(input)
}

pub(crate) fn parse_suppressions(input: &str) -> Result<Vec<SuppressionReport>> {
    repeat(0.., report).parse(input).map_err(|e| anyhow!("could not read suppressions: {e}"))
}
