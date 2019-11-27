use regex::{Captures, Regex};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::hash::Hash;
use std::ops::Index;

lazy_static! {
    static ref ENV_VAR_REGEX: Regex = Regex::new(r"(\$[\w\?\-#!\$_@\*]*)").unwrap();
    static ref PARTIAL_BRACKET_ENV_VAR_REGEX: Regex =
        Regex::new(r"(\$\{([\w\?\-#!\$_@\*]*)\}?)").unwrap();
    static ref BRACKET_ENV_VAR_REGEX: Regex = Regex::new(r"(\$\{([\w\?\-#!\$_@\*]+)\})").unwrap();
}

type Key = String;
type Value = String;
type Map = HashMap<Key, Value>;

/// Env encapsulates environment variables and their manipulation.
pub struct Env {
    env: Map,
}

impl Env {
    pub fn new() -> Env {
        Env {
            env: env::vars().collect(),
        }
    }

    pub fn insert(&mut self, key: Key, value: Value) {
        self.env.insert(key, value);
    }

    pub fn remove<S>(&mut self, key: &S)
    where
        S: ?Sized + Hash + Eq,
        Key: Borrow<S>,
    {
        self.env.remove(key);
    }

    pub fn get<S>(&self, key: &S) -> Option<&Value>
    where
        S: ?Sized + Hash + Eq,
        Key: Borrow<S>,
    {
        self.env.get(key)
    }

    pub fn contains_key<S>(&self, key: &S) -> bool
    where
        S: ?Sized + Hash + Eq,
        Key: Borrow<S>,
    {
        self.env.contains_key(key)
    }

    /// Append value to value at key but only if current value doesn't already contain input value.
    /** If key doesn't exist then an empty string entry will be created. */
    pub fn append<S>(&mut self, key: &S, value: Value)
    where
        S: ?Sized + Hash + Eq + ToString,
        Key: Borrow<S>,
    {
        if !self.env.contains_key(key) {
            self.env.insert(key.to_string(), "".to_string());
        }
        let old_value = self.env[&key].clone();
        if !old_value.contains(&value) {
            self.env.insert(key.to_string(), old_value + &value);
        }
    }

    /// Replace value with another value for value at key but only if current value already is
    /// contained.
    /** If key doesn't exist then an empty string entry will be created. */
    pub fn replace<S>(&mut self, key: &S, old_value: Value, new_value: Value)
    where
        S: ?Sized + Hash + Eq + ToString,
        Key: Borrow<S>,
    {
        if !self.env.contains_key(key) {
            self.env.insert(key.to_string(), "".to_string());
        }
        let value = self.env[key].clone();
        if value.contains(&old_value) {
            self.env
                .insert(key.to_string(), value.replace(&old_value, &new_value));
        }
    }

    pub fn print(&self) {
        let mut keys: Vec<&Key> = self.env.keys().peekable().collect();
        keys.sort();
        for k in &keys {
            println!("{}={}", k, self.env[*k]);
        }
    }

    /// Replaces all environment variables in \p data and returns resulting string.
    pub fn replace_vars<S>(&self, data: &S) -> Value
    where
        S: ?Sized + Hash + Eq + ToString,
        Key: Borrow<S>,
    {
        let mut res = data.to_string();
        for (k, v) in &self.env {
            // Bracketed version always replaces.
            res = res.replace(&format!("${{{}}}", k), &v);

            // Non-bracketed version can only replace when complete subset of string. For instance,
            // "$USER" must not replace in "$USERNAME" but "$USERNAME" can since it's the complete
            // string.
            let lookfor = format!("${}", k);
            res = ENV_VAR_REGEX
                .replace_all(&res, |caps: &Captures| {
                    let m = caps.get(0).unwrap().as_str();
                    if m == lookfor {
                        v.to_string()
                    } else {
                        m.to_string()
                    }
                })
                .into_owned();
        }
        res
    }

    // TODO: -> Option<String>
    /// Returns environment variable at position in text.
    pub fn var_at_pos(pos: usize, text: &str) -> Value {
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

    // TODO: return Option<Value>
    /// Returns partial environment variable at position in text.
    pub fn partial_var_at_pos(pos: usize, text: &str) -> Value {
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
}

impl Default for Env {
    fn default() -> Env {
        Env {
            env: HashMap::new(),
        }
    }
}

impl<S> Index<&S> for Env
where
    S: ?Sized + Hash + Eq,
    Key: Borrow<S>,
{
    type Output = Value;

    fn index(&self, key: &S) -> &Self::Output {
        &self.env[key]
    }
}

impl AsRef<Map> for Env {
    fn as_ref(&self) -> &Map {
        &self.env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_current_vars() {
        assert_eq!(Env::new().env, env::vars().collect());
    }

    #[test]
    fn default_is_empty() {
        let env = Env::default();
        assert!(env.env.is_empty());
    }

    #[test]
    fn as_ref() {
        let mut env = Env::default();
        env.insert("foo".to_string(), "bar".to_string());
        env.insert("baz".to_string(), "taz".to_string());
        assert_eq!(*env.as_ref(), env.env);
        assert_eq!(2, env.as_ref().len());
    }

    #[test]
    fn insert() {
        let mut env = Env::default();
        assert!(!env.contains_key("a"));
        env.insert("a".to_string(), "b".to_string());
        assert!(env.contains_key("a"));
        assert_eq!(env["a"], "b");
    }

    #[test]
    fn remove() {
        let mut env = Env::default();
        env.insert("a".to_string(), "b".to_string());
        assert!(env.contains_key("a"));
        env.remove("a");
        assert!(!env.contains_key("a"));
    }

    #[test]
    fn get() {
        let mut env = Env::default();
        env.insert("a".to_string(), "b".to_string());
        assert_eq!(env.get("a"), Some(&"b".to_string()));
        assert_eq!(env.get("b"), None);
    }

    #[test]
    fn contains_key() {
        let mut env = Env::default();
        assert!(!env.env.contains_key("a"));
        assert!(!env.contains_key("a"));
        env.insert("a".to_string(), "b".to_string());
        assert!(env.env.contains_key("a"));
        assert!(env.contains_key("a"));
    }

    #[test]
    fn append_value_for_key_with_no_value() {
        let mut env = Env::default();
        env.append("foo", "a".to_string());
        assert!(env.contains_key("foo"));
        assert_eq!("a", env["foo"]);
    }

    #[test]
    fn append_value_for_key_with_prior_value() {
        let mut env = Env::default();
        env.insert("foo".to_string(), "a".to_string());
        env.append("foo", "b".to_string());
        assert!(env.contains_key("foo"));
        assert_eq!("ab", env["foo"]);
    }

    #[test]
    fn dont_append_value_for_key_if_value_exits() {
        let mut env = Env::default();
        env.insert("foo".to_string(), "a".to_string());
        env.append("foo", "a".to_string());
        assert!(env.contains_key("foo"));
        assert_eq!("a", env["foo"]);
    }

    #[test]
    fn replace_value_for_key_with_no_value() {
        let mut env = Env::default();
        env.replace("foo", "a".to_string(), "b".to_string());
        assert!(env.contains_key("foo"));
        assert_eq!("", env["foo"]);
    }

    #[test]
    fn replace_value_for_key_with_prior_value() {
        let mut env = Env::default();
        env.insert("foo".to_string(), "a".to_string());
        env.replace("foo", "a".to_string(), "b".to_string());
        assert!(env.contains_key("foo"));
        assert_eq!("b", env["foo"]);
    }

    #[test]
    fn dont_replace_value_for_key_if_value_doesnt_exists() {
        let mut env = Env::default();
        env.replace("foo", "a".to_string(), "b".to_string());
        assert!(env.contains_key("foo"));
        assert_eq!("", env["foo"]);
    }

    #[test]
    fn replace_vars_general() {
        let input = String::from("$ONE, ${TWO}, $ONE, $THREE");
        let mut env = Env::default();
        env.insert("ONE".to_string(), "1".to_string());
        env.insert("TWO".to_string(), "2".to_string());
        let output = env.replace_vars(&input);
        assert_eq!(output, "1, 2, 1, $THREE".to_string());
    }

    #[test]
    fn replace_vars_dont_when_subset() {
        let input = String::from("$USERNAME");
        let mut env = Env::default();
        env.insert("USER".to_string(), "test".to_string());
        let output = env.replace_vars(&input);

        // $USERNAME is not present and $USER is subset of $USERNAME, so don't replace!
        assert_eq!(output, "$USERNAME".to_string());
    }

    #[test]
    fn replace_vars_do_when_last() {
        let input = String::from("$USER");
        let mut env = Env::default();
        env.insert("USER".to_string(), "test".to_string());
        let output = env.replace_vars(&input);
        assert_eq!(output, "test".to_string());
    }

    #[test]
    fn replace_vars_do_when_subset_when_bracketed() {
        let input = String::from("${USER}NAME");
        let mut env = Env::default();
        env.insert("USER".to_string(), "test".to_string());
        let output = env.replace_vars(&input);
        assert_eq!(output, "testNAME".to_string());
    }

    #[test]
    fn replace_vars_use_longest_match() {
        let input = String::from("$USERNAME");
        let mut env = Env::default();
        env.insert("USER".to_string(), "test".to_string());
        env.insert("USERNAME".to_string(), "foobar".to_string());
        let output = env.replace_vars(&input);
        assert_eq!(output, "foobar".to_string());
    }

    #[test]
    fn partial_env_var_at_pos_start() {
        assert_eq!(
            Env::partial_var_at_pos(6, "hello ${world and universe"),
            "${world"
        );
    }

    #[test]
    fn partial_env_var_at_pos_middle() {
        assert_eq!(
            Env::partial_var_at_pos(9, "hello ${world and universe"),
            "${world"
        );
    }

    #[test]
    fn partial_env_var_at_pos_end() {
        assert_eq!(
            Env::partial_var_at_pos(12, "hello ${world and universe"),
            "${world"
        );
    }

    #[test]
    fn partial_env_var_at_pos_can_yield_full_match() {
        assert_eq!(
            Env::partial_var_at_pos(9, "hello ${world} and universe"),
            "${world}"
        );
    }

    #[test]
    fn partial_env_var_at_pos_only_dollar_sign_bracket() {
        assert_eq!(Env::partial_var_at_pos(6, "hello ${  and universe"), "${");
    }

    #[test]
    fn partial_env_var_at_pos_only_dollar_sign_bracket_dash() {
        assert_eq!(Env::partial_var_at_pos(6, "hello ${-  and universe"), "${-");
    }

    #[test]
    fn env_var_at_pos_beginning() {
        assert_eq!(Env::var_at_pos(0, "$hello world and universe"), "$hello");
    }

    #[test]
    fn env_var_at_pos_start() {
        assert_eq!(Env::var_at_pos(6, "hello $world and universe"), "$world");
    }

    #[test]
    fn env_var_at_pos_middle() {
        assert_eq!(Env::var_at_pos(2, "$hello world and universe"), "$hello");
    }

    #[test]
    fn env_var_at_pos_end() {
        assert_eq!(Env::var_at_pos(11, "hello $world and universe"), "$world");
    }

    #[test]
    fn env_var_at_pos_right_after() {
        assert_eq!(Env::var_at_pos(12, "hello $world and universe"), "$world");
    }

    #[test]
    fn env_var_at_pos_after() {
        assert_eq!(Env::var_at_pos(13, "hello $world  and universe"), "");
    }

    #[test]
    fn env_var_at_pos_only_dollar_sign() {
        assert_eq!(Env::var_at_pos(6, "hello $  and universe"), "$");
    }

    #[test]
    fn env_var_at_pos_dollar_dash() {
        assert_eq!(Env::var_at_pos(6, "hello $- and universe"), "$-");
    }

    #[test]
    fn bracket_env_var_at_pos_start() {
        assert_eq!(
            Env::var_at_pos(6, "hello ${world} and universe"),
            "${world}"
        );
    }

    #[test]
    fn bracket_env_var_at_pos_middle() {
        assert_eq!(
            Env::var_at_pos(10, "hello ${world} and universe"),
            "${world}"
        );
    }

    #[test]
    fn bracket_env_var_at_pos_end() {
        assert_eq!(
            Env::var_at_pos(13, "hello ${world} and universe"),
            "${world}"
        );
    }
}
