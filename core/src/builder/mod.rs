use std::sync::Arc;

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::engine::{Engine, GQLRequest};

#[derive(Debug, Default)]

pub struct Input {
    pub name: String,
    pub fields: Vec<Field>,
    pub value: Option<Value>,
}

#[derive(Debug, Default)]
pub struct Output {
    pub name: String,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

impl Output {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Field {
    pub name: String,
    pub list: bool,
    pub wrap_list: bool,
    pub fields: Option<Vec<Field>>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct Query<'a> {
    pub engine: &'a dyn Engine,
    pub operation: String,
    pub name: String,
    pub method: String,
    pub model: String,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

impl<'a> Query<'a> {
    pub async fn perform<T: DeserializeOwned>(&self, request: GQLRequest) -> T {
        // TODO: check if engine running
        // println!("{:?}", &request);

        let response = self.engine.perform(request).await;

        // println!("{:?}", response);

        // TODO: error handling
        serde_json::from_value(response.data.unwrap().result).unwrap()
    }

    pub fn build(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!("{} {}", self.operation, self.name));
        string.push_str("{");
        string.push_str("result: ");

        string.push_str(&self.build_inner());

        string.push_str("}");

        return string;
    }

    fn build_inner(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!("{}{}", self.method, self.model));

        if self.inputs.len() > 0 {
            string.push_str(&self.build_inputs(&self.inputs));
        }

        string.push_str(" ");

        if self.outputs.len() > 0 {
            string.push_str(&self.build_outputs(&self.outputs));
        }

        string
    }

    fn build_inputs(&self, inputs: &Vec<Input>) -> String {
        let mut string = String::new();

        string.push_str("(");

        for input in inputs {
            string.push_str(&input.name);

            string.push_str(":");

            let next = match &input.value {
                Some(value) => serde_json::to_string(value).unwrap(),
                None => self.build_fields(false, false, &input.fields),
            };

            string.push_str(&next);

            string.push_str(",");
        }

        string.push_str(")");

        string
    }

    fn build_outputs(&self, outputs: &Vec<Output>) -> String {
        let mut string = String::new();

        string.push_str("{");

        for output in outputs {
            string.push_str(&output.name);
            string.push_str(" ");

            if output.inputs.len() > 0 {
                string.push_str(&self.build_inputs(&output.inputs));
            }

            if output.outputs.len() > 0 {
                string.push_str(&self.build_outputs(&output.outputs));
            }
        }

        string.push_str("}");

        string
    }

    fn build_fields(&self, list: bool, wrap_list: bool, fields: &Vec<Field>) -> String {
        let mut string = String::new();

        if !list {
            string.push_str("{");
        }

        for field in fields {
            if wrap_list {
                string.push_str("{");
            }

            if field.name != "" {
                string.push_str(&field.name);
                string.push_str(":")
            }

            if field.list {
                string.push_str("[");
            }

            if let Some(fields) = &field.fields {
                string.push_str(&self.build_fields(field.list, field.wrap_list, &fields));
            }

            if let Some(value) = &field.value {
                string.push_str(&serde_json::to_string(&value).unwrap());
            }

            if field.list {
                string.push_str("]");
            }

            if wrap_list {
                string.push_str("}");
            }

            string.push_str(",");
        }

        if !list {
            string.push_str("}");
        }

        string
    }
}

pub fn transform_equals(mut fields: Vec<Field>) -> Vec<Field> {
    for mut field in &mut fields {
        if let Some(fields) = &field.fields {
            if let Some(inner) = fields.iter().find(|f| f.name == "equals") {
                field.value = inner.value.clone();
                field.fields = None;
            }
        }
    }

    fields
}