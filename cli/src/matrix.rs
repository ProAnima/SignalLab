//! The parameter matrix of `signallab run` and `validate`: one experiment run
//! once per combination of values — every target, every credential, every
//! payload size — each a run of its own with its own report, checked all
//! before any of them sends anything.
//!
//! `--matrix NAME=V1,V2` adds an axis (the same name again adds values to it;
//! names and values are trimmed); `--matrix-file` adds axes (`{"NAME": [values]}`,
//! by name in alphabetical order, after the flags') or a list of combinations
//! (`[{"NAME": value, …}]`, each crossed with the flags' axes). Axes multiply,
//! the first one varying slowest. A value given twice, or a combination listed
//! twice, is one run: the count that is bounded is the runs that would start.

use std::path::Path;

use serde_json::Value;
use signal_lab_engine::error::EngineError;

use crate::fail::Failure;

/// One combination: a value per name, in the order the names were given.
pub type Combination = Vec<(String, String)>;

/// More runs than a pipeline should start from one command.
pub const MAX_COMBINATIONS: usize = 256;

/// `values` added to `into`, each once, in the order first given.
fn add_values(into: &mut Vec<String>, values: impl IntoIterator<Item = String>) {
    for value in values {
        if !into.contains(&value) {
            into.push(value);
        }
    }
}

/// A value as a parameter takes it: text; numbers and booleans as written.
fn text_of(name: &str, value: &Value) -> Result<String, Failure> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Number(number) => Ok(number.to_string()),
        Value::Bool(flag) => Ok(flag.to_string()),
        _ => Err(Failure::invalid(EngineError::new("cli.matrix_value_invalid").with("name", name))),
    }
}

/// Axes from `NAME=V1,V2` flags: the same name again adds values.
fn axes_of_flags(flags: &[String]) -> Result<Vec<(String, Vec<String>)>, Failure> {
    let mut axes: Vec<(String, Vec<String>)> = Vec::new();
    for flag in flags {
        let Some((name, values)) = flag.split_once('=').filter(|(name, _)| !name.trim().is_empty()) else {
            return Err(Failure::invalid(EngineError::new("cli.matrix_invalid").with("value", flag)));
        };
        // `host = a, b` as a shell or a YAML block writes it: the spaces are not part of the values.
        let values = values.split(',').map(|value| value.trim().to_string());
        let name = name.trim();
        match axes.iter_mut().find(|(known, _)| known == name) {
            Some((_, known)) => add_values(known, values),
            None => {
                let mut fresh = Vec::new();
                add_values(&mut fresh, values);
                axes.push((name.to_string(), fresh));
            }
        }
    }
    Ok(axes)
}

/// What a matrix file holds: axes, or the combinations themselves.
enum FromFile {
    Axes(Vec<(String, Vec<String>)>),
    List(Vec<Combination>),
}

fn read_file(path: &Path) -> Result<FromFile, Failure> {
    let invalid = |detail: String| Failure::invalid(EngineError::new("cli.matrix_file_invalid").with("path", path.display()).because(detail));
    let text = std::fs::read_to_string(path).map_err(|error| Failure::invalid(EngineError::new("file.io").with("path", path.display()).because(error)))?;
    // serde_json's maps keep their keys sorted (no `preserve_order`): a file's
    // axes, and a listed combination's values, come by name.
    match serde_json::from_str::<Value>(&text).map_err(|error| invalid(error.to_string()))? {
        Value::Object(axes) => axes
            .into_iter()
            .map(|(name, values)| match values {
                Value::Array(values) if !values.is_empty() => {
                    let mut unique = Vec::new();
                    add_values(&mut unique, values.iter().map(|value| text_of(&name, value)).collect::<Result<Vec<_>, _>>()?);
                    Ok((name, unique))
                }
                _ => Err(invalid(format!("{name}: a list of values"))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(FromFile::Axes),
        Value::Array(list) if !list.is_empty() => {
            let mut unique: Vec<Combination> = Vec::new();
            for entry in &list {
                let combination = match entry {
                    Value::Object(values) if !values.is_empty() => values.iter().map(|(name, value)| Ok((name.clone(), text_of(name, value)?))).collect::<Result<Combination, Failure>>()?,
                    _ => return Err(invalid("each entry: an object of NAME: value".into())),
                };
                // Sorted by name, so the same values listed twice compare equal.
                if !unique.contains(&combination) {
                    unique.push(combination);
                }
            }
            Ok(FromFile::List(unique))
        }
        _ => Err(invalid("an object of NAME: [values], or a list of objects".into())),
    }
}

/// Every combination to run; one empty combination when there is no matrix.
pub fn combinations(flags: &[String], file: Option<&Path>) -> Result<Vec<Combination>, Failure> {
    let mut axes = axes_of_flags(flags)?;
    let mut base: Vec<Combination> = vec![Vec::new()];
    if let Some(path) = file {
        match read_file(path)? {
            FromFile::Axes(more) => {
                for (name, values) in more {
                    if axes.iter().any(|(known, _)| known == &name) {
                        return Err(Failure::invalid(EngineError::new("cli.matrix_conflict").with("name", name)));
                    }
                    axes.push((name, values));
                }
            }
            FromFile::List(list) => {
                if let Some((name, _)) = list.iter().flatten().find(|(name, _)| axes.iter().any(|(known, _)| known == name)) {
                    return Err(Failure::invalid(EngineError::new("cli.matrix_conflict").with("name", name)));
                }
                base = list;
            }
        }
    }
    // Counted before anything is built: the values are distinct and each axis
    // has a name of its own, so this is the number of runs that would start.
    let Some(count) = axes.iter().try_fold(base.len() as u128, |count, (_, values)| count.checked_mul(values.len() as u128)) else {
        return Err(Failure::invalid(EngineError::new("cli.matrix_too_wide").with("axes", axes.len()).with("max", MAX_COMBINATIONS)));
    };
    if count > MAX_COMBINATIONS as u128 {
        return Err(Failure::invalid(EngineError::new("cli.matrix_too_large").with("n", count).with("max", MAX_COMBINATIONS)));
    }
    // The first axis varies slowest; a listed combination comes before the axes.
    let mut all = base;
    for (name, values) in &axes {
        all = all.into_iter().flat_map(|combination| values.iter().map(move |value| {
            let mut next = combination.clone();
            next.push((name.clone(), value.clone()));
            next
        })).collect();
    }
    Ok(all)
}

/// Every name the matrix sets.
pub fn names(combinations: &[Combination]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for (name, _) in combinations.iter().flatten() {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    names
}

/// `file [device=10.0.0.20:9000, who=lab]`: how a combination's run is named.
pub fn label(base: &str, combination: &Combination) -> String {
    if combination.is_empty() {
        return base.to_string();
    }
    let values: Vec<String> = combination.iter().map(|(name, value)| format!("{name}={value}")).collect();
    format!("{base} [{}]", values.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(given: &[&str]) -> Vec<String> {
        given.iter().map(|flag| flag.to_string()).collect()
    }

    fn pairs(combination: &Combination) -> Vec<String> {
        combination.iter().map(|(name, value)| format!("{name}={value}")).collect()
    }

    #[test]
    fn axes_multiply_the_first_varying_slowest() {
        let all = combinations(&flags(&["host=a,b", "size=1,10,100"]), None).unwrap();
        assert_eq!(all.len(), 6);
        assert_eq!(pairs(&all[0]), ["host=a", "size=1"]);
        assert_eq!(pairs(&all[2]), ["host=a", "size=100"]);
        assert_eq!(pairs(&all[3]), ["host=b", "size=1"]);
        // The same name again adds values; a value may be empty, or hold "=".
        let more = combinations(&flags(&["q=x=1", "q=,y"]), None).unwrap();
        assert_eq!(more.iter().map(|combination| combination[0].1.clone()).collect::<Vec<_>>(), ["x=1", "", "y"]);
        // Spaces around names and values go; a value given twice is one run.
        let spaced = combinations(&flags(&[" host = a , b,a ", "host=b, c\r"]), None).unwrap();
        assert_eq!(spaced.iter().map(pairs).collect::<Vec<_>>(), [["host=a"], ["host=b"], ["host=c"]]);
        assert_eq!(combinations(&[], None).unwrap(), vec![Vec::new()], "no matrix: one run, as before");
        assert_eq!(label("smoke.json", &all[4]), "smoke.json [host=b, size=10]");
        assert_eq!(label("smoke.json", &Vec::new()), "smoke.json");
        assert_eq!(names(&all), ["host", "size"]);
    }

    #[test]
    fn a_file_adds_axes_or_lists_combinations() {
        let dir = std::env::temp_dir().join(format!("signallab-matrix-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let axes = dir.join("axes.json");
        std::fs::write(&axes, r#"{"user": ["admin", "guest"], "retries": [1, 3], "tls": [true]}"#).unwrap();
        let all = combinations(&flags(&["host=a,b"]), Some(&axes)).unwrap();
        assert_eq!(all.len(), 8);
        assert_eq!(pairs(&all[0]), ["host=a", "retries=1", "tls=true", "user=admin"], "the flags' axes, then the file's by name");
        assert_eq!(pairs(&all[1]), ["host=a", "retries=1", "tls=true", "user=guest"], "the last varies fastest");
        let twice = dir.join("twice.json");
        std::fs::write(&twice, r#"{"user": ["admin", "admin", "guest"]}"#).unwrap();
        assert_eq!(combinations(&[], Some(&twice)).unwrap().len(), 2, "a value listed twice is one run");
        let list = dir.join("list.json");
        std::fs::write(&list, r#"[{"user": "admin", "pass": "x,y"}, {"user": "guest", "pass": ""}, {"pass": "x,y", "user": "admin"}]"#).unwrap();
        let listed = combinations(&flags(&["host=a,b"]), Some(&list)).unwrap();
        assert_eq!(listed.iter().map(pairs).collect::<Vec<_>>(), [
            vec!["pass=x,y", "user=admin", "host=a"], vec!["pass=x,y", "user=admin", "host=b"],
            vec!["pass=", "user=guest", "host=a"], vec!["pass=", "user=guest", "host=b"],
        ], "the list in its order, once each, crossed with the flags; a comma inside a listed value stays");
        let code = |text: &str, given: &[&str]| {
            let path = dir.join("bad.json");
            std::fs::write(&path, text).unwrap();
            combinations(&flags(given), Some(&path)).unwrap_err().error.code.clone()
        };
        assert_eq!(code("{nope", &[]), "cli.matrix_file_invalid");
        assert_eq!(code(r#"{"a": []}"#, &[]), "cli.matrix_file_invalid");
        assert_eq!(code(r#"{"a": [null]}"#, &[]), "cli.matrix_value_invalid");
        assert_eq!(code(r#"{"host": ["x"]}"#, &["host=a"]), "cli.matrix_conflict");
        assert_eq!(code("42", &[]), "cli.matrix_file_invalid");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_matrix_is_bounded_and_written_right() {
        let wide = (0..9).map(|i| format!("p{i}=a,b")).collect::<Vec<_>>();
        let failure = combinations(&wide, None).unwrap_err();
        assert_eq!((failure.error.code.as_str(), failure.error.params["n"].as_str()), ("cli.matrix_too_large", "512"));
        // Duplicates do not count: 300 copies of one value are one run.
        let same = format!("n={}", vec!["1"; 300].join(","));
        assert_eq!(combinations(&[same], None).unwrap().len(), 1);
        // So many axes that the product cannot be counted: refused, never built.
        let endless: Vec<String> = (0..140).map(|i| format!("p{i}=a,b")).collect();
        let failure = combinations(&endless, None).unwrap_err();
        assert_eq!((failure.error.code.as_str(), failure.error.params["axes"].as_str()), ("cli.matrix_too_wide", "140"));
        for bad in ["novalue", "=x"] {
            assert_eq!(combinations(&flags(&[bad]), None).unwrap_err().error.code, "cli.matrix_invalid", "{bad}");
        }
    }
}
