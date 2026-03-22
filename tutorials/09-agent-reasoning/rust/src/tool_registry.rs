//! Tool registry: specifications, lookup, and argument validation.
//!
//! Tools are described by `ToolSpec` structs held in a flat `Vec`.  The agent
//! can look up a tool by its `name_id` (an opaque u32 identifier) and
//! validate a `ToolCallArgs` against the tool's declared parameters.

/// The kind of a tool parameter.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParamKind {
    StringParam,
    IntParam,
    BoolParam,
}

/// A single parameter in a tool specification.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ToolParam {
    pub name_id: u32,
    pub kind: ParamKind,
    pub required: bool,
}

/// The specification of a tool that the agent may call.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ToolSpec {
    pub name_id: u32,
    pub description_id: u32,
    pub params: Vec<ToolParam>,
}

/// Concrete arguments supplied in a tool call.
pub struct ToolCallArgs {
    /// Each entry is `(param_name_id, kind_of_value_provided)`.
    pub param_values: Vec<(u32, ParamKind)>,
}

/// Linear search for a tool by `name_id`.  Returns the index if found.
pub fn find_tool(registry: &[ToolSpec], name_id: u32) -> Option<usize> {
    let mut i: usize = 0;
    while i < registry.len() {
        if registry[i].name_id == name_id {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Check whether a single required parameter is present in the supplied args
/// with the correct kind.
fn param_satisfied(param: &ToolParam, args: &ToolCallArgs) -> bool {
    let mut i: usize = 0;
    while i < args.param_values.len() {
        if args.param_values[i].0 == param.name_id && args.param_values[i].1 == param.kind {
            return true;
        }
        i += 1;
    }
    false
}

/// Check that a supplied argument refers to a parameter that actually exists
/// in the spec, with matching kind.
fn arg_in_spec(arg_name_id: u32, arg_kind: &ParamKind, spec: &ToolSpec) -> bool {
    let mut i: usize = 0;
    while i < spec.params.len() {
        if spec.params[i].name_id == arg_name_id && spec.params[i].kind == *arg_kind {
            return true;
        }
        i += 1;
    }
    false
}

/// Validate a tool call against its specification.
///
/// Returns `true` when:
/// 1. Every **required** parameter in the spec is present in `args` with matching kind.
/// 2. Every argument in `args` refers to a parameter that exists in the spec with matching kind.
pub fn validate_tool_call(spec: &ToolSpec, args: &ToolCallArgs) -> bool {
    // Check all required params are satisfied
    let mut i: usize = 0;
    while i < spec.params.len() {
        if spec.params[i].required && !param_satisfied(&spec.params[i], args) {
            return false;
        }
        i += 1;
    }
    // Check all provided args are valid
    let mut j: usize = 0;
    while j < args.param_values.len() {
        if !arg_in_spec(args.param_values[j].0, &args.param_values[j].1, spec) {
            return false;
        }
        j += 1;
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_registry() -> Vec<ToolSpec> {
        vec![
            ToolSpec {
                name_id: 1,
                description_id: 100,
                params: vec![
                    ToolParam { name_id: 10, kind: ParamKind::StringParam, required: true },
                    ToolParam { name_id: 11, kind: ParamKind::IntParam, required: false },
                ],
            },
            ToolSpec {
                name_id: 2,
                description_id: 200,
                params: vec![
                    ToolParam { name_id: 20, kind: ParamKind::BoolParam, required: true },
                ],
            },
        ]
    }

    #[test]
    fn test_find_tool_present() {
        let reg = sample_registry();
        assert_eq!(find_tool(&reg, 1), Some(0));
        assert_eq!(find_tool(&reg, 2), Some(1));
    }

    #[test]
    fn test_find_tool_absent() {
        let reg = sample_registry();
        assert_eq!(find_tool(&reg, 99), None);
    }

    #[test]
    fn test_find_tool_empty_registry() {
        assert_eq!(find_tool(&[], 1), None);
    }

    #[test]
    fn test_validate_all_required_present() {
        let reg = sample_registry();
        let args = ToolCallArgs {
            param_values: vec![(10, ParamKind::StringParam)],
        };
        assert!(validate_tool_call(&reg[0], &args));
    }

    #[test]
    fn test_validate_with_optional() {
        let reg = sample_registry();
        let args = ToolCallArgs {
            param_values: vec![
                (10, ParamKind::StringParam),
                (11, ParamKind::IntParam),
            ],
        };
        assert!(validate_tool_call(&reg[0], &args));
    }

    #[test]
    fn test_validate_missing_required() {
        let reg = sample_registry();
        let args = ToolCallArgs {
            param_values: vec![(11, ParamKind::IntParam)],
        };
        assert!(!validate_tool_call(&reg[0], &args));
    }

    #[test]
    fn test_validate_wrong_kind() {
        let reg = sample_registry();
        let args = ToolCallArgs {
            param_values: vec![(10, ParamKind::IntParam)],  // should be StringParam
        };
        assert!(!validate_tool_call(&reg[0], &args));
    }

    #[test]
    fn test_validate_extra_unknown_param() {
        let reg = sample_registry();
        let args = ToolCallArgs {
            param_values: vec![
                (10, ParamKind::StringParam),
                (99, ParamKind::BoolParam),  // not in spec
            ],
        };
        assert!(!validate_tool_call(&reg[0], &args));
    }

    #[test]
    fn test_validate_empty_args_no_required() {
        let spec = ToolSpec {
            name_id: 3,
            description_id: 300,
            params: vec![
                ToolParam { name_id: 30, kind: ParamKind::StringParam, required: false },
            ],
        };
        let args = ToolCallArgs { param_values: vec![] };
        assert!(validate_tool_call(&spec, &args));
    }
}
