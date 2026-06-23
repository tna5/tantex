use std::collections::HashMap;
use tantivy::schema::Field;

/// Canonical name for the internal per-path sub-field of a json field.
/// E.g. `subfield_internal_name("selectors", "emails")` → `"__sub__selectors__emails"`.
pub fn subfield_internal_name(json_field: &str, path: &str) -> String {
    format!("__sub__{}__{}", json_field, path)
}

/// Parse an internal sub-field name back into `(json_field, path)`.
/// Returns `None` for non-internal names.
pub fn parse_subfield_name(name: &str) -> Option<(String, String)> {
    let rest = name.strip_prefix("__sub__")?;
    let sep = rest.find("__")?;
    Some((rest[..sep].to_string(), rest[sep + 2..].to_string()))
}

/// Build a lookup map `"selectors.emails" → Field` from `field_map`.
/// Only entries whose name matches the `__sub__` convention are included.
pub fn subfield_routes_from_field_map(field_map: &HashMap<String, Field>) -> HashMap<String, Field> {
    field_map
        .iter()
        .filter_map(|(name, &field)| {
            parse_subfield_name(name).map(|(json_field, path)| {
                (format!("{}.{}", json_field, path), field)
            })
        })
        .collect()
}

/// Rewrite a user query so that `selectors.emails:` → `__sub__selectors__emails:`.
/// Safe: `:` only appears as a field-value separator in Lucene syntax, never
/// inside a quoted phrase, so a plain string replace cannot cause false rewrites.
pub fn rewrite_query(query: &str, routes: &HashMap<String, Field>) -> String {
    if routes.is_empty() {
        return query.to_string();
    }
    let mut result = query.to_string();
    for path in routes.keys() {
        // path = "selectors.emails"
        let parts: Vec<&str> = path.splitn(2, '.').collect();
        if parts.len() == 2 {
            let internal = subfield_internal_name(parts[0], parts[1]);
            result = result.replace(&format!("{}:", path), &format!("{}:", internal));
        }
    }
    result
}
