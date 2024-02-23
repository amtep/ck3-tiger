//! By splitting the builder up into stages, we achieve two goals.
//! - The order of calls is enforced, leading to more consistent code. E.g. calls to `weak()` or
//!     `strong()` should always directly follow the opening call.
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

pub fn report(key: ErrorKey, severity: Severity) -> ReportBuilderStage1 {
    ReportBuilderStage1::new(key, severity)
}

// =================================================================================================
// =============== Builder internals:
// =================================================================================================

#[derive(Debug, Clone, Copy)]
#[must_use]
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

    /// Optional step for when confidence is not known at compile time.
    pub fn conf(mut self, conf: Confidence) -> Self {
        self.2 = conf;
        self
    }

    /// Sets the main report message.
    pub fn msg<S: Into<String>>(self, msg: S) -> ReportBuilderStage2 {
        ReportBuilderStage2 { stage1: self, msg: msg.into(), info: None }
    }
}

#[derive(Debug)]
#[must_use]
pub struct ReportBuilderStage2 {
    stage1: ReportBuilderStage1,
    msg: String,
    info: Option<String>,
}

impl ReportBuilderStage2 {
    /// Optional step. Adds an info section to the report.
    pub fn info<S: Into<String>>(mut self, info: S) -> Self {
        let info = info.into();
        self.info = if info.is_empty() { None } else { Some(info) };
        self
    }

    /// Optional step. Adds an info section to the report if the `info` parameter is `Some`.
    pub fn opt_info<S: Into<String>>(mut self, info: Option<S>) -> Self {
        self.info = info.map(Into::into);
        self
    }

    pub fn loc<E: ErrorLoc>(self, eloc: E) -> ReportBuilderStage3 {
        let length = eloc.loc_length();
        ReportBuilderStage3 {
            stage1: self.stage1,
            msg: self.msg,
            info: self.info,
            pointers: vec![PointedMessage { loc: eloc.into_loc(), length, msg: None }],
        }
    }

    #[cfg(any(feature = "ck3", feature = "vic3"))]
    pub fn loc_msg<E: ErrorLoc, S: Into<String>>(self, eloc: E, msg: S) -> ReportBuilderStage3 {
        let length = eloc.loc_length();
        ReportBuilderStage3 {
            stage1: self.stage1,
            msg: self.msg,
            info: self.info,
            pointers: vec![PointedMessage { loc: eloc.into_loc(), length, msg: Some(msg.into()) }],
        }
    }

    pub fn pointers(self, pointers: Vec<PointedMessage>) -> ReportBuilderStage3 {
        ReportBuilderStage3 { stage1: self.stage1, msg: self.msg, info: self.info, pointers }
    }
}

#[derive(Debug)]
#[must_use]
pub struct ReportBuilderStage3 {
    stage1: ReportBuilderStage1,
    msg: String,
    info: Option<String>,
    pointers: Vec<PointedMessage>,
}

impl ReportBuilderStage3 {
    pub fn loc<E: ErrorLoc, S: Into<String>>(mut self, eloc: E, msg: S) -> Self {
        let length = eloc.loc_length();
        self.pointers.push(PointedMessage { loc: eloc.into_loc(), length, msg: Some(msg.into()) });
        self
    }
    pub fn opt_loc<E: ErrorLoc, S: Into<String>>(mut self, eloc: Option<E>, msg: S) -> Self {
        if let Some(eloc) = eloc {
            let length = eloc.loc_length();
            self.pointers.push(PointedMessage {
                loc: eloc.into_loc(),
                length,
                msg: Some(msg.into()),
            });
        }
        self
    }
    /// Build the report and return it.
    /// Build the report and return it.
    pub fn build(self) -> LogReport {
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
