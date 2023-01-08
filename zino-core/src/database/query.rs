use crate::{Column, Map, Schema, Validation};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
/// SQL query builder.
pub struct Query {
    // Projection fields.
    fields: Vec<String>,
    // Filter.
    filter: Map,
    // Order.
    order: String,
    // Limit.
    limit: u64,
    // Offset.
    offset: u64,
}

impl Query {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            filter: Map::new(),
            order: String::new(),
            limit: 10,
            offset: 0,
        }
    }

    /// Updates the query using the json object and returns the validation result.
    #[must_use]
    pub fn read_map(&mut self, data: Map) -> Validation {
        let mut validation = Validation::new();
        let filter = &mut self.filter;
        let mut order = String::new();
        for (key, value) in data.into_iter() {
            match key.as_str() {
                "fields" => {
                    if let Some(fields) = Validation::parse_array(&value) {
                        self.fields = fields;
                    }
                }
                "sort_by" => {
                    if let Some(sort_by) = Validation::parse_string(&value) {
                        if sort_by.contains('.') {
                            order = sort_by.replace('.', "->'") + "'" + &order;
                        } else {
                            order = sort_by.to_string() + &order;
                        }
                    }
                }
                "sort_order" => {
                    if let Some(sort_order) = Validation::parse_string(&value) {
                        if sort_order.eq_ignore_ascii_case("asc") {
                            order += " ASC";
                        } else {
                            order += " DESC";
                        }
                    }
                }
                "limit" => {
                    if let Some(result) = Validation::parse_u64(&value) {
                        match result {
                            Ok(limit) => self.limit = limit,
                            Err(err) => validation.record_fail("limit", err.to_string()),
                        }
                    }
                }
                "offset" => {
                    if let Some(result) = Validation::parse_u64(&value) {
                        match result {
                            Ok(offset) => self.offset = offset,
                            Err(err) => validation.record_fail("offset", err.to_string()),
                        }
                    }
                }
                "timestamp" | "nonce" | "signature" => (),
                _ => {
                    if !key.starts_with('$') {
                        if key.contains('.') {
                            if let Some((key, path)) = key.split_once('.') {
                                if let Ok(index) = path.parse::<usize>() {
                                    if let Some(vec) = filter.get_mut(key) {
                                        if let Some(vec) = vec.as_array_mut() {
                                            if index > vec.len() {
                                                vec.resize(index, Value::Null);
                                            }
                                            vec.insert(index, value);
                                        }
                                    } else {
                                        let mut vec = Vec::new();
                                        vec.resize(index, Value::Null);
                                        vec.insert(index, value);
                                        filter.insert(key.to_string(), vec.into());
                                    }
                                } else if let Some(map) = filter.get_mut(key) {
                                    if let Some(map) = map.as_object_mut() {
                                        map.insert(path.to_string(), value);
                                    }
                                } else {
                                    let mut map = Map::new();
                                    map.insert(path.to_string(), value);
                                    filter.insert(key.to_string(), map.into());
                                }
                            }
                        } else if value != "" && value != "all" {
                            filter.insert(key, value);
                        }
                    }
                }
            }
        }
        if !(order.is_empty() || order.starts_with(' ')) {
            if order.ends_with(" ASC") || order.ends_with(" DESC") {
                self.order = order;
            } else {
                self.order = order + " DESC";
            }
        }
        validation
    }

    /// Retains the projection fields in the allow list of columns.
    /// If the projection fields are empty, it will be set to the allow list.
    #[inline]
    pub fn allow_fields<const N: usize>(&mut self, columns: [&str; N]) {
        let fields = &mut self.fields;
        if fields.is_empty() {
            self.fields = columns.map(|col| col.to_string()).to_vec();
        } else {
            fields.retain(|field| {
                columns
                    .iter()
                    .any(|col| field == col || field.ends_with(&format!(" {col}")))
            })
        }
    }

    /// Removes the projection fields in the deny list of columns.
    #[inline]
    pub fn deny_fields<const N: usize>(&mut self, columns: [&str; N]) {
        self.fields.retain(|field| {
            !columns
                .iter()
                .any(|col| field == col || field.ends_with(&format!(" {col}")))
        })
    }

    /// Appends to the query filter.
    #[inline]
    pub fn append_filter(&mut self, filter: &mut Map) {
        self.filter.append(filter);
    }

    /// Inserts a key-value pair into the query filter.
    #[inline]
    pub fn insert_filter(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.filter.insert(key.into(), value.into());
    }

    /// Sets the query order.
    #[inline]
    pub fn set_order(&mut self, order: String) {
        self.order = order;
    }

    /// Sets the query limit.
    #[inline]
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    /// Sets the query offset.
    #[inline]
    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    /// Returns a reference to the projection fields.
    #[inline]
    pub fn fields(&self) -> &[String] {
        self.fields.as_slice()
    }

    /// Returns a reference to the filter.
    #[inline]
    pub fn filter(&self) -> &Map {
        &self.filter
    }

    /// Returns a reference to the sort order.
    #[inline]
    pub fn order(&self) -> &str {
        &self.order
    }

    /// Returns the query limit.
    #[inline]
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the query offset.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Formats projection fields.
    pub(crate) fn format_fields(&self) -> String {
        let fields = &self.fields;
        if fields.is_empty() {
            "*".to_string()
        } else {
            fields.join(", ")
        }
    }

    // Formats the selection with a logic operator.
    fn format_selection<M: Schema>(selection: &Map, operator: &str) -> String {
        let mut conditions = Vec::new();
        for (key, value) in selection {
            match key.as_str() {
                "$and" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(condition);
                    }
                }
                "$or" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
                        conditions.push(condition);
                    }
                }
                "$not" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(format!("(NOT {condition})"));
                    }
                }
                "$nor" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
                        conditions.push(format!("(NOT {condition})"));
                    }
                }
                "$text" => {
                    if let Some(value) = value.as_object() {
                        if let Some(condition) = Self::parse_text_search(value) {
                            conditions.push(condition);
                        }
                    }
                }
                _ => {
                    if let Some(col) = M::get_column(key) {
                        let condition = col.format_postgres_filter(key, value);
                        conditions.push(condition);
                    }
                }
            }
        }
        if conditions.is_empty() {
            String::new()
        } else {
            format!("({})", conditions.join(operator))
        }
    }

    /// Formats the query filter to generate SQL `WHERE` expression.
    pub(crate) fn format_filter<M: Schema>(&self) -> String {
        let filter = &self.filter;
        if filter.is_empty() {
            return String::new();
        }

        let (sort_by, sort_order) = self.order.split_once(' ').unwrap_or(("", ""));
        let mut expression = " ".to_string();
        let mut conditions = Vec::new();
        for (key, value) in filter {
            match key.as_str() {
                "sample" => {
                    if let Some(Ok(value)) = Validation::parse_f64(value) {
                        let condition = format!("random() < {value}");
                        conditions.push(condition);
                    }
                }
                "$and" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(condition);
                    }
                }
                "$or" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
                        conditions.push(condition);
                    }
                }
                "$not" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(format!("NOT {condition}"));
                    }
                }
                "$nor" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
                        conditions.push(format!("NOT {condition}"));
                    }
                }
                "$text" => {
                    if let Some(value) = value.as_object() {
                        if let Some(condition) = Self::parse_text_search(value) {
                            conditions.push(condition);
                        }
                    }
                }
                "$join" => {
                    if let Some(value) = value.as_str() {
                        expression += value;
                    }
                }
                _ => {
                    if let Some(col) = M::get_column(key) {
                        let condition = if key == sort_by {
                            // Use the filter condition to optimize pagination offset.
                            let sort_order = sort_order.to_ascii_uppercase();
                            let operator = if sort_order.starts_with(" DESC") {
                                "<"
                            } else {
                                ">"
                            };
                            let value = col.encode_postgres_value(value);
                            format!("{key} {operator} {value}")
                        } else {
                            col.format_postgres_filter(key, value)
                        };
                        conditions.push(condition);
                    }
                }
            }
        }
        if !conditions.is_empty() {
            expression += &format!("WHERE {}", conditions.join(" AND "));
        };
        if let Some(Value::String(group_by)) = filter.get("group_by") {
            expression += &format!("GROUP BY {group_by}");
            if let Some(Value::Object(selection)) = filter.get("having") {
                let condition = Self::format_selection::<M>(selection, " AND ");
                expression += &format!("HAVING {condition}");
            }
        }
        expression
    }

    /// Formats the query sort to generate SQL `ORDER BY` expression.
    pub(crate) fn format_sort(&self) -> String {
        let order = &self.order;
        if order.is_empty() {
            String::new()
        } else {
            format!("ORDER BY {order}")
        }
    }

    /// Formats the query pagination to generate SQL `LIMIT` expression.
    pub(crate) fn format_pagination(&self) -> String {
        if let Some((sort_by, _)) = self.order.split_once(' ') {
            if self.filter.contains_key(sort_by) {
                return format!("LIMIT {}", self.limit);
            }
        }
        format!("LIMIT {} OFFSET {}", self.limit, self.offset)
    }

    /// Parses text search filter.
    fn parse_text_search(filter: &Map) -> Option<String> {
        let columns: Option<Vec<String>> = Validation::parse_array(filter.get("$columns"));
        if let Some(columns) = columns {
            if let Some(search) = Validation::parse_string(filter.get("$search")) {
                let column = columns.join(" || ' ' || ");
                let language = Validation::parse_string(filter.get("$language"))
                    .unwrap_or_else(|| "english".to_string());
                let search = Column::format_postgres_string(&search);
                let condition = format!(
                    "to_tsvector('{language}', {column}) @@ websearch_to_tsquery('{language}', '{search}')",
                );
                return Some(condition);
            }
        }
        None
    }
}