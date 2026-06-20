# Query Syntax Reference

Tantex uses [tantivy](https://github.com/quickwit-oss/tantivy)'s query parser. The default search fields are all `text` and `json` fields in the schema.

---

## Term queries

Match documents that contain a specific term in any default field:

```
rust
```

Match in a specific field:

```
title:rust
```

Terms are lowercased and tokenized before matching. Use the `raw` tokenizer for exact-match fields.

---

## Phrase queries

Match documents where the terms appear adjacent and in order:

```
"full text search"
title:"search engine"
```

---

## Boolean operators

| Operator | Syntax | Meaning |
|---|---|---|
| AND | `term1 AND term2` or `+term1 +term2` | Both terms must match |
| OR | `term1 OR term2` or `term1 term2` | At least one term must match |
| NOT | `NOT term` or `-term` | Term must not match |

Examples:

```
rust AND performance
rust OR golang
+rust -python
title:rust AND body:performance
```

When multiple terms appear without an explicit operator, the default is **OR**.

---

## Wildcard queries

**All documents:**

```
*
```

**Prefix matching:**

```
rust*
```

Matches `rust`, `rustacean`, `rustlang`, etc.

> Wildcard queries can be slow on large indexes. Prefer exact terms where possible.

---

## Range queries

### Numeric ranges

```
views:[100 TO 1000]      // inclusive
views:{100 TO 1000}      // exclusive
views:[100 TO *]         // 100 or more
views:[* TO 1000]        // 1000 or less
score:[0.5 TO 1.0]       // f64 range
```

### Date ranges

Dates must be in RFC 3339 format:

```
published_at:[2024-01-01T00:00:00Z TO 2024-12-31T23:59:59Z]
published_at:[2024-06-01T00:00:00Z TO *]
```

Numeric and date range queries require the field to have `fast: true` in the schema.

---

## Field-specific queries

Prefix any term, phrase, or range with `field_name:` to restrict the query to that field:

```
author:alice
title:"search engine"
body:performance
metadata.color:blue
```

For `json` fields, use dot notation to address nested keys: `metadata.color:blue`.

---

## Combining queries

```
+title:"full text" +body:performance -language:python
(title:rust OR title:golang) AND body:performance
```

---

## Escaping special characters

The following characters have special meaning in the query syntax and must be escaped with a backslash when used as literals:

```
+ - && || ! ( ) { } [ ] ^ " ~ * ? : \ /
```

Example — searching for the literal string `C++`:

```
title:C\+\+
```
