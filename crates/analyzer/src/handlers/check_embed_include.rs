use crate::analyzer_error::AnalyzerError;
use std::fs;
use std::path::PathBuf;
use veryl_parser::ParolError;
use veryl_parser::resource_table;
use veryl_parser::veryl_grammar_trait::*;
use veryl_parser::veryl_token::TokenSource;
use veryl_parser::veryl_walker::{Handler, HandlerPoint};

#[derive(Default)]
pub struct CheckEmbedInclude {
    pub errors: Vec<AnalyzerError>,
    point: HandlerPoint,
    in_component: bool,
}

impl CheckEmbedInclude {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Handler for CheckEmbedInclude {
    fn set_point(&mut self, p: HandlerPoint) {
        self.point = p;
    }
}

impl VerylGrammarTrait for CheckEmbedInclude {
    fn module_declaration(&mut self, _arg: &ModuleDeclaration) -> Result<(), ParolError> {
        match self.point {
            HandlerPoint::Before => self.in_component = true,
            HandlerPoint::After => self.in_component = false,
        }
        Ok(())
    }

    fn interface_declaration(&mut self, _arg: &InterfaceDeclaration) -> Result<(), ParolError> {
        match self.point {
            HandlerPoint::Before => self.in_component = true,
            HandlerPoint::After => self.in_component = false,
        }
        Ok(())
    }

    fn package_declaration(&mut self, _arg: &PackageDeclaration) -> Result<(), ParolError> {
        match self.point {
            HandlerPoint::Before => self.in_component = true,
            HandlerPoint::After => self.in_component = false,
        }
        Ok(())
    }

    fn embed_declaration(&mut self, arg: &EmbedDeclaration) -> Result<(), ParolError> {
        if let HandlerPoint::Before = self.point {
            let way = arg.identifier.identifier_token.to_string();
            let lang = arg.identifier0.identifier_token.to_string();

            if !EMBED_WAY.contains(&way.as_str()) {
                self.errors.push(AnalyzerError::unknown_embed_way(
                    &way,
                    &arg.identifier.as_ref().into(),
                ));
            }

            if !EMBED_LANG.contains(&lang.as_str()) {
                self.errors.push(AnalyzerError::unknown_embed_lang(
                    &lang,
                    &arg.identifier0.as_ref().into(),
                ));
            }

            if self.in_component && (way != "inline" || lang != "sv") {
                self.errors.push(AnalyzerError::invalid_embed(
                    &way,
                    &lang,
                    &arg.identifier.as_ref().into(),
                ));
            }
        }
        Ok(())
    }

    fn include_declaration(&mut self, arg: &IncludeDeclaration) -> Result<(), ParolError> {
        if let HandlerPoint::Before = self.point {
            let way = arg.identifier.identifier_token.to_string();

            if !INCLUDE_WAY.contains(&way.as_str()) {
                self.errors.push(AnalyzerError::unknown_include_way(
                    &way,
                    &arg.identifier.as_ref().into(),
                ));
            }

            let path = arg.string_literal.string_literal_token.to_string();
            let path = path.strip_prefix('"').unwrap();
            let path = path.strip_suffix('"').unwrap();
            if let TokenSource::File { path: x, .. } = arg.identifier.identifier_token.token.source
            {
                let mut base = resource_table::get_path_value(x).unwrap();
                if base.starts_with("file://") {
                    base = PathBuf::from(base.to_string_lossy().strip_prefix("file://").unwrap());
                }
                let base = base.parent().unwrap();
                let path = base.join(path);
                let text = fs::read_to_string(&path);
                if let Err(err) = text {
                    self.errors.push(AnalyzerError::include_failure(
                        path.to_string_lossy().as_ref(),
                        &err.to_string(),
                        &arg.string_literal.string_literal_token.token.into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

const EMBED_WAY: [&str; 2] = ["inline", "cocotb"];
const EMBED_LANG: [&str; 2] = ["sv", "py"];
const INCLUDE_WAY: [&str; 2] = ["inline", "cocotb"];
