//! By splitting the builder up into stages, we achieve two goals.
//! - The order of calls is enforced, leading to more consistent code. E.g. calls to weak() or
//!     strong() should always directly follow the opening call.
//! - The user is forced to add at least one pointer, making it impossible to create a report
//!     without pointers, which would lead to panics.

use crate::report::{log, Confidence, ErrorKey, ErrorLoc, LogReport, PointedMessage, Severity};

// =================================================================================================
// =============== Starting points:
// =================================================================================================

pub fn tips(key: ErrorKey) -> ReportBuilderStage1 {
    ReportBuilderStage1::new(key, Severity::Tips)
}

pub fn untidy(key: ErrorKey) -> ReportBuilderStage1 {
    ReportBuilderStage1::new(key, Severity::Untidy)
}

pub fn warn(key: ErrorKey) -> ReportBuilderStage1 {
    ReportBuilderStage1::new(key, Severity::Warning)
}

pub fn err(key: ErrorKey) -> ReportBuilderStage1 {
    ReportBuilderStage1::new(key, Severity::Error)
}

pub fn fatal(key: ErrorKey) -> ReportBuilderStage1 {
    ReportBuilderStage1::new(key, Severity::Fatal)
}

// =================================================================================================
// =============== Builder internals:
// =================================================================================================

#[derive(Debug, Clone, Copy)]
pub struct ReportBuilderStage1(ErrorKey, Severity, Confidence);

impl ReportBuilderStage1 {
    /// For internal use only.
    fn new(key: ErrorKey, severity: Severity) -> Self {
        Self(key, severity, Confidence::Reasonable)
    }
    /// Optional step. Confidence defaults to Reasonable but this overrides it to Weak.
    pub fn weak(mut self) -> Self {
        self.2 = Confidence::Weak;
        self
    }
    /// Optional step. Confidence defaults to Reasonable but this overrides it to Strong.
    pub fn strong(mut self) -> Self {
        self.2 = Confidence::Strong;
        self
    }
    /// Sets the main report message.
    pub fn msg(self, msg: &str) -> ReportBuilderStage2 {
        ReportBuilderStage2 { stage1: self, msg, info: None }
    }
}

#[derive(Debug)]
pub struct ReportBuilderStage2<'a> {
    stage1: ReportBuilderStage1,
    msg: &'a str,
    info: Option<&'a str>,
}

impl<'a> ReportBuilderStage2<'a> {
    /// Optional step. Adds an info section to the report.
    pub fn info(mut self, info: &'a str) -> Self {
        self.info = if info.is_empty() { None } else { Some(info) };
        self
    }
    pub fn loc<E: ErrorLoc>(self, loc: E) -> ReportBuilderStage3<'a> {
        ReportBuilderStage3 {
            stage1: self.stage1,
            msg: self.msg,
            info: self.info,
            pointers: vec![PointedMessage { location: loc.into_loc(), length: 1, msg: None }],
        }
    }
    pub fn loc_msg<E: ErrorLoc>(self, loc: E, msg: &'a str) -> ReportBuilderStage3 {
        ReportBuilderStage3 {
            stage1: self.stage1,
            msg: self.msg,
            info: self.info,
            pointers: vec![PointedMessage { location: loc.into_loc(), length: 1, msg: Some(msg) }],
        }
    }
    pub fn pointers(self, pointers: Vec<PointedMessage<'a>>) -> ReportBuilderStage3 {
        ReportBuilderStage3 { stage1: self.stage1, msg: self.msg, info: self.info, pointers }
    }
}

#[derive(Debug)]
pub struct ReportBuilderStage3<'a> {
    stage1: ReportBuilderStage1,
    msg: &'a str,
    info: Option<&'a str>,
    pointers: Vec<PointedMessage<'a>>,
}

impl<'a> ReportBuilderStage3<'a> {
    pub fn loc<E: ErrorLoc>(mut self, loc: E, msg: &'a str) -> Self {
        self.pointers.push(PointedMessage { location: loc.into_loc(), length: 1, msg: Some(msg) });
        self
    }
    /// Build the report and returns it.
    pub fn build(self) -> LogReport<'a> {
        LogReport {
            key: self.stage1.0,
            severity: self.stage1.1,
            confidence: self.stage1.2,
            msg: self.msg,
            info: self.info,
            pointers: self.pointers,
        }
    }
    /// Build the report and push it to be printed.
    pub fn push(self) {
        log(self.build());
    }
}
