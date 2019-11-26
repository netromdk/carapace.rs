use glob::glob;
use json::JsonValue;
use regex::Regex;

use std::collections::HashMap;

lazy_static! {
    static ref WORD_REGEX: Regex = Regex::new(r"(\w+)").unwrap();
    static ref GLOB_REGEX: Regex = Regex::new(r"(([\w\d.\\/\.]*\*[\w\d.\\/\.]*)+)").unwrap();
    static ref ENV_VAR_REGEX: Regex = Regex::new(r"(\$[\w\?\-#!\$_@\*]*)").unwrap();
    static ref BRACKET_ENV_VAR_REGEX: Regex = Regex::new(r"(\$\{([\w\?\-#!\$_@\*]+)\})").unwrap();
    static ref PARTIAL_BRACKET_ENV_VAR_REGEX: Regex =
        Regex::new(r"(\$\{([\w\?\-#!\$_@\*]*)\}?)").unwrap();
}

/// Check if `pos`ition is within first word in `text`.
pub fn in_first_word(pos: usize, text: &str) -> bool {
    if let Some(wpos) = text.find(char::is_whitespace) {
        return pos < wpos;
    }
    true
}

// TODO: -> Option<String>
pub fn word_at_pos(pos: usize, text: &str) -> String {
    assert!(pos <= text.len());
    for cap in WORD_REGEX.captures_iter(text) {
        let cap = cap.get(0).unwrap();
        if pos >= cap.start() && pos <= cap.end() {
            return cap.as_str().to_string();
        }
    }
    "".to_string()
}

pub fn glob_at_pos(pos: usize, text: &str) -> String {
    assert!(pos <= text.len());
    for cap in GLOB_REGEX.captures_iter(text) {
        let cap = cap.get(0).unwrap();
        if pos >= cap.start() && pos <= cap.end() {
            return cap.as_str().to_string();
        }
    }
    "".to_string()
}

// TODO: -> Option<String>
pub fn env_var_at_pos(pos: usize, text: &str) -> String {
    assert!(pos <= text.len());
    for cap in BRACKET_ENV_VAR_REGEX.captures_iter(text) {
        let cap0 = cap.get(0).unwrap();
        let cap1 = cap.get(1);
        if pos >= cap0.start() && pos <= cap0.end() {
            if let Some(cap1_val) = cap1 {
                return cap1_val.as_str().to_string();
            }
        }
    }
    for cap in ENV_VAR_REGEX.captures_iter(text) {
        let cap = cap.get(0).unwrap();
        if pos >= cap.start() && pos <= cap.end() {
            return cap.as_str().to_string();
        }
    }
    "".to_string()
}

// TODO: -> Option<String>
pub fn partial_env_var_at_pos(pos: usize, text: &str) -> String {
    assert!(pos <= text.len());
    for cap in PARTIAL_BRACKET_ENV_VAR_REGEX.captures_iter(text) {
        let cap0 = cap.get(0).unwrap();
        let cap1 = cap.get(1);
        if pos >= cap0.start() && pos <= cap0.end() {
            if let Some(cap1_val) = cap1 {
                return cap1_val.as_str().to_string();
            }
        }
    }
    for cap in ENV_VAR_REGEX.captures_iter(text) {
        let cap = cap.get(0).unwrap();
        if pos >= cap.start() && pos <= cap.end() {
            return cap.as_str().to_string();
        }
    }
    "".to_string()
}

pub fn hash_map_to_json<S: ::std::hash::BuildHasher>(
    map: &HashMap<String, String, S>,
) -> JsonValue {
    let mut val = JsonValue::new_object();
    for (key, value) in map {
        val[key] = JsonValue::from(value.clone());
    }
    val
}

pub fn json_obj_to_hash_map(obj: &JsonValue) -> HashMap<String, String> {
    assert!(obj.is_object());
    let mut map = HashMap::new();
    for (key, val) in obj.entries() {
        if let Some(s) = val.as_str() {
            map.insert(key.to_string(), s.to_string());
        }
    }
    map
}

pub fn expand_glob(input: &str) -> Vec<String> {
    let mut res = Vec::new();
    for path in glob(input).unwrap().filter_map(Result::ok) {
        res.push(path.to_str().unwrap().to_string());
    }
    if res.is_empty() {
        res.push(input.to_string());
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_first_word_beginning() {
        assert!(in_first_word(0, "hello world"));
    }

    #[test]
    fn in_first_word_middle() {
        assert!(in_first_word(2, "hello world"));
    }

    #[test]
    fn in_first_word_end() {
        assert!(in_first_word(4, "hello world"));
    }

    #[test]
    fn in_first_word_on_boundary_whitespace() {
        assert!(!in_first_word(5, "hello world"));
    }

    #[test]
    fn in_first_word_next_word() {
        assert!(!in_first_word(6, "hello world"));
    }

    #[test]
    fn word_at_pos_beginning() {
        assert_eq!(word_at_pos(0, "hello world and universe"), "hello");
    }

    #[test]
    fn word_at_pos_start() {
        assert_eq!(word_at_pos(6, "hello world and universe"), "world");
    }

    #[test]
    fn word_at_pos_middle() {
        assert_eq!(word_at_pos(2, "hello world and universe"), "hello");
    }

    #[test]
    fn word_at_pos_end() {
        assert_eq!(word_at_pos(10, "hello world and universe"), "world");
    }

    #[test]
    fn word_at_pos_right_after() {
        assert_eq!(word_at_pos(11, "hello world and universe"), "world");
    }

    #[test]
    fn word_at_pos_after() {
        assert_eq!(word_at_pos(12, "hello world  and universe"), "");
    }

    #[test]
    fn env_var_at_pos_beginning() {
        assert_eq!(env_var_at_pos(0, "$hello world and universe"), "$hello");
    }

    #[test]
    fn env_var_at_pos_start() {
        assert_eq!(env_var_at_pos(6, "hello $world and universe"), "$world");
    }

    #[test]
    fn env_var_at_pos_middle() {
        assert_eq!(env_var_at_pos(2, "$hello world and universe"), "$hello");
    }

    #[test]
    fn env_var_at_pos_end() {
        assert_eq!(env_var_at_pos(11, "hello $world and universe"), "$world");
    }

    #[test]
    fn env_var_at_pos_right_after() {
        assert_eq!(env_var_at_pos(12, "hello $world and universe"), "$world");
    }

    #[test]
    fn env_var_at_pos_after() {
        assert_eq!(env_var_at_pos(13, "hello $world  and universe"), "");
    }

    #[test]
    fn env_var_at_pos_only_dollar_sign() {
        assert_eq!(env_var_at_pos(6, "hello $  and universe"), "$");
    }

    #[test]
    fn env_var_at_pos_dollar_dash() {
        assert_eq!(env_var_at_pos(6, "hello $- and universe"), "$-");
    }

    #[test]
    fn bracket_env_var_at_pos_start() {
        assert_eq!(env_var_at_pos(6, "hello ${world} and universe"), "${world}");
    }

    #[test]
    fn bracket_env_var_at_pos_middle() {
        assert_eq!(
            env_var_at_pos(10, "hello ${world} and universe"),
            "${world}"
        );
    }

    #[test]
    fn bracket_env_var_at_pos_end() {
        assert_eq!(
            env_var_at_pos(13, "hello ${world} and universe"),
            "${world}"
        );
    }

    #[test]
    fn partial_env_var_at_pos_start() {
        assert_eq!(
            partial_env_var_at_pos(6, "hello ${world and universe"),
            "${world"
        );
    }

    #[test]
    fn partial_env_var_at_pos_middle() {
        assert_eq!(
            partial_env_var_at_pos(9, "hello ${world and universe"),
            "${world"
        );
    }

    #[test]
    fn partial_env_var_at_pos_end() {
        assert_eq!(
            partial_env_var_at_pos(12, "hello ${world and universe"),
            "${world"
        );
    }

    #[test]
    fn partial_env_var_at_pos_can_yield_full_match() {
        assert_eq!(
            partial_env_var_at_pos(9, "hello ${world} and universe"),
            "${world}"
        );
    }

    #[test]
    fn partial_env_var_at_pos_only_dollar_sign_bracket() {
        assert_eq!(partial_env_var_at_pos(6, "hello ${  and universe"), "${");
    }

    #[test]
    fn partial_env_var_at_pos_only_dollar_sign_bracket_dash() {
        assert_eq!(partial_env_var_at_pos(6, "hello ${-  and universe"), "${-");
    }

    #[test]
    fn test_hash_map_to_json() {
        let mut map = HashMap::new();
        map.insert("one".to_string(), "1".to_string());
        map.insert("two".to_string(), "2".to_string());
        map.insert("three".to_string(), "3".to_string());

        let jmap = hash_map_to_json(&map);
        assert!(jmap.is_object());
        assert!(jmap.has_key("one"));
        assert_eq!(jmap["one"], JsonValue::String("1".to_string()));
        assert!(jmap.has_key("two"));
        assert_eq!(jmap["two"], JsonValue::String("2".to_string()));
        assert!(jmap.has_key("three"));
        assert_eq!(jmap["three"], JsonValue::String("3".to_string()));
    }

    #[test]
    fn test_json_obj_to_hash_map() {
        let obj = json::object![
            "one" => "1",
            "two" => "2",
            "three" => "3",
        ];

        let map = json_obj_to_hash_map(&obj);
        assert_eq!(map.len(), 3);
        assert!(map.contains_key("one"));
        assert_eq!(map.get("one"), Some(&"1".to_string()));
        assert!(map.contains_key("two"));
        assert_eq!(map.get("two"), Some(&"2".to_string()));
        assert!(map.contains_key("three"));
        assert_eq!(map.get("three"), Some(&"3".to_string()));
    }
}
