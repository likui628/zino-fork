use crate::{
    extension::{TomlTableExt, TomlValueExt},
    TomlValue,
};
use convert_case::{Case, Casing};
use toml::Table;
use utoipa::openapi::{
    content::Content,
    path::{Operation, OperationBuilder, Parameter, ParameterBuilder, ParameterIn, PathItemType},
    request_body::{RequestBody, RequestBodyBuilder},
    schema::{KnownFormat, Object, ObjectBuilder, Ref, Schema, SchemaFormat, SchemaType},
    tag::{Tag, TagBuilder},
    Deprecated, Required,
};

/// Parses the tag.
pub(super) fn parse_tag(name: &str, config: &Table) -> Tag {
    let mut tag_builder = TagBuilder::new().name(name);
    if let Some(name) = config.get_str("name") {
        tag_builder = tag_builder.name(name);
    }
    if let Some(description) = config.get_str("description") {
        tag_builder = tag_builder.description(Some(description));
    }
    tag_builder.build()
}

/// Parses the operation.
pub(super) fn parse_operation(name: &str, path: &str, config: &Table) -> Operation {
    let mut operation_builder = OperationBuilder::new()
        .tag(name)
        .response("default", Ref::from_response_name("default"))
        .response("error", Ref::from_response_name("error"));
    if let Some(tags) = config.get_str_array("tags") {
        let tags = tags.into_iter().map(|s| s.to_owned()).collect::<Vec<_>>();
        operation_builder = operation_builder.tags(Some(tags));
    }
    if let Some(tag) = config.get_str("tag") {
        operation_builder = operation_builder.tag(tag);
    }
    if let Some(summary) = config.get_str("summary") {
        operation_builder = operation_builder.summary(Some(summary));
    }
    if let Some(description) = config.get_str("description") {
        operation_builder = operation_builder.description(Some(description));
    }
    if let Some(operation_id) = config.get_str("operation_id") {
        operation_builder = operation_builder.operation_id(Some(operation_id));
    }
    if let Some(deprecated) = config.get_bool("deprecated") {
        let deprecated = if deprecated {
            Deprecated::True
        } else {
            Deprecated::False
        };
        operation_builder = operation_builder.deprecated(Some(deprecated));
    }
    for parameter in self::parse_path_parameters(path).into_iter() {
        operation_builder = operation_builder.parameter(parameter);
    }
    if let Some(query) = config.get_table("query") {
        for parameter in self::parse_query_parameters(query).into_iter() {
            operation_builder = operation_builder.parameter(parameter);
        }
    }
    if let Some(body) = config.get_table("requestBody") {
        let request_body = self::parse_request_body(body);
        operation_builder = operation_builder.request_body(Some(request_body));
    }
    operation_builder.build()
}

/// Parses the schema.
pub(super) fn parse_schema(config: &Table) -> Schema {
    let schema_type_name = config.get_str("type").unwrap_or("object");
    let is_array_object = schema_type_name == "array" && config.get_str("items") == Some("object");
    let schema_type = if is_array_object {
        SchemaType::Object
    } else {
        parse_schema_type(schema_type_name)
    };
    let mut object_builder = ObjectBuilder::new().schema_type(schema_type);
    for (key, value) in config {
        if key == "default" {
            object_builder = object_builder.default(Some(value.to_json_value()));
        } else if key == "example" {
            object_builder = object_builder.example(Some(value.to_json_value()));
        }
        match value {
            TomlValue::String(value) => match key.as_str() {
                "format" => {
                    let format = parse_schema_format(value);
                    object_builder = object_builder.format(Some(format));
                }
                "title" => {
                    object_builder = object_builder.title(Some(value));
                }
                "description" => {
                    object_builder = object_builder.description(Some(value));
                }
                "pattern" => {
                    object_builder = object_builder.pattern(Some(value));
                }
                _ => {
                    if !(key == "type" || (key == "items" && is_array_object)) {
                        let object = Object::with_type(parse_schema_type(value));
                        object_builder = object_builder.property(key, object);
                    }
                }
            },
            TomlValue::Integer(value) => match key.as_str() {
                "max_length" => {
                    object_builder = object_builder.max_length(usize::try_from(*value).ok());
                }
                "min_length" => {
                    object_builder = object_builder.min_length(usize::try_from(*value).ok());
                }
                "max_properties" => {
                    object_builder = object_builder.max_properties(usize::try_from(*value).ok());
                }
                "min_properties" => {
                    object_builder = object_builder.min_properties(usize::try_from(*value).ok());
                }
                _ => (),
            },
            TomlValue::Float(value) => match key.as_str() {
                "multiple_of" => {
                    object_builder = object_builder.multiple_of(Some(*value));
                }
                "maximum" => {
                    object_builder = object_builder.maximum(Some(*value));
                }
                "minimum" => {
                    object_builder = object_builder.minimum(Some(*value));
                }
                "exclusive_maximum" => {
                    object_builder = object_builder.exclusive_maximum(Some(*value));
                }
                "exclusive_minimum" => {
                    object_builder = object_builder.exclusive_minimum(Some(*value));
                }
                _ => (),
            },
            TomlValue::Boolean(value) => match key.as_str() {
                "write_only" => {
                    object_builder = object_builder.write_only(Some(*value));
                }
                "read_only" => {
                    object_builder = object_builder.read_only(Some(*value));
                }
                "nullable" => {
                    object_builder = object_builder.nullable(*value);
                }
                "deprecated" => {
                    let deprecated = if *value {
                        Deprecated::True
                    } else {
                        Deprecated::False
                    };
                    object_builder = object_builder.deprecated(Some(deprecated));
                }
                _ => (),
            },
            TomlValue::Array(vec) => match key.as_str() {
                "required" => {
                    for field in vec.iter().filter_map(|v| v.as_str()) {
                        object_builder = object_builder.required(field);
                    }
                }
                "enum" => {
                    let values = vec.iter().filter_map(|v| v.as_str());
                    object_builder = object_builder.enum_values(Some(values));
                }
                "examples" => {
                    for example in vec.iter() {
                        object_builder = object_builder.example(Some(example.to_json_value()));
                    }
                }
                _ => (),
            },
            TomlValue::Table(config) => {
                let object = parse_schema(config);
                object_builder = object_builder.property(key, object);
            }
            _ => (),
        }
    }
    if is_array_object {
        Schema::Array(object_builder.to_array_builder().build())
    } else {
        Schema::Object(object_builder.build())
    }
}

/// Parses the path item type.
pub(super) fn parse_path_item_type(method: &str) -> PathItemType {
    match method {
        "POST" => PathItemType::Post,
        "PUT" => PathItemType::Put,
        "DELETE" => PathItemType::Delete,
        "OPTIONS" => PathItemType::Options,
        "HEAD" => PathItemType::Head,
        "PATCH" => PathItemType::Patch,
        "TRACE" => PathItemType::Trace,
        "CONNECT" => PathItemType::Connect,
        _ => PathItemType::Get,
    }
}

/// Parses the schema type.
fn parse_schema_type(basic_type: &str) -> SchemaType {
    match basic_type {
        "boolean" => SchemaType::Boolean,
        "integer" => SchemaType::Integer,
        "number" => SchemaType::Number,
        "string" => SchemaType::String,
        "array" => SchemaType::Array,
        "object" => SchemaType::Object,
        _ => SchemaType::Value,
    }
}

/// Parses the schema format.
fn parse_schema_format(format: &str) -> SchemaFormat {
    match format {
        "int8" => SchemaFormat::KnownFormat(KnownFormat::Int8),
        "int16" => SchemaFormat::KnownFormat(KnownFormat::Int16),
        "int32" => SchemaFormat::KnownFormat(KnownFormat::Int32),
        "int64" => SchemaFormat::KnownFormat(KnownFormat::Int64),
        "uint8" => SchemaFormat::KnownFormat(KnownFormat::UInt8),
        "uint16" => SchemaFormat::KnownFormat(KnownFormat::UInt16),
        "uint32" => SchemaFormat::KnownFormat(KnownFormat::UInt32),
        "uint64" => SchemaFormat::KnownFormat(KnownFormat::UInt64),
        "float" => SchemaFormat::KnownFormat(KnownFormat::Float),
        "double" => SchemaFormat::KnownFormat(KnownFormat::Double),
        "byte" => SchemaFormat::KnownFormat(KnownFormat::Byte),
        "binary" => SchemaFormat::KnownFormat(KnownFormat::Binary),
        "date" => SchemaFormat::KnownFormat(KnownFormat::Date),
        "datetime" => SchemaFormat::KnownFormat(KnownFormat::DateTime),
        "password" => SchemaFormat::KnownFormat(KnownFormat::Password),
        "uuid" => SchemaFormat::KnownFormat(KnownFormat::Uuid),
        _ => SchemaFormat::Custom(format.to_owned()),
    }
}

/// Parses the path parameters.
fn parse_path_parameters(path: &str) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for segment in path.split('/') {
        if let Some(part) = segment.strip_prefix('{') && let Some(name) = part.strip_suffix('}') {
            let schema_name = name.to_case(Case::Camel);
            let parameter = ParameterBuilder::new()
                .name(name)
                .schema(Some(Ref::from_schema_name(schema_name)))
                .parameter_in(ParameterIn::Path)
                .required(Required::True)
                .build();
            parameters.push(parameter);
        }
    }
    parameters
}

/// Parses the query parameters.
fn parse_query_parameters(query: &Table) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for (key, value) in query {
        let mut parameter_builder = ParameterBuilder::new()
            .name(key)
            .parameter_in(ParameterIn::Query);
        if let Some(config) = value.as_table() {
            if let Some(schema) = config.get_str("schema") {
                let schema_name = schema.to_case(Case::Camel);
                let schema_object = Ref::from_schema_name(schema_name);
                parameter_builder = parameter_builder.schema(Some(schema_object));
            } else {
                let object = parse_schema(config);
                parameter_builder = parameter_builder.schema(Some(object));
            };
        } else if let Some(basic_type) = value.as_str() {
            let object = Object::with_type(parse_schema_type(basic_type));
            parameter_builder = parameter_builder.schema(Some(object));
        }
        parameters.push(parameter_builder.build());
    }
    parameters
}

/// Parses the request body.
fn parse_request_body(config: &Table) -> RequestBody {
    let mut body_builder = RequestBodyBuilder::new().required(Some(Required::True));
    if let Some(schema) = config.get_str("schema") {
        body_builder = body_builder.content(
            "application/json",
            Content::new(Ref::from_schema_name(schema)),
        );
    }
    body_builder.build()
}