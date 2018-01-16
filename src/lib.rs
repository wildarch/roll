#[allow(unused_imports)]
// This would be flagged as an unused import but we actually use it in the tests module
#[macro_use]
extern crate serde_json;
use serde_json::value::{Value, Number};
extern crate serde;
use serde::de::DeserializeOwned;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use super::*;
        let json = json!({
            "we": {
                "are": [
                    {
                        "in": "way",
                        "deep": true
                    },
                    {
                        "in": "too",
                        "too": "deep"
                    },
                    42
                ],
                "definitely": null
            }
        });

        let a = json.roll().at("we").at("are");
        let b = a.clone().array().object().string();
        let c = a.array().number()[0].as_u64();

        assert_eq!(b, vec!["way", "too", "deep"]);
        assert_eq!(c, Some(42));
    }
}

#[derive(Debug, Clone)]
pub enum MatchNode {
    Key(String),
    Index(usize),
    Object,
    Array,
}

#[derive(Debug, Clone)]
pub struct MatchStack<'a> {
    stack: Vec<MatchNode>,
    value: &'a Value,
}

impl<'a> From<&'a Value> for MatchStack<'a> {
    fn from(v: &'a Value) -> MatchStack<'a> {
        MatchStack {
            stack: Vec::new(),
            value: v
        }
    }
}

impl<'a> MatchStack<'a> {
    pub fn at<S: Into<String>>(mut self, key: S) -> MatchStack<'a> {
        self.stack.push(MatchNode::Key(key.into()));
        self
    }

    pub fn index(mut self, i: usize) -> MatchStack<'a> {
        self.stack.push(MatchNode::Index(i));
        self
    }

    pub fn array(mut self) -> MatchStack<'a> {
        self.stack.push(MatchNode::Array);
        self
    }

    pub fn object(mut self) -> MatchStack<'a> {
        self.stack.push(MatchNode::Object);
        self
    }

    pub fn value(self) -> Vec<&'a Value> {
        let mut results = Vec::new();
        self.evaluate(&mut results, Some(self.value), 0);
        results
    }

    fn evaluate(&self, results: &mut Vec<&'a Value>, value: Option<&'a Value>, depth: usize) {
        if depth >= self.stack.len() {
            if let Some(val) = value {
                results.push(val);
            }
            return;
        }
        match self.stack[depth] {
            MatchNode::Key(ref key) => 
                self.evaluate(results, value.and_then(|v| v.get(key)), depth+1),
            MatchNode::Index(i) =>
                self.evaluate(results, value.and_then(|v| v.get(i)), depth+1),
            MatchNode::Object => {
                if let Some(&Value::Object(ref obj)) = value {
                    for val in obj.values() {
                        self.evaluate(results, Some(val), depth+1)
                    }
                }
            },
            MatchNode::Array => {
                if let Some(&Value::Array(ref arr)) = value {
                    for val in arr {
                        self.evaluate(results, Some(val), depth+1);
                    }
                }
            }
        }
    }

    pub fn bool(self) -> Vec<bool> {
        self.value().into_iter().flat_map(|v| v.as_bool()).collect()
    }

    pub fn number(self) -> Vec<&'a Number> {
        self.value().into_iter().flat_map(|v| match v {
            &Value::Number(ref n) => Some(n),
            _ => None
        }).collect()
    }

    pub fn str(self) -> Vec<&'a str> {
        self.value().into_iter().flat_map(|v| v.as_str()).collect()
    }

    pub fn string(self) -> Vec<String> {
        self.str().into_iter().map(|s| s.to_owned()).collect()
    }

    pub fn deserialize<T: DeserializeOwned>(self) -> Vec<T> {
        self.value().into_iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect()
    }
}

pub trait Roll {
    fn roll<'a>(&'a self) -> MatchStack<'a>;
}

impl Roll for Value {
    fn roll<'a>(&'a self) -> MatchStack<'a> {
        MatchStack::from(self)
    }
}
