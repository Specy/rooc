use crate::parser::il::PreExp;
use crate::parser::model_transformer::{TransformError, TransformerContext};
use crate::primitives::{Primitive, PrimitiveKind};
use crate::runtime_builtin::{default_type_check, RoocFunction};
use crate::type_checker::type_checker_context::{FunctionContext, TypeCheckerContext, WithType};
use crate::utils::serialize_json_compatible;
use js_sys::Function;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub enum JsFunctionRuntimeError {
    WrongType(String),
}
//TODO implement into
impl JsFunctionRuntimeError {
    pub fn into_js_value(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap_or(JsValue::NULL)
    }
}

#[derive(Debug, Clone)]
enum ReturnType {
    Primitive(PrimitiveKind),
    Function(Function),
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JsFunction {
    js_function: Function,
    function_name: String,
    arg_types: Vec<(String, PrimitiveKind)>,
    return_type: ReturnType,
    type_checker: Option<Function>,
}

#[wasm_bindgen]
impl JsFunction {
    pub fn clone_wasm(&self) -> JsFunction {
        self.clone()
    }

    pub fn new(
        js_function: JsValue,
        function_name: String,
        arg_types: Vec<JsValue>, //this is an array of (String, PrimitiveKind)
        return_type: JsValue,
        type_checker: JsValue,
    ) -> Result<JsFunction, JsValue> {
        let arg_types = arg_types
            .into_iter()
            .map(|arr| {
                serde_wasm_bindgen::from_value::<(String, PrimitiveKind)>(arr).map_err(|_| {
                    JsFunctionRuntimeError::WrongType(
                        "Expected a tuple of (String, PrimitiveKind)".to_string(),
                    )
                    .into_js_value()
                })
            })
            .collect::<Result<Vec<(String, PrimitiveKind)>, JsValue>>()?;
        if !js_function.is_function() {
            return Err(JsFunctionRuntimeError::WrongType(format!(
                "Expected a function, got {:?}",
                js_function
            ))
            .into_js_value());
        }
        let js_function = js_sys::Function::from(js_function);
        let return_type = if return_type.is_function() {
            ReturnType::Function(js_sys::Function::from(return_type))
        } else {
            ReturnType::Primitive(
                serde_wasm_bindgen::from_value::<PrimitiveKind>(return_type).map_err(|_| {
                    JsFunctionRuntimeError::WrongType("Expected a PrimitiveKind".to_string())
                        .into_js_value()
                })?,
            )
        };

        let type_checker = if type_checker.is_function() {
            Some(js_sys::Function::from(type_checker))
        } else {
            None
        };

        Ok(Self {
            js_function,
            function_name,
            arg_types,
            return_type,
            type_checker,
        })
    }
}

impl RoocFunction for JsFunction {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        let args = args
            .iter()
            .map(|arg| arg.as_primitive(context, fn_context))
            .collect::<Result<Vec<Primitive>, TransformError>>()?;
        let js_args =
            serialize_json_compatible(&args).map_err(|e| TransformError::Other(e.to_string()))?;
        let arr = js_sys::Array::from(&js_args);
        let result = self.js_function.apply(&JsValue::NULL, &arr).map_err(|e| {
            TransformError::Other(
                e.as_string()
                    .unwrap_or("Error thrown in function".to_string()),
            )
        })?;
        let primitive = serde_wasm_bindgen::from_value(result)
            .map_err(|e| TransformError::Other(e.to_string()))?;
        Ok(primitive)
    }

    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        self.arg_types.clone()
    }

    fn get_return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        match &self.return_type {
            ReturnType::Primitive(kind) => kind.clone(),
            ReturnType::Function(f) => {
                let static_primitives = args
                    .iter()
                    .map(|p| p.as_static_primitive())
                    .collect::<Vec<_>>();
                let args: Vec<PrimitiveKind> = args
                    .iter()
                    .map(|arg| arg.get_type(context, fn_context))
                    .collect();
                let js_args = serde_wasm_bindgen::to_value(&args).unwrap_or(JsValue::NULL);
                let js_static_primitives =
                    serde_wasm_bindgen::to_value(&static_primitives).unwrap_or(JsValue::NULL);
                let arr = js_sys::Array::from(&js_args);
                let arr_static_primitives = js_sys::Array::from(&js_static_primitives);
                let pars = js_sys::Array::new();
                pars.push(&arr);
                pars.push(&arr_static_primitives);
                let result = f.apply(&JsValue::NULL, &pars).unwrap_or(JsValue::NULL);
                serde_wasm_bindgen::from_value(result).unwrap_or(PrimitiveKind::Any)
            }
        }
    }

    fn get_function_name(&self) -> String {
        self.function_name.clone()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        if let Some(type_checker) = &self.type_checker {
            let args: Vec<PrimitiveKind> = args
                .iter()
                .map(|arg| arg.get_type(context, fn_context))
                .collect();
            let js_args = serde_wasm_bindgen::to_value(&args)
                .map_err(|e| TransformError::Other(e.to_string()))?;
            let arr = js_sys::Array::from(&js_args);
            let result = type_checker.apply(&JsValue::NULL, &arr).map_err(|e| {
                TransformError::Other(
                    e.as_string()
                        .unwrap_or("Error thrown in function".to_string()),
                )
            })?;
            if result.is_null() || result.is_undefined() {
                Ok(())
            } else {
                let transform_error: TransformError =
                    serde_wasm_bindgen::from_value::<String>(result)
                        .map(|e| TransformError::Other(e))
                        .map_err(|e| TransformError::Other(e.to_string()))?;
                Err(transform_error)
            }
        } else {
            default_type_check(args, &self.arg_types, context, fn_context)
        }
    }
}
