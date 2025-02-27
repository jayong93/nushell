use ical::parser::vcard::component::*;
use ical::property::Property;
use indexmap::map::IndexMap;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, Spanned, Type,
    Value,
};

#[derive(Clone)]
pub struct FromVcf;

impl Command for FromVcf {
    fn name(&self) -> &str {
        "from vcf"
    }

    fn signature(&self) -> Signature {
        Signature::build("from vcf")
            .input_output_types(vec![(Type::String, Type::Table(vec![]))])
            .category(Category::Formats)
    }

    fn usage(&self) -> &str {
        "Parse text as .vcf and create table."
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let head = call.head;
        from_vcf(input, head)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "'BEGIN:VCARD
N:Foo
FN:Bar
EMAIL:foo@bar.com
END:VCARD' | from vcf",
            description: "Converts ics formatted string to table",
            result: Some(Value::List {
                vals: vec![Value::Record {
                    cols: vec!["properties".to_string()],
                    vals: vec![Value::List {
                        vals: vec![
                            Value::Record {
                                cols: vec![
                                    "name".to_string(),
                                    "value".to_string(),
                                    "params".to_string(),
                                ],
                                vals: vec![
                                    Value::test_string("N"),
                                    Value::test_string("Foo"),
                                    Value::Nothing {
                                        span: Span::test_data(),
                                    },
                                ],
                                span: Span::test_data(),
                            },
                            Value::Record {
                                cols: vec![
                                    "name".to_string(),
                                    "value".to_string(),
                                    "params".to_string(),
                                ],
                                vals: vec![
                                    Value::test_string("FN"),
                                    Value::test_string("Bar"),
                                    Value::Nothing {
                                        span: Span::test_data(),
                                    },
                                ],
                                span: Span::test_data(),
                            },
                            Value::Record {
                                cols: vec![
                                    "name".to_string(),
                                    "value".to_string(),
                                    "params".to_string(),
                                ],
                                vals: vec![
                                    Value::test_string("EMAIL"),
                                    Value::test_string("foo@bar.com"),
                                    Value::Nothing {
                                        span: Span::test_data(),
                                    },
                                ],
                                span: Span::test_data(),
                            },
                        ],
                        span: Span::test_data(),
                    }],
                    span: Span::test_data(),
                }],
                span: Span::test_data(),
            }),
        }]
    }
}

fn from_vcf(input: PipelineData, head: Span) -> Result<PipelineData, ShellError> {
    let (input_string, span, metadata) = input.collect_string_strict(head)?;

    let input_string = input_string
        .lines()
        .map(|x| x.trim().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let input_bytes = input_string.as_bytes();
    let cursor = std::io::Cursor::new(input_bytes);
    let parser = ical::VcardParser::new(cursor);

    let iter = parser.map(move |contact| match contact {
        Ok(c) => contact_to_value(c, head),
        Err(e) => Value::Error {
            error: ShellError::UnsupportedInput(
                format!("input cannot be parsed as .vcf ({e})"),
                "value originates from here".into(),
                head,
                span,
            ),
        },
    });

    let collected: Vec<_> = iter.collect();
    Ok(Value::List {
        vals: collected,
        span: head,
    }
    .into_pipeline_data_with_metadata(metadata))
}

fn contact_to_value(contact: VcardContact, span: Span) -> Value {
    let mut row = IndexMap::new();
    row.insert(
        "properties".to_string(),
        properties_to_value(contact.properties, span),
    );
    Value::from(Spanned { item: row, span })
}

fn properties_to_value(properties: Vec<Property>, span: Span) -> Value {
    Value::List {
        vals: properties
            .into_iter()
            .map(|prop| {
                let mut row = IndexMap::new();

                let name = Value::String {
                    val: prop.name,
                    span,
                };
                let value = match prop.value {
                    Some(val) => Value::String { val, span },
                    None => Value::Nothing { span },
                };
                let params = match prop.params {
                    Some(param_list) => params_to_value(param_list, span),
                    None => Value::Nothing { span },
                };

                row.insert("name".to_string(), name);
                row.insert("value".to_string(), value);
                row.insert("params".to_string(), params);
                Value::from(Spanned { item: row, span })
            })
            .collect::<Vec<Value>>(),
        span,
    }
}

fn params_to_value(params: Vec<(String, Vec<String>)>, span: Span) -> Value {
    let mut row = IndexMap::new();

    for (param_name, param_values) in params {
        let values: Vec<Value> = param_values
            .into_iter()
            .map(|val| Value::string(val, span))
            .collect();
        let values = Value::List { vals: values, span };
        row.insert(param_name, values);
    }

    Value::from(Spanned { item: row, span })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(FromVcf {})
    }
}
