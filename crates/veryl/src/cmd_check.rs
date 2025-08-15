use crate::OptCheck;
use crate::context::Context;
use log::info;
use miette::{self, Diagnostic, IntoDiagnostic, Result, WrapErr};
use std::fs;
use thiserror::Error;
use veryl_analyzer::{Analyzer, AnalyzerError};
use veryl_metadata::Metadata;
use veryl_parser::Parser;

pub struct CmdCheck {
    opt: OptCheck,
}

#[derive(Error, Diagnostic, Debug)]
#[error("veryl check failed")]
pub struct CheckError {
    #[related]
    pub related: Vec<AnalyzerError>,
    error_count: u32,
    error_count_limit: u32,
}

impl CheckError {
    pub fn new(error_count_limit: u32) -> Self {
        Self {
            related: Vec::new(),
            error_count: 0,
            error_count_limit,
        }
    }

    pub fn append(mut self, x: &mut Vec<AnalyzerError>) -> Self {
        for x in x.drain(0..) {
            if !x.is_error() || self.error_count_limit == 0 {
                self.related.push(x);
            } else if self.error_count < self.error_count_limit {
                self.related.push(x);
                self.error_count += 1;
            }
        }
        self
    }

    pub fn check_err(self) -> Result<Self> {
        if self.related.iter().all(|x| !x.is_error()) {
            Ok(self)
        } else {
            Err(self.into())
        }
    }

    pub fn check_all(self) -> Result<Self> {
        if self.related.is_empty() {
            Ok(self)
        } else {
            Err(self.into())
        }
    }
}

impl CmdCheck {
    pub fn new(opt: OptCheck) -> Self {
        Self { opt }
    }

    pub fn exec(&self, metadata: &mut Metadata) -> Result<bool> {
        let paths = metadata.paths(&self.opt.files, true, true)?;

        let mut check_error = CheckError::new(metadata.build.error_count_limit);
        let mut contexts = Vec::new();

        for path in &paths {
            info!("Processing file ({})", path.src.to_string_lossy());

            let input = fs::read_to_string(&path.src)
                .into_diagnostic()
                .wrap_err("")?;
            let parser = Parser::parse(&input, &path.src)?;

            let analyzer = Analyzer::new(metadata);
            let mut errors = analyzer.analyze_pass1(&path.prj, &path.src, &parser.veryl);
            check_error = check_error.append(&mut errors).check_err()?;

            let context = Context::new(path.clone(), input, parser, analyzer)?;
            contexts.push(context);
        }

        let mut errors = Analyzer::analyze_post_pass1();
        check_error = check_error.append(&mut errors).check_err()?;

        for context in &contexts {
            let path = &context.path;
            let mut errors =
                context
                    .analyzer
                    .analyze_pass2(&path.prj, &path.src, &context.parser.veryl);
            check_error = check_error.append(&mut errors).check_err()?;
        }

        let info = Analyzer::analyze_post_pass2();

        for context in &contexts {
            let path = &context.path;
            let mut errors =
                context
                    .analyzer
                    .analyze_pass3(&path.prj, &path.src, &context.parser.veryl, &info);
            check_error = check_error.append(&mut errors).check_err()?;
        }

        let _ = check_error.check_all()?;
        Ok(true)
    }
}
