use std::fmt::Display;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::pipe::pipe::{PipeError, Pipeable, PipeableData};
use crate::solvers::simplex::IntoCanonicalTableau;
use crate::transformers::linearizer::Linearizer;
use crate::RoocParser;

#[wasm_bindgen]
pub enum Pipes {
    CompilerPipe,
    PreModelPipe,
    ModelPipe,
    LinearModelPipe,
    StandardLinearModelPipe,
    TableauPipe,
    OptimalTableauPipe,
}

//-------------------- Source Compiler --------------------
pub struct CompilerPipe {}
impl CompilerPipe {
    pub fn new() -> CompilerPipe {
        CompilerPipe {}
    }
}

impl Pipeable for CompilerPipe {
    fn pipe(&self, data: &mut PipeableData) -> Result<PipeableData, PipeError> {
        let str = data.as_string_data()?;
        let parser = RoocParser::new(str.clone());
        Ok(PipeableData::Parser(parser))
    }
}
//-------------------- Pre Model --------------------
pub struct PreModelPipe {}
impl PreModelPipe {
    pub fn new() -> PreModelPipe {
        PreModelPipe {}
    }
}
impl Pipeable for PreModelPipe {
    fn pipe(&self, data: &mut PipeableData) -> Result<PipeableData, PipeError> {
        let parser = data.as_parser()?;
        match parser.parse() {
            Ok(model) => Ok(PipeableData::PreModel(model)),
            Err(e) => Err(PipeError::CompilationError {
                error: e,
                source: parser.source.clone(),
            }),
        }
    }
}
//-------------------- Model --------------------
pub struct ModelPipe {}
impl ModelPipe {
    pub fn new() -> ModelPipe {
        ModelPipe {}
    }
}
impl Pipeable for ModelPipe {
    fn pipe(&self, data: &mut PipeableData) -> Result<PipeableData, PipeError> {
        let pre_model = data.as_pre_model()?;
        if let Err(e) = pre_model.create_type_checker() {
            return Err(PipeError::TransformError {
                error: e,
                source: pre_model.get_source().unwrap_or("".to_string()),
            });
        }
        match pre_model.clone().transform() {
            Ok(model) => Ok(PipeableData::Model(model)),
            Err(e) => Err(PipeError::TransformError {
                error: e,
                source: pre_model.get_source().unwrap_or("".to_string()),
            }),
        }
    }
}
//-------------------- Linear Model --------------------
pub struct LinearModelPipe {}
impl LinearModelPipe {
    pub fn new() -> LinearModelPipe {
        LinearModelPipe {}
    }
}
impl Pipeable for LinearModelPipe {
    fn pipe(&self, data: &mut PipeableData) -> Result<PipeableData, PipeError> {
        let model = data.as_model()?;
        let linearizer = Linearizer::linearize(model.clone());
        match linearizer {
            Ok(linear) => Ok(PipeableData::LinearModel(linear)),
            Err(e) => Err(PipeError::LinearizationError(e)),
        }
    }
}
//-------------------- Standard Linear Model --------------------
pub struct StandardLinearModelPipe {}
impl StandardLinearModelPipe {
    pub fn new() -> StandardLinearModelPipe {
        StandardLinearModelPipe {}
    }
}
impl Pipeable for StandardLinearModelPipe {
    fn pipe(&self, data: &mut PipeableData) -> Result<PipeableData, PipeError> {
        let linear_model = data.as_linear_model()?;
        let standard = linear_model.clone().into_standard_form();
        match standard {
            Ok(standard) => Ok(PipeableData::StandardLinearModel(standard)),
            Err(e) => Err(PipeError::StandardizationError(e)),
        }
    }
}
//-------------------- Tableau --------------------
pub struct TableauPipe {}
impl TableauPipe {
    pub fn new() -> TableauPipe {
        TableauPipe {}
    }
}
impl Pipeable for TableauPipe {
    fn pipe(&self, data: &mut PipeableData) -> Result<PipeableData, PipeError> {
        let standard_linear_model = data.as_standard_linear_model()?;
        let tableau = standard_linear_model.into_canonical();
        match tableau {
            Ok(tableau) => Ok(PipeableData::Tableau(tableau)),
            Err(e) => Err(PipeError::CanonicalizationError(e)),
        }
    }
}
//-------------------- Optimal Tableau --------------------
pub struct OptimalTableauPipe {}
impl OptimalTableauPipe {
    pub fn new() -> OptimalTableauPipe {
        OptimalTableauPipe {}
    }
}
impl Pipeable for OptimalTableauPipe {
    fn pipe(&self, data: &mut PipeableData) -> Result<PipeableData, PipeError> {
        let mut tableau = data.as_tableau()?.clone();
        let optimal_tableau = tableau.solve(1000);
        match optimal_tableau {
            Ok(optimal_tableau) => Ok(PipeableData::OptimalTableau(optimal_tableau)),
            Err(e) => Err(PipeError::SimplexError(e, tableau)),
        }
    }
}
