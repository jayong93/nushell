use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Category, Example, PipelineData, ShellError, Signature, Span, Type, Value};

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "math arctan"
    }

    fn signature(&self) -> Signature {
        Signature::build("math arctan")
            .switch("degrees", "Return degrees instead of radians", Some('d'))
            .input_output_types(vec![(Type::Number, Type::Float)])
            .vectorizes_over_list(true)
            .category(Category::Math)
    }

    fn usage(&self) -> &str {
        "Returns the arctangent of the number."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["trigonometry", "inverse"]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let head = call.head;
        let use_degrees = call.has_flag("degrees");
        // This doesn't match explicit nulls
        if matches!(input, PipelineData::Empty) {
            return Err(ShellError::PipelineEmpty(head));
        }
        input.map(
            move |value| operate(value, head, use_degrees),
            engine_state.ctrlc.clone(),
        )
    }

    fn examples(&self) -> Vec<Example> {
        let pi = std::f64::consts::PI;
        vec![
            Example {
                description: "Get the arctangent of 1",
                example: "1 | math arctan",
                result: Some(Value::test_float(pi / 4.0f64)),
            },
            Example {
                description: "Get the arctangent of -1 in degrees",
                example: "-1 | math arctan -d",
                result: Some(Value::test_float(-45.0)),
            },
        ]
    }
}

fn operate(value: Value, head: Span, use_degrees: bool) -> Value {
    match value {
        numeric @ (Value::Int { .. } | Value::Float { .. }) => {
            let (val, span) = match numeric {
                Value::Int { val, span } => (val as f64, span),
                Value::Float { val, span } => (val, span),
                _ => unreachable!(),
            };

            let val = val.atan();
            let val = if use_degrees { val.to_degrees() } else { val };

            Value::Float { val, span }
        }
        Value::Error { .. } => value,
        other => Value::Error {
            error: ShellError::OnlySupportsThisInputType(
                "numeric".into(),
                other.get_type().to_string(),
                head,
                other.expect_span(),
            ),
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
