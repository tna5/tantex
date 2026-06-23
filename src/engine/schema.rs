use std::collections::HashMap;

use tantivy::schema::{
    BytesOptions, DateOptions, Field, IpAddrOptions, JsonObjectOptions, NumericOptions, Schema,
    TextFieldIndexing, TextOptions, IndexRecordOption,
};
use tantivy::store::{Compressor, ZstdCompressor};
use tantivy::IndexSettings;

use crate::engine::query::subfield_internal_name;
use crate::protocol::messages::{FieldDefinition, SchemaDefinition, SubFieldDef};

/// Parse the user-supplied compression name into a tantivy Compressor.
/// Supported in tantivy 0.22: "none", "lz4" (default), "zstd", "zstd:<level>".
pub fn parse_compressor(name: &str) -> Result<Compressor, String> {
    let normalized = name.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "none" | "off" | "" => Ok(Compressor::None),
        "lz4" => Ok(Compressor::Lz4),
        "zstd" => Ok(Compressor::Zstd(ZstdCompressor::default())),
        s if s.starts_with("zstd:") => {
            let level: i32 = s[5..]
                .parse()
                .map_err(|e| format!("Invalid zstd level: {}", e))?;
            Ok(Compressor::Zstd(ZstdCompressor {
                compression_level: Some(level),
            }))
        }
        other => Err(format!(
            "Unknown compression '{}'. Use one of: none, lz4, zstd, zstd:<level>",
            other
        )),
    }
}

/// Render a Compressor back into the same name format used at create time.
pub fn compressor_name(c: &Compressor) -> String {
    match c {
        Compressor::None => "none".to_string(),
        Compressor::Lz4 => "lz4".to_string(),
        Compressor::Zstd(z) => match z.compression_level {
            Some(level) => format!("zstd:{}", level),
            None => "zstd".to_string(),
        },
    }
}

/// Build IndexSettings from the user-supplied options on the SchemaDefinition.
/// Falls back to tantivy defaults for any unset field.
pub fn build_index_settings(schema_def: &SchemaDefinition) -> Result<IndexSettings, String> {
    let mut settings = IndexSettings::default();
    if let Some(ref name) = schema_def.compression {
        settings.docstore_compression = parse_compressor(name)?;
    }
    if let Some(bs) = schema_def.block_size {
        if bs < 1024 {
            return Err(format!("block_size must be at least 1024 (got {})", bs));
        }
        settings.docstore_blocksize = bs;
    }
    Ok(settings)
}

/// Build a tantivy Schema and field name → Field mapping from our SchemaDefinition.
///
/// For json fields that carry `field_tokenizers`, the json field itself is stored
/// but not indexed; instead one internal text field per path is added to the schema
/// under the name `__sub__{json_field}__{path}`.  These internal fields are indexed
/// with the specified tokenizer and are not stored (retrieval always uses the stored
/// json field).
pub fn build_schema(
    schema_def: &SchemaDefinition,
) -> Result<(Schema, HashMap<String, Field>), Box<dyn std::error::Error>> {
    let mut builder = Schema::builder();
    let mut field_map = HashMap::new();

    for field_def in &schema_def.fields {
        if field_def.name.trim().is_empty() {
            return Err("Field name cannot be empty".into());
        }

        // Strip "array<…>" wrapper: "array<text>" → "text", "array<ip>" → "ip".
        let raw_type = field_def.field_type.as_str();
        let effective_type = if let Some(inner) = raw_type.strip_prefix("array<").and_then(|s| s.strip_suffix('>')) {
            inner
        } else {
            raw_type
        };
        let field = match effective_type {
            "text" => {
                let mut text_opts = TextOptions::default();
                if field_def.stored {
                    text_opts = text_opts.set_stored();
                }
                if field_def.indexed {
                    let indexing = TextFieldIndexing::default()
                        .set_tokenizer(&field_def.tokenizer)
                        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
                    text_opts = text_opts.set_indexing_options(indexing);
                }
                if field_def.fast {
                    text_opts = text_opts.set_fast(None);
                }
                builder.add_text_field(&field_def.name, text_opts)
            }
            "u64" => {
                let mut opts = NumericOptions::default();
                if field_def.stored {
                    opts = opts.set_stored();
                }
                if field_def.indexed {
                    opts = opts.set_indexed();
                }
                if field_def.fast {
                    opts = opts.set_fast();
                }
                builder.add_u64_field(&field_def.name, opts)
            }
            "i64" => {
                let mut opts = NumericOptions::default();
                if field_def.stored {
                    opts = opts.set_stored();
                }
                if field_def.indexed {
                    opts = opts.set_indexed();
                }
                if field_def.fast {
                    opts = opts.set_fast();
                }
                builder.add_i64_field(&field_def.name, opts)
            }
            "f64" => {
                let mut opts = NumericOptions::default();
                if field_def.stored {
                    opts = opts.set_stored();
                }
                if field_def.indexed {
                    opts = opts.set_indexed();
                }
                if field_def.fast {
                    opts = opts.set_fast();
                }
                builder.add_f64_field(&field_def.name, opts)
            }
            "date" => {
                let mut opts = DateOptions::default();
                if field_def.stored {
                    opts = opts.set_stored();
                }
                if field_def.indexed {
                    opts = opts.set_indexed();
                }
                if field_def.fast {
                    opts = opts.set_fast();
                }
                builder.add_date_field(&field_def.name, opts)
            }
            "bool" => {
                let mut opts = NumericOptions::default();
                if field_def.stored {
                    opts = opts.set_stored();
                }
                if field_def.indexed {
                    opts = opts.set_indexed();
                }
                if field_def.fast {
                    opts = opts.set_fast();
                }
                builder.add_bool_field(&field_def.name, opts)
            }
            "bytes" => {
                let mut opts = BytesOptions::default();
                if field_def.stored {
                    opts = opts.set_stored();
                }
                if field_def.indexed {
                    opts = opts.set_indexed();
                }
                if field_def.fast {
                    opts = opts.set_fast();
                }
                builder.add_bytes_field(&field_def.name, opts)
            }
            "json" => {
                // Resolve sub-field map: `fields` takes precedence over `field_tokenizers`.
                let resolved: Option<std::collections::BTreeMap<String, SubFieldDef>> =
                    if let Some(ref f) = field_def.fields {
                        Some(f.clone())
                    } else if let Some(ref ft) = field_def.field_tokenizers {
                        Some(ft.iter().map(|(k, tok)| (k.clone(), SubFieldDef {
                            field_type: "text".to_string(),
                            tokenizer: tok.clone(),
                            stored: false,
                            indexed: true,
                            fast: false,
                        })).collect())
                    } else {
                        None
                    };

                let mut opts = JsonObjectOptions::default();
                if field_def.stored {
                    opts = opts.set_stored();
                }
                // When per-path sub-fields are declared, the json field itself is not indexed.
                if field_def.indexed && resolved.is_none() {
                    let indexing = TextFieldIndexing::default()
                        .set_tokenizer(&field_def.tokenizer)
                        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
                    opts = opts.set_indexing_options(indexing);
                }
                let json_field = builder.add_json_field(&field_def.name, opts);
                field_map.insert(field_def.name.clone(), json_field);

                // Create one internal field per declared sub-path.
                if let Some(ref ft) = resolved {
                    for (path, sub_def) in ft {
                        let internal_name = subfield_internal_name(&field_def.name, path);
                        let sub_field = match sub_def.field_type.as_str() {
                            "text" => {
                                let mut text_opts = TextOptions::default();
                                if sub_def.stored { text_opts = text_opts.set_stored(); }
                                if sub_def.indexed {
                                    text_opts = text_opts.set_indexing_options(
                                        TextFieldIndexing::default()
                                            .set_tokenizer(&sub_def.tokenizer)
                                            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
                                    );
                                }
                                if sub_def.fast { text_opts = text_opts.set_fast(None); }
                                builder.add_text_field(&internal_name, text_opts)
                            }
                            "u64" => {
                                let mut n = NumericOptions::default();
                                if sub_def.stored { n = n.set_stored(); }
                                if sub_def.indexed { n = n.set_indexed(); }
                                if sub_def.fast { n = n.set_fast(); }
                                builder.add_u64_field(&internal_name, n)
                            }
                            "i64" => {
                                let mut n = NumericOptions::default();
                                if sub_def.stored { n = n.set_stored(); }
                                if sub_def.indexed { n = n.set_indexed(); }
                                if sub_def.fast { n = n.set_fast(); }
                                builder.add_i64_field(&internal_name, n)
                            }
                            "f64" => {
                                let mut n = NumericOptions::default();
                                if sub_def.stored { n = n.set_stored(); }
                                if sub_def.indexed { n = n.set_indexed(); }
                                if sub_def.fast { n = n.set_fast(); }
                                builder.add_f64_field(&internal_name, n)
                            }
                            "date" => {
                                let mut d = DateOptions::default();
                                if sub_def.stored { d = d.set_stored(); }
                                if sub_def.indexed { d = d.set_indexed(); }
                                if sub_def.fast { d = d.set_fast(); }
                                builder.add_date_field(&internal_name, d)
                            }
                            "bool" => {
                                let mut n = NumericOptions::default();
                                if sub_def.stored { n = n.set_stored(); }
                                if sub_def.indexed { n = n.set_indexed(); }
                                if sub_def.fast { n = n.set_fast(); }
                                builder.add_bool_field(&internal_name, n)
                            }
                            "ip" => {
                                let mut ip = IpAddrOptions::default();
                                if sub_def.stored { ip = ip.set_stored(); }
                                if sub_def.indexed { ip = ip.set_indexed(); }
                                if sub_def.fast { ip = ip.set_fast(); }
                                builder.add_ip_addr_field(&internal_name, ip)
                            }
                            other => return Err(format!(
                                "Unknown sub-field type '{}' on field '{}' path '{}'",
                                other, field_def.name, path
                            ).into()),
                        };
                        field_map.insert(internal_name, sub_field);
                    }
                }
                continue;
            }
            "ip" => {
                let mut opts = IpAddrOptions::default();
                if field_def.stored {
                    opts = opts.set_stored();
                }
                if field_def.indexed {
                    opts = opts.set_indexed();
                }
                if field_def.fast {
                    opts = opts.set_fast();
                }
                builder.add_ip_addr_field(&field_def.name, opts)
            }
            other => {
                return Err(format!("Unknown field type: {}", other).into());
            }
        };
        field_map.insert(field_def.name.clone(), field);
    }

    Ok((builder.build(), field_map))
}

/// Reconstruct a SchemaDefinition from a tantivy Schema and the IndexSettings
/// stored alongside it. Pass `None` for `settings` when only the fields matter.
///
/// Internal `__sub__*` fields are stripped; their definitions are re-attached
/// to the owning json field's `fields` map.
pub fn schema_to_definition(schema: &Schema, settings: Option<&IndexSettings>) -> SchemaDefinition {
    use tantivy::schema::FieldType;
    use std::collections::BTreeMap;

    // First pass: collect sub-field definitions from internal __sub__ fields.
    let mut json_ft: HashMap<String, BTreeMap<String, SubFieldDef>> = HashMap::new();
    for (_, entry) in schema.fields() {
        let name = entry.name();
        if let Some(rest) = name.strip_prefix("__sub__") {
            if let Some(sep) = rest.find("__") {
                let json_field = &rest[..sep];
                let path = &rest[sep + 2..];
                let sub_def = match entry.field_type() {
                    FieldType::Str(opts) => SubFieldDef {
                        field_type: "text".to_string(),
                        tokenizer: opts.get_indexing_options()
                            .map(|i| i.tokenizer().to_string())
                            .unwrap_or_else(|| "default".to_string()),
                        stored: opts.is_stored(),
                        indexed: opts.get_indexing_options().is_some(),
                        fast: opts.is_fast(),
                    },
                    FieldType::U64(opts) => SubFieldDef {
                        field_type: "u64".to_string(), tokenizer: "default".to_string(),
                        stored: opts.is_stored(), indexed: opts.is_indexed(), fast: opts.is_fast(),
                    },
                    FieldType::I64(opts) => SubFieldDef {
                        field_type: "i64".to_string(), tokenizer: "default".to_string(),
                        stored: opts.is_stored(), indexed: opts.is_indexed(), fast: opts.is_fast(),
                    },
                    FieldType::F64(opts) => SubFieldDef {
                        field_type: "f64".to_string(), tokenizer: "default".to_string(),
                        stored: opts.is_stored(), indexed: opts.is_indexed(), fast: opts.is_fast(),
                    },
                    FieldType::Date(opts) => SubFieldDef {
                        field_type: "date".to_string(), tokenizer: "default".to_string(),
                        stored: opts.is_stored(), indexed: opts.is_indexed(), fast: opts.is_fast(),
                    },
                    FieldType::Bool(opts) => SubFieldDef {
                        field_type: "bool".to_string(), tokenizer: "default".to_string(),
                        stored: opts.is_stored(), indexed: opts.is_indexed(), fast: opts.is_fast(),
                    },
                    FieldType::IpAddr(opts) => SubFieldDef {
                        field_type: "ip".to_string(), tokenizer: "default".to_string(),
                        stored: opts.is_stored(), indexed: opts.is_indexed(), fast: opts.is_fast(),
                    },
                    _ => continue,
                };
                json_ft.entry(json_field.to_string()).or_default().insert(path.to_string(), sub_def);
            }
        }
    }

    // Second pass: build the public field list, skipping internal fields.
    let mut fields = Vec::new();
    for (_, field_entry) in schema.fields() {
        let name = field_entry.name().to_string();
        if name.starts_with("__sub__") {
            continue;
        }

        let (field_type, stored, indexed, fast, tokenizer) = match field_entry.field_type() {
            FieldType::Str(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.get_indexing_options().is_some();
                let fast = opts.is_fast();
                let tokenizer = opts
                    .get_indexing_options()
                    .map(|i| i.tokenizer().to_string())
                    .unwrap_or_else(|| "default".to_string());
                ("text".to_string(), stored, indexed, fast, tokenizer)
            }
            FieldType::U64(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.is_indexed();
                let fast = opts.is_fast();
                ("u64".to_string(), stored, indexed, fast, "default".to_string())
            }
            FieldType::I64(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.is_indexed();
                let fast = opts.is_fast();
                ("i64".to_string(), stored, indexed, fast, "default".to_string())
            }
            FieldType::F64(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.is_indexed();
                let fast = opts.is_fast();
                ("f64".to_string(), stored, indexed, fast, "default".to_string())
            }
            FieldType::Date(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.is_indexed();
                let fast = opts.is_fast();
                ("date".to_string(), stored, indexed, fast, "default".to_string())
            }
            FieldType::Bool(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.is_indexed();
                let fast = opts.is_fast();
                ("bool".to_string(), stored, indexed, fast, "default".to_string())
            }
            FieldType::Bytes(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.is_indexed();
                let fast = opts.is_fast();
                ("bytes".to_string(), stored, indexed, fast, "default".to_string())
            }
            FieldType::JsonObject(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.get_text_indexing_options().is_some();
                let tokenizer = opts
                    .get_text_indexing_options()
                    .map(|i| i.tokenizer().to_string())
                    .unwrap_or_else(|| "default".to_string());
                ("json".to_string(), stored, indexed, false, tokenizer)
            }
            FieldType::IpAddr(opts) => {
                let stored = opts.is_stored();
                let indexed = opts.is_indexed();
                let fast = opts.is_fast();
                ("ip".to_string(), stored, indexed, fast, "default".to_string())
            }
            _ => ("unknown".to_string(), false, false, false, "default".to_string()),
        };

        let fields_map = if field_type == "json" {
            json_ft.remove(&name).filter(|m| !m.is_empty())
        } else {
            None
        };

        fields.push(FieldDefinition {
            name,
            field_type,
            stored,
            indexed,
            fast,
            tokenizer,
            fields: fields_map,
            field_tokenizers: None,
        });
    }

    let (compression, block_size) = match settings {
        Some(s) => (
            Some(compressor_name(&s.docstore_compression)),
            Some(s.docstore_blocksize),
        ),
        None => (None, None),
    };

    SchemaDefinition {
        fields,
        compression,
        block_size,
    }
}
