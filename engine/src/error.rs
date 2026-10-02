//! The one error shape the experiment engine reports. It carries a stable code
//! and values, never a sentence: the interface turns `err.<code>` into text in
//! the user's language, so switching language re-renders every message and a
//! saved report can be read in either. See `docs/milestone-4-reactive.md`.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use super::secrets;

/// Which field of a node a problem is about; the interface shows `field.<key>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    pub key: String,
    /// 1-based position for repeated fields ("header 2", "argument 3").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
}

impl Field {
    pub fn new(key: &str) -> Self {
        Field { key: key.into(), index: None }
    }

    pub fn nth(key: &str, index: usize) -> Self {
        Field { key: key.into(), index: Some(index) }
    }
}

/// What went wrong and where. Read through `EngineError`, which boxes it so a
/// `Result` carrying one stays a pointer wide on the paths that succeed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ErrorData {
    /// Stable identifier, also the translation key suffix.
    pub code: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<Field>,
    /// Technical text from the operating system, a parser or a library.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EngineError(Box<ErrorData>);

pub type EngineResult<T> = Result<T, EngineError>;

impl Deref for EngineError {
    type Target = ErrorData;
    fn deref(&self) -> &ErrorData {
        &self.0
    }
}

impl DerefMut for EngineError {
    fn deref_mut(&mut self) -> &mut ErrorData {
        &mut self.0
    }
}

impl EngineError {
    pub fn new(code: &str) -> Self {
        EngineError(Box::new(ErrorData { code: code.into(), params: BTreeMap::new(), node: None, field: None, detail: None }))
    }

    /// The code, for callers that keep only it.
    pub fn into_code(self) -> String {
        self.0.code
    }

    pub fn with(mut self, name: &str, value: impl ToString) -> Self {
        self.params.insert(name.into(), value.to_string());
        self
    }

    /// The node it is about; an inner error that already names one keeps it.
    pub fn at(mut self, node: &str) -> Self {
        if self.node.is_none() {
            self.node = Some(node.into());
        }
        self
    }

    /// The field it is about; an inner error that already names one keeps it.
    pub fn in_field(mut self, field: Field) -> Self {
        if self.field.is_none() {
            self.field = Some(field);
        }
        self
    }

    pub fn because(mut self, detail: impl ToString) -> Self {
        let detail = detail.to_string();
        if !detail.is_empty() {
            self.detail = Some(detail);
        }
        self
    }

    /// The same error with secret values replaced by the mask.
    pub fn masked(mut self, values: &[String]) -> Self {
        if values.is_empty() {
            return self;
        }
        for value in self.params.values_mut() {
            *value = secrets::mask(value, values);
        }
        self.detail = self.detail.take().map(|detail| secrets::mask(&detail, values));
        self
    }

    pub fn is(&self, code: &str) -> bool {
        self.code == code
    }
}

/// For logs and test messages only; people see the translated text.
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(node) = &self.node {
            write!(f, "{node}: ")?;
        }
        if let Some(field) = &self.field {
            match field.index {
                Some(index) => write!(f, "{} {index}: ", field.key)?,
                None => write!(f, "{}: ", field.key)?,
            }
        }
        write!(f, "{}", self.code)?;
        if !self.params.is_empty() {
            let params: Vec<String> = self.params.iter().map(|(name, value)| format!("{name}={value}")).collect();
            write!(f, " {{{}}}", params.join(", "))?;
        }
        if let Some(detail) = &self.detail {
            write!(f, " ({detail})")?;
        }
        Ok(())
    }
}

impl std::error::Error for EngineError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builders_keep_the_innermost_location_and_serialize_compactly() {
        let error = EngineError::new("name.unknown")
            .with("name", "token")
            .in_field(Field::nth("header_value", 2))
            .at("login")
            .at("outer")
            .in_field(Field::new("url"))
            .because("");
        assert_eq!(error.node.as_deref(), Some("login"));
        assert_eq!(error.field, Some(Field::nth("header_value", 2)));
        assert_eq!(error.detail, None, "an empty detail is no detail");
        assert_eq!(
            serde_json::to_value(&error).unwrap(),
            serde_json::json!({ "code": "name.unknown", "params": { "name": "token" }, "node": "login",
                                "field": { "key": "header_value", "index": 2 } })
        );
        assert_eq!(serde_json::to_value(EngineError::new("graph.unreachable")).unwrap(), serde_json::json!({ "code": "graph.unreachable" }));
        assert_eq!(error.to_string(), "login: header_value 2: name.unknown {name=token}");
        let round: EngineError = serde_json::from_value(serde_json::to_value(&error).unwrap()).unwrap();
        assert_eq!(round, error);
    }

    /// The interface shows `err.<code>` and `field.<key>`; a code without a
    /// text would reach the user as a bare identifier. `ru.ts` is typed
    /// against `en.ts`, so English coverage is translation coverage.
    #[test]
    fn every_code_and_field_the_engine_uses_has_a_text() {
        use std::collections::BTreeSet;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let en = std::fs::read_to_string(root.join("../src/lib/locales/en.ts")).unwrap();
        let code = regex::Regex::new(r#"EngineError::new\("([a-z0-9_.]+)"\)"#).unwrap();
        let field = regex::Regex::new(r#"Field::(?:new|nth)\("([a-z0-9_]+)""#).unwrap();
        let (mut codes, mut fields) = (BTreeSet::new(), BTreeSet::new());
        // The engine, the server and the command line report EngineErrors; all are scanned.
        for folder in [root.join("src"), root.join("../server/src"), root.join("../cli/src")] {
            for entry in std::fs::read_dir(&folder).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_some_and(|extension| extension == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    codes.extend(code.captures_iter(&text).map(|found| found[1].to_string()));
                    fields.extend(field.captures_iter(&text).map(|found| found[1].to_string()));
                }
            }
        }
        assert!(codes.len() > 100 && fields.len() > 20, "the scan found the engine's codes");
        let missing: Vec<String> = codes
            .iter()
            .map(|code| format!("err.{code}"))
            .chain(fields.iter().map(|key| format!("field.{key}")))
            .filter(|key| !en.contains(&format!("\"{key}\":")))
            .collect();
        assert!(missing.is_empty(), "no text in en.ts for: {missing:?}");

        // And no text for a code that no longer exists; these four are the renderer's own.
        let own = ["unknown", "position", "show", "details"];
        let key = regex::Regex::new(r#""err\.([a-z0-9_.]+)":"#).unwrap();
        let stale: Vec<&str> = key
            .captures_iter(&en)
            .map(|found| found.get(1).unwrap().as_str())
            .filter(|key| !codes.contains(*key) && !own.contains(key))
            .collect();
        assert!(stale.is_empty(), "en.ts has texts for codes the engine no longer uses: {stale:?}");
    }

    #[test]
    fn masking_reaches_params_and_detail() {
        let error = EngineError::new("transport.refused").with("target", "http://x/?key=s3cret").because("refused s3cret");
        let masked = error.masked(&["s3cret".into()]);
        assert_eq!(masked.params["target"], format!("http://x/?key={}", secrets::MASK));
        assert_eq!(masked.detail.as_deref(), Some(format!("refused {}", secrets::MASK).as_str()));
    }
}
