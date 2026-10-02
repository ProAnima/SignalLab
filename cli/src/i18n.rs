//! Messages in the reader's language, from the interface's own dictionaries
//! (embedded by `build.rs`), filled in the way `src/lib/translate.ts` does it:
//! `{name}` is the value as given; `{n, plural, one {…} few {…} other {…}}`
//! picks a branch by the language's plural rules (`=0 {…}` first) with `#` the
//! number as the language writes it; `{v, select, true {…} other {…}}` picks
//! by the value. Failures read as `src/lib/errors.ts` words them:
//! *Node · Field — message*, the system's own text kept apart as the detail.

use std::collections::{BTreeMap, HashMap};

use serde_json::Value;
use signal_lab_engine::error::EngineError;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}
pub use embedded::TEMPLATES;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Lang {
    En,
    Ru,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }

    /// A language tag or locale (`ru`, `ru-RU`, `ru_RU.UTF-8`), by its base language.
    fn from_tag(tag: &str) -> Option<Lang> {
        let base: String = tag.trim().chars().take_while(|char| char.is_ascii_alphabetic()).collect::<String>().to_ascii_lowercase();
        match base.as_str() {
            "en" => Some(Lang::En),
            "ru" => Some(Lang::Ru),
            _ => None,
        }
    }

    /// `--lang`, else `SIGNALLAB_LANG`, else the locale (`LC_ALL`, `LC_MESSAGES`,
    /// `LANG`, the first one set), else English.
    pub fn choose(flag: Option<Lang>, env: impl Fn(&str) -> Option<String>) -> Lang {
        if let Some(lang) = flag {
            return lang;
        }
        if let Some(lang) = env("SIGNALLAB_LANG").and_then(|tag| Lang::from_tag(&tag)) {
            return lang;
        }
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|name| env(name).filter(|value| !value.is_empty()));
        locale.and_then(|tag| Lang::from_tag(&tag)).unwrap_or(Lang::En)
    }
}

/// A value of a message: text as it is, or a number.
#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    Text(String),
    Int(i128),
    Float(f64),
}

impl Arg {
    /// As JavaScript's `String(value)` writes it.
    fn shown(&self) -> String {
        match self {
            Arg::Text(text) => text.clone(),
            Arg::Int(number) => number.to_string(),
            Arg::Float(number) => js_number(*number),
        }
    }

    /// As JavaScript's `Number(value)` reads it; `None` when that is not a finite number.
    fn number(&self) -> Option<f64> {
        let number = match self {
            Arg::Text(text) if text.trim().is_empty() => 0.0,
            Arg::Text(text) => text.trim().parse::<f64>().ok()?,
            Arg::Int(number) => *number as f64,
            Arg::Float(number) => *number,
        };
        number.is_finite().then_some(number)
    }
}

impl From<&str> for Arg {
    fn from(text: &str) -> Self {
        Arg::Text(text.to_string())
    }
}

impl From<String> for Arg {
    fn from(text: String) -> Self {
        Arg::Text(text)
    }
}

impl From<u64> for Arg {
    fn from(number: u64) -> Self {
        Arg::Int(number as i128)
    }
}

impl From<usize> for Arg {
    fn from(number: usize) -> Self {
        Arg::Int(number as i128)
    }
}

impl From<f64> for Arg {
    fn from(number: f64) -> Self {
        Arg::Float(number)
    }
}

impl From<&Value> for Arg {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(text) => Arg::Text(text.clone()),
            Value::Number(number) => match (number.as_i64(), number.as_u64()) {
                (Some(int), _) => Arg::Int(int as i128),
                (_, Some(int)) => Arg::Int(int as i128),
                _ => Arg::Float(number.as_f64().unwrap_or(f64::NAN)),
            },
            Value::Bool(flag) => Arg::Text(flag.to_string()),
            Value::Null => Arg::Text("null".into()),
            other => Arg::Text(other.to_string()),
        }
    }
}

pub type Params = BTreeMap<String, Arg>;

/// `params!{ "name" => value, … }`
#[macro_export]
macro_rules! params {
    ($($name:expr => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut params = $crate::i18n::Params::new();
        $(params.insert($name.to_string(), $crate::i18n::Arg::from($value));)*
        params
    }};
}

/// JavaScript's shortest decimal for a number (no exponent: Signal Lab's values never need one).
fn js_number(number: f64) -> String {
    if number.is_nan() {
        return "NaN".into();
    }
    if number.is_infinite() {
        return if number > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if number == 0.0 {
        return "0".into();
    }
    number.to_string()
}

/// A number as the language writes it, at most three decimals, rounded half
/// away from zero on its shortest decimal form — `Intl.NumberFormat(lang,
/// { maximumFractionDigits: 3 })`.
pub fn format_number(number: f64, lang: Lang) -> String {
    if !number.is_finite() {
        return js_number(number);
    }
    let negative = number < 0.0;
    let digits = js_number(number.abs());
    let (int_part, frac_part) = digits.split_once('.').unwrap_or((&digits, ""));
    let mut int_digits: Vec<u8> = int_part.bytes().map(|byte| byte - b'0').collect();
    let mut frac: Vec<u8> = frac_part.bytes().map(|byte| byte - b'0').collect();
    if frac.len() > 3 {
        let round_up = frac[3] >= 5;
        frac.truncate(3);
        if round_up {
            // Carry through the fraction, then the integer part.
            let mut carry = true;
            for digit in frac.iter_mut().rev().chain(int_digits.iter_mut().rev()) {
                if !carry {
                    break;
                }
                if *digit == 9 {
                    *digit = 0;
                } else {
                    *digit += 1;
                    carry = false;
                }
            }
            if carry {
                int_digits.insert(0, 1);
            }
        }
    }
    while frac.last() == Some(&0) {
        frac.pop();
    }
    let (group, decimal) = match lang {
        Lang::En => (",", "."),
        Lang::Ru => ("\u{a0}", ","),
    };
    let mut out = String::new();
    let count = int_digits.len();
    for (at, digit) in int_digits.iter().enumerate() {
        if at > 0 && (count - at).is_multiple_of(3) {
            out.push_str(group);
        }
        out.push((b'0' + digit) as char);
    }
    if !frac.is_empty() {
        out.push_str(decimal);
        out.extend(frac.iter().map(|digit| (b'0' + digit) as char));
    }
    let zero = int_digits.iter().all(|digit| *digit == 0) && frac.is_empty();
    if negative && !zero {
        out.insert(0, '-');
    }
    out
}

/// The CLDR plural category of `number` — what `Intl.PluralRules(lang)` selects.
pub fn plural_category(number: f64, lang: Lang) -> &'static str {
    let n = number.abs();
    let integer = n.fract() == 0.0;
    match lang {
        Lang::En => {
            if n == 1.0 {
                "one"
            } else {
                "other"
            }
        }
        Lang::Ru => {
            if !integer {
                return "other";
            }
            let i = (n % 1_000_000_000.0) as u64;
            let (last, last_two) = (i % 10, i % 100);
            if last == 1 && last_two != 11 {
                "one"
            } else if (2..=4).contains(&last) && !(12..=14).contains(&last_two) {
                "few"
            } else {
                "many"
            }
        }
    }
}

/// A `{name, plural|select, …}` placeholder: its value, kind and branches.
struct Plural {
    name: String,
    select: bool,
    branches: Vec<(String, String)>,
    /// Byte offset just past its closing brace.
    end: usize,
}

impl Plural {
    fn branch(&self, selector: &str) -> Option<&str> {
        self.branches.iter().rev().find(|(name, _)| name == selector).map(|(_, text)| text.as_str())
    }
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// `{name}` at the start of `text`: the name and the length taken.
fn simple(text: &str) -> Option<(&str, usize)> {
    let bytes = text.as_bytes();
    let name_len = bytes.iter().skip(1).take_while(|byte| is_word(**byte)).count();
    (bytes.first() == Some(&b'{') && name_len > 0 && bytes.get(1 + name_len) == Some(&b'}')).then(|| (&text[1..1 + name_len], name_len + 2))
}

fn skip_space(text: &str, mut at: usize) -> usize {
    while let Some(char) = text[at..].chars().next().filter(|char| char.is_whitespace()) {
        at += char.len_utf8();
    }
    at
}

/// The plural or select placeholder starting at `from` (on its `{`), as `parsePlural` reads it.
fn parse_plural(text: &str, from: usize) -> Option<Plural> {
    let bytes = text.as_bytes();
    if bytes.get(from) != Some(&b'{') {
        return None;
    }
    let name_start = from + 1;
    let name_len = bytes[name_start..].iter().take_while(|byte| is_word(**byte)).count();
    if name_len == 0 {
        return None;
    }
    let name = &text[name_start..name_start + name_len];
    let mut at = skip_space(text, name_start + name_len);
    if bytes.get(at) != Some(&b',') {
        return None;
    }
    at = skip_space(text, at + 1);
    let select = if text[at..].starts_with("plural") {
        false
    } else if text[at..].starts_with("select") {
        true
    } else {
        return None;
    };
    at = skip_space(text, at + 6);
    if bytes.get(at) != Some(&b',') {
        return None;
    }
    at += 1;
    let mut branches = Vec::new();
    loop {
        at = skip_space(text, at);
        match bytes.get(at) {
            Some(b'}') => return (!branches.is_empty()).then(|| Plural { name: name.to_string(), select, branches, end: at + 1 }),
            None => return None,
            _ => {}
        }
        // `=\d+` or `[A-Za-z0-9_-]+`, then optional space and `{`.
        let selector_len = if bytes[at] == b'=' {
            let digits = bytes[at + 1..].iter().take_while(|byte| byte.is_ascii_digit()).count();
            if digits == 0 {
                return None;
            }
            digits + 1
        } else {
            bytes[at..].iter().take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_' || **byte == b'-').count()
        };
        if selector_len == 0 {
            return None;
        }
        let selector = &text[at..at + selector_len];
        at = skip_space(text, at + selector_len);
        if bytes.get(at) != Some(&b'{') {
            return None;
        }
        let start = at + 1;
        let mut depth = 1;
        let mut index = start;
        while index < bytes.len() && depth > 0 {
            match bytes[index] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        if depth > 0 {
            return None;
        }
        branches.push((selector.to_string(), text[start..index - 1].to_string()));
        at = index;
    }
}

fn pick(plural: &Plural, number: f64, lang: Lang) -> &str {
    plural
        .branch(&format!("={}", js_number(number)))
        .or_else(|| plural.branch(plural_category(number, lang)))
        .or_else(|| plural.branch("other"))
        .unwrap_or("")
}

/// `text` with its placeholders filled in for `lang`; `hash` is what `#` stands for inside a plural branch.
pub fn format(text: &str, params: Option<&Params>, lang: Lang, hash: Option<&str>) -> String {
    if params.is_none() && hash.is_none() {
        return text.to_string();
    }
    let mut out = String::new();
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let char = rest.chars().next().unwrap();
        if char == '#' {
            if let Some(hash) = hash {
                out.push_str(hash);
                at += 1;
                continue;
            }
        }
        let Some(params) = params.filter(|_| char == '{') else {
            out.push(char);
            at += char.len_utf8();
            continue;
        };
        if let Some((name, taken)) = simple(rest) {
            match params.get(name) {
                Some(value) => out.push_str(&value.shown()),
                None => out.push_str(&rest[..taken]),
            }
            at += taken;
            continue;
        }
        if let Some(plural) = parse_plural(text, at) {
            match params.get(&plural.name) {
                Some(value) if plural.select => {
                    let shown = value.shown();
                    let branch = plural.branch(&shown).or_else(|| plural.branch("other")).unwrap_or("");
                    out.push_str(&format(branch, Some(params), lang, hash));
                }
                Some(value) => match value.number() {
                    Some(number) => out.push_str(&format(pick(&plural, number, lang), Some(params), lang, Some(&format_number(number, lang)))),
                    None => out.push_str(&value.shown()),
                },
                // A value nobody gave: the placeholder stays whole, as an unknown {name} does.
                None => out.push_str(&text[at..plural.end]),
            }
            at = plural.end;
            continue;
        }
        out.push(char);
        at += 1;
    }
    out
}

/// The texts of one language, with English behind it and the key behind that.
pub struct Texts {
    pub lang: Lang,
    own: HashMap<&'static str, &'static str>,
    english: HashMap<&'static str, &'static str>,
}

impl Texts {
    pub fn new(lang: Lang) -> Self {
        let english: HashMap<_, _> = embedded::EN.iter().copied().collect();
        let own = match lang {
            Lang::En => english.clone(),
            Lang::Ru => embedded::RU.iter().copied().collect(),
        };
        Texts { lang, own, english }
    }

    pub fn has(&self, key: &str) -> bool {
        self.english.contains_key(key)
    }

    fn text(&self, key: &str) -> Option<&'static str> {
        self.own.get(key).or_else(|| self.english.get(key)).copied()
    }

    /// The text of `key` with `params` filled in.
    pub fn t(&self, key: &str, params: &Params) -> String {
        match self.text(key) {
            Some(text) => format(text, Some(params), self.lang, None),
            None => key.to_string(),
        }
    }

    /// The text of `key`, which takes no values.
    pub fn plain(&self, key: &str) -> String {
        self.t(key, &Params::new())
    }

    /// A node as the reader knows it: the name of its type.
    pub fn node_label(&self, kind: &str) -> String {
        let key = format!("exp.node.{kind}");
        if self.has(&key) {
            self.plain(&key)
        } else {
            kind.to_string()
        }
    }

    /// Values for an engine message, with a comparison operator (`op`) and an
    /// impairment preset (`profile`, `before`: `4g`) in words.
    pub fn message_params(&self, params: &Value) -> Params {
        let mut out = Params::new();
        if let Value::Object(map) = params {
            for (name, value) in map {
                out.insert(name.clone(), Arg::from(value));
            }
        }
        if let Some(Arg::Text(op)) = out.get("op").cloned() {
            let key = format!("exp.op.{op}");
            if self.has(&key) {
                out.insert("op".into(), Arg::Text(self.plain(&key)));
            }
        }
        for name in ["profile", "before"] {
            if let Some(Arg::Text(preset)) = out.get(name).cloned() {
                let key = format!("ns.preset.{preset}");
                if self.has(&key) {
                    out.insert(name.into(), Arg::Text(self.plain(&key)));
                }
            }
        }
        out
    }

    /// The label of a field: `field.<key>`, numbered for repeated fields.
    fn field_label(&self, key: &str, index: Option<usize>) -> String {
        let text_key = format!("field.{key}");
        let name = if self.has(&text_key) { self.plain(&text_key) } else { key.to_string() };
        match index {
            Some(index) => format!("{name} {index}"),
            None => name,
        }
    }

    /// A failure as a person reads it. `wording` lists dictionary prefixes
    /// to try before `err.` (the command line's own wording of a code that
    /// the app words for its screens); `node_label` names a node.
    pub fn describe(&self, error: &EngineError, wording: &[&str], node_label: &dyn Fn(&str) -> Option<String>) -> Described {
        let params: Value = serde_json::to_value(&error.params).unwrap_or_default();
        let params = self.message_params(&params);
        let key = wording.iter().map(|prefix| format!("{prefix}{}", error.code)).chain([format!("err.{}", error.code)]).find(|key| self.has(key));
        let message = match key {
            Some(key) => self.t(&key, &params),
            None => self.t("err.unknown", &crate::params! { "code" => error.code.as_str() }),
        };
        let mut place = Vec::new();
        if let Some(label) = error.node.as_deref().and_then(node_label) {
            place.push(label);
        }
        if let Some(field) = &error.field {
            place.push(self.field_label(&field.key, field.index));
        }
        if let Some(position) = error.params.get("position") {
            place.push(self.t("err.position", &crate::params! { "position" => position.as_str() }));
        }
        let place = place.join(" · ");
        let text = if place.is_empty() { message.clone() } else { format!("{place} — {message}") };
        Described { message, place, detail: error.detail.clone().filter(|detail| !detail.is_empty()), text }
    }
}

/// A failure in words.
#[derive(Debug, Clone, PartialEq)]
pub struct Described {
    /// What went wrong and why.
    pub message: String,
    /// Where: node, field, position — "HTTP request · URL". Empty when nowhere in particular.
    pub place: String,
    /// The system's, parser's or library's own text.
    pub detail: Option<String>,
    /// `place — message`, for one line.
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILES: &str = "{n, plural, one {# файл} few {# файла} many {# файлов} other {# файла}}";

    fn fmt(text: &str, params: &Params, lang: Lang) -> String {
        format(text, Some(params), lang, None)
    }

    /// The dictionaries embedded are the interface's, every key of them.
    #[test]
    fn the_extraction_finds_every_key_of_the_dictionaries() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/lib/locales");
        for (lang, embedded) in [("en", embedded::EN), ("ru", embedded::RU)] {
            let source = std::fs::read_to_string(root.join(format!("{lang}.ts"))).unwrap().replace("\r\n", "\n");
            // Every entry starts a line with two spaces and its quoted key.
            let lines = source.lines().filter(|line| line.starts_with("  \"") && line.contains("\":")).count();
            assert_eq!(embedded.len(), lines, "{lang}.ts: every entry, nothing else");
            assert!(embedded.len() > 1000);
        }
        let english: std::collections::BTreeSet<_> = embedded::EN.iter().map(|(key, _)| *key).collect();
        let russian: std::collections::BTreeSet<_> = embedded::RU.iter().map(|(key, _)| *key).collect();
        assert_eq!(english, russian);
        let texts = Texts::new(Lang::En);
        assert_eq!(texts.plain("exp.node.http"), "HTTP request");
        assert_eq!(Texts::new(Lang::Ru).plain("exp.node.end"), "Финиш");
        assert_eq!(texts.plain("no.such.key"), "no.such.key", "an unknown key shows as itself");
    }

    /// The cases of tests/i18n.test.mjs, here.
    #[test]
    fn placeholders_take_values_as_given_and_plurals_follow_the_language() {
        assert_eq!(fmt("to {target}:{port}", &params! { "target" => "127.0.0.1", "port" => 9000u64 }, Lang::Ru), "to 127.0.0.1:9000", "a port is not grouped");
        let signals = "{n, plural, one {# signal} other {# signals}}";
        assert_eq!(fmt(signals, &params! { "n" => 1u64 }, Lang::En), "1 signal");
        assert_eq!(fmt(signals, &params! { "n" => 2u64 }, Lang::En), "2 signals");
        assert_eq!(fmt(signals, &params! { "n" => 0u64 }, Lang::En), "0 signals");
        for (n, text) in [(1.0, "1 файл"), (2.0, "2 файла"), (5.0, "5 файлов"), (11.0, "11 файлов"), (21.0, "21 файл"), (22.0, "22 файла"), (1.5, "1,5 файла")] {
            assert_eq!(fmt(FILES, &params! { "n" => n }, Lang::Ru), text, "{n}");
        }
        assert_eq!(fmt(FILES, &params! { "n" => 12345u64 }, Lang::Ru).replace('\u{a0}', " "), "12 345 файлов", "# is the number as the language writes it");
        assert_eq!(fmt("{n, plural, =0 {nothing} one {# item} other {# items}}", &params! { "n" => 0u64 }, Lang::En), "nothing", "an exact value first");
        assert_eq!(fmt("{n, plural, one {# of {total}} other {# of {total}}}", &params! { "n" => 3u64, "total" => 7u64 }, Lang::En), "3 of 7", "values inside a branch");
        assert_eq!(fmt("{n, plural, one {# frame} other {# frames}}", &params! { "n" => "1" }, Lang::En), "1 frame", "engine values arrive as strings");
        assert_eq!(fmt("{n, plural, one {# frame} other {# frames}}", &params! { "n" => "many" }, Lang::En), "many", "a value that is not a number shows as it is");
    }

    #[test]
    fn a_russian_count_takes_the_form_its_number_needs() {
        let texts = Texts::new(Lang::Ru);
        let signals = |n: u64| texts.t("log.libraryLoaded", &params! { "n" => n, "path" => "x" });
        assert_eq!(signals(1), "1 сигнал из x");
        assert_eq!(signals(2), "2 сигнала из x");
        assert_eq!(signals(5), "5 сигналов из x");
        assert_eq!(signals(21), "21 сигнал из x");
        assert_eq!(signals(112), "112 сигналов из x");
    }

    #[test]
    fn select_picks_a_branch_by_the_value() {
        let text = "{topic}{retain, select, true { · retain} other {}}";
        assert_eq!(fmt(text, &params! { "topic" => "a/b", "retain" => "true" }, Lang::En), "a/b · retain");
        assert_eq!(fmt(text, &params! { "topic" => "a/b", "retain" => "false" }, Lang::En), "a/b");
        let nested = "{n, plural, one {# {kind, select, udp {datagram} other {packet}}} other {# {kind, select, udp {datagrams} other {packets}}}}";
        assert_eq!(fmt(nested, &params! { "n" => 2u64, "kind" => "udp" }, Lang::En), "2 datagrams", "nested in a plural");
    }

    #[test]
    fn anything_else_in_braces_is_text() {
        assert_eq!(fmt("use {{name}} here", &params! { "other" => 1u64 }, Lang::En), "use {{name}} here");
        assert_eq!(fmt("hello {who}", &params! { "other" => 1u64 }, Lang::En), "hello {who}", "an unknown value stays visible");
        assert_eq!(
            fmt("on {bind}{n, plural, one { + {g}} other { + {g}}}", &params! { "bind" => "x", "g" => "y" }, Lang::En),
            "on x{n, plural, one { + {g}} other { + {g}}}",
            "…a plural without its value too, whole"
        );
        assert_eq!(fmt("{n, plural, one {# x}", &params! { "n" => 1u64 }, Lang::En), "{n, plural, one {# x}", "a broken plural is left as it is");
        assert_eq!(fmt("# and {a}", &params! { "a" => "b" }, Lang::En), "# and b", "# means the number only inside a plural");
        assert_eq!(format("no params {x}", None, Lang::En, None), "no params {x}");
    }

    #[test]
    fn numbers_are_written_the_language_s_way() {
        assert_eq!(format_number(1234567.0, Lang::En), "1,234,567");
        assert_eq!(format_number(1234567.0, Lang::Ru), "1\u{a0}234\u{a0}567");
        assert_eq!(format_number(1234.0, Lang::Ru), "1\u{a0}234");
        assert_eq!(format_number(0.125, Lang::En), "0.125");
        assert_eq!(format_number(1.5, Lang::Ru), "1,5");
        assert_eq!(format_number(12345.5678, Lang::Ru), "12\u{a0}345,568");
        assert_eq!(format_number(1.0005, Lang::En), "1.001", "half away from zero, on the decimal the number reads as");
        assert_eq!(format_number(999.9996, Lang::En), "1,000");
        assert_eq!(format_number(-0.0005, Lang::En), "-0.001");
        assert_eq!(format_number(-5.0, Lang::Ru), "-5");
        assert_eq!(format_number(0.1 + 0.2, Lang::En), "0.3");
    }

    #[test]
    fn plural_rules_are_cldr_s() {
        let ru = |n: f64| plural_category(n, Lang::Ru);
        assert_eq!([ru(1.0), ru(2.0), ru(4.0), ru(5.0), ru(11.0), ru(12.0), ru(14.0), ru(21.0), ru(22.0), ru(25.0), ru(101.0), ru(111.0), ru(0.0), ru(1.5)], ["one", "few", "few", "many", "many", "many", "many", "one", "few", "many", "one", "many", "many", "other"]);
        assert_eq!([plural_category(1.0, Lang::En), plural_category(0.0, Lang::En), plural_category(2.0, Lang::En), plural_category(1.5, Lang::En)], ["one", "other", "other", "other"]);
    }

    #[test]
    fn the_language_is_the_flag_then_signallab_lang_then_the_locale() {
        let env = |pairs: &'static [(&'static str, &'static str)]| move |name: &str| pairs.iter().find(|(key, _)| *key == name).map(|(_, value)| value.to_string());
        assert_eq!(Lang::choose(Some(Lang::Ru), env(&[("SIGNALLAB_LANG", "en")])), Lang::Ru);
        assert_eq!(Lang::choose(None, env(&[("SIGNALLAB_LANG", "ru-RU"), ("LANG", "en_US.UTF-8")])), Lang::Ru);
        assert_eq!(Lang::choose(None, env(&[("LANG", "ru_RU.UTF-8")])), Lang::Ru);
        assert_eq!(Lang::choose(None, env(&[("LC_ALL", "C"), ("LANG", "ru_RU.UTF-8")])), Lang::En, "LC_ALL wins, and C is English");
        assert_eq!(Lang::choose(None, env(&[("LC_ALL", ""), ("LC_MESSAGES", "ru_RU"), ("LANG", "en_US")])), Lang::Ru, "an empty one is skipped");
        assert_eq!(Lang::choose(None, env(&[("SIGNALLAB_LANG", "de")])), Lang::En, "a language Signal Lab does not have");
        assert_eq!(Lang::choose(None, env(&[])), Lang::En);
    }

    #[test]
    fn failures_read_as_the_interface_words_them() {
        let texts = Texts::new(Lang::En);
        let error = EngineError::new("name.unknown").with("name", "token").in_field(signal_lab_engine::error::Field::nth("header_value", 2)).at("login").because("raw");
        let described = texts.describe(&error, &[], &|id| (id == "login").then(|| "HTTP request".to_string()));
        assert_eq!(described.place, format!("HTTP request · {} 2", texts.plain("field.header_value")));
        assert_eq!(described.message, texts.t("err.name.unknown", &params! { "name" => "token" }));
        assert_eq!(described.text, format!("{} — {}", described.place, described.message));
        assert_eq!(described.detail.as_deref(), Some("raw"));
        // Built at run time, so the scan for codes in engine/src/error.rs does not ask for a text.
        let unknown = texts.describe(&EngineError::new(&["no", "such_code"].join(".")), &[], &|_| None);
        assert_eq!(unknown.text, "Unexpected engine error (no.such_code)");
        // The command line's own wording of a code, where it has one.
        let missing = EngineError::new("secret.missing").with("name", "API_TOKEN");
        assert!(texts.describe(&missing, &["cli.err."], &|_| None).text.contains("SIGNALLAB_SECRET_API_TOKEN"));
        assert_eq!(texts.describe(&missing, &[], &|_| None).text, texts.t("err.secret.missing", &params! { "name" => "API_TOKEN" }));
        // A comparison operator in words.
        let compared = texts.message_params(&serde_json::json!({ "op": "eq", "n": 3 }));
        assert_eq!(compared["op"], Arg::Text(texts.plain("exp.op.eq")));
        assert_eq!(compared["n"], Arg::Int(3));
    }
}
