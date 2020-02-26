extern crate linked_hash_map;
extern crate num_bigint;
extern crate parser;

use linked_hash_map::LinkedHashMap;
use num_bigint::BigInt;
use parser::Command;
use std::hash::{Hash, Hasher};

pub mod resp_type {
    type RespType = char;

    // simple types

    pub const BLOB_STRING: RespType = '$';
    // $<length>\r\n<bytes>\r\n
    pub const SIMPLE_STRING: RespType = '+';
    // +<string>\r\n
    pub const SIMPLE_ERROR: RespType = '-';
    // -<string>\r\n
    pub const NUMBER: RespType = ':';
    // :<number>\r\n
    pub const NULL: RespType = '_';
    // _\r\n  resp2: -1\r\n
    pub const DOUBLE: RespType = ',';
    // ,<floating-point-number>\r\n
    pub const BOOLEAN: RespType = '#';
    // #t\r\n or #f\r\n
    pub const BLOBERROR: RespType = '!';
    // !<length>\r\n<bytes>\r\n
    pub const VERBATIMSTRING: RespType = '=';
    // =<length>\r\n<format(3 bytes):><bytes>\r\n
    pub const BIGNUMBER: RespType = '(';

    // Aggregate data types

    // (<big number>\n
    pub const ARRAY: RespType = '*';
    // *<elements number>\r\n... numelements other types ...
    pub const MAP: RespType = '%';
    // %<elements number>\r\n... numelements key/value pair of other types ...
    pub const SET: RespType = '~';
    // ~<elements number>\r\n... numelements other types ...
    pub const ATTRIBUTE: RespType = '|';
    // |~<elements number>\r\n... numelements map type ...
    pub const PUSH: RespType = '>';
    // ><elements number>\r\n<first item is String>\r\n... numelements-1 other types ...

    //special type

    pub const STREAM: &str = "$EOF:"; // $EOF:<40 bytes marker><CR><LF>... any number of bytes of data here not containing the marker ...<40 bytes marker>
}

#[derive(Debug, Eq, PartialEq, Clone, PartialOrd)]
pub struct Float64([u8; 8]);

impl From<f64> for Float64 {
    fn from(d: f64) -> Self {
        Float64(d.to_be_bytes())
    }
}

impl Float64 {
    fn to_string(&self) -> String {
        format!("{:?}", self.to_f64())
    }

    fn to_f64(&self) -> f64 {
        f64::from_be_bytes(self.0)
    }
}

impl Hash for Float64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// A command Value to send to a client
#[derive(Eq, Hash, PartialOrd, PartialEq, Debug)]
pub enum Value {
    /// A blob String  $<length>\r\n<bytes>\r\n
    Blob(Vec<u8>),
    /// A simple String  +<string>\r\n
    String(Vec<u8>),
    /// A simple error string -<string>\r\n
    Error(String),
    /// A number :<number>\r\n
    Number(i64),
    /// Type Null // resp3: _\r\n
    Null,
    /// Type Double of value defined resp3
    Double(Float64),
    /// A Boolean of value defined resp3
    Boolean(bool),
    /// type BlobError similar to simple string
    BlobError(String),
    /// bigint number
    Bigint(BigInt),

    /// An array of value that may mix different types
    Array(Vec<Value>),
    /// A map of value linked hash map
    Map(LinkedHashMap<Value, Value>),
    /// A Set
    Set(Vec<Value>),
    /// A Attribute like map
    Attribute(LinkedHashMap<Value, Value>),
    /// A Push type
    Push(Vec<Value>),
    // / A stream type
    // Stream(Vec<Value>),
}

impl Value {
    /// as_bytes test example
    ///
    ///```
    /// # extern crate num_bigint;
    /// # use resp::{Value,Float64};
    /// # use num_bigint::BigInt;
    /// # use linked_hash_map::LinkedHashMap;
    ///
    /// let number_value = Value::Number(1);
    /// assert_eq!(b":1\r\n".to_vec(), number_value.as_bytes(2));
    ///
    /// let blob_value = Value::Blob(b"123".to_vec());
    /// assert_eq!(b"$3\r\n123\r\n".to_vec(), blob_value.as_bytes(2));
    ///
    /// let boolean_value = Value::Boolean(true);
    /// assert_eq!(b"#t\r\n".to_vec(), boolean_value.as_bytes(3));
    ///
    /// let blob_error = Value::BlobError("woca".to_string());
    /// assert_eq!(b"!4\r\nwoca\r\n".to_vec(), blob_error.as_bytes(3));
    ///
    /// let double_inf_value = Value::Double(Float64::from(std::f64::INFINITY));
    /// assert_eq!(b",inf\r\n".to_vec(), double_inf_value.as_bytes(3));
    ///
    /// let double_neg_inf_value = Value::Double(Float64::from(std::f64::NEG_INFINITY));
    /// assert_eq!(b",-inf\r\n".to_vec(), double_neg_inf_value.as_bytes(3));
    ///
    /// let double_value = Value::Double(Float64::from(1.23_f64));
    /// assert_eq!(b",1.23\r\n".to_vec(), double_value.as_bytes(3));
    ///
    /// let bigint_type = BigInt::parse_bytes(b"3492890328409238509324850943850943825024385",10).unwrap();
    /// let bigint_value = Value::Bigint(bigint_type);
    /// assert_eq!(b"(3492890328409238509324850943850943825024385\r\n".to_vec(), bigint_value.as_bytes(3));
    ///
    /// let mut map = LinkedHashMap::new();
    /// map.insert(Value::String(b"first".to_vec()),Value::Number(1));
    /// map.insert(Value::String(b"second".to_vec()),Value::Number(2));
    /// let value_map = Value::Map(map);
    /// assert_eq!(b"%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n".to_vec(), value_map.as_bytes(3));
    ///
    /// let value_set = Value::Set(vec![
    ///         Value::String(b"orange".to_vec()),
    ///         Value::String(b"apple".to_vec()),
    ///         Value::Boolean(true),
    ///         Value::Number(100),
    ///         Value::Number(999),
    ///     ]);
    /// assert_eq!(b"~+orange\r\n+apple\r\n#t\r\n:100\r\n:999\r\n".to_vec(),value_set.as_bytes(3));
    ///
    ///
    /// // Attribute type |1\r\n+key-popularity\r\n%2\r\n$1\r\na\r\n,0.1923\r\n$1\r\nb\r\n,0.0012\r\n*2\r\n:2039123\r\n:9543892\r\n
    ///
    /// let push_value = Value::Push(
    ///     vec![
    ///         Value::String(b"pubsub".to_vec()),
    ///         Value::String(b"message".to_vec()),
    ///         Value::String(b"somechannel".to_vec()),
    ///         Value::String(b"this is the message".to_vec()),
    ///     ]);
    /// assert_eq!(b">4\r\n+pubsub\r\n+message\r\n+somechannel\r\n+this is the message\r\n".to_vec(),push_value.as_bytes(3));
    /// ```
    /// Serializes the value into an array of bytes using Redis protocol.
    pub fn as_bytes(&self, resp_version: usize) -> Vec<u8> {
        return match *self {
            Value::Null => {
                if resp_version == 2 {
                    [&b"-1"[..], &"\r\n".to_owned().into_bytes()[..]].concat()
                } else {
                    [&b"_"[..], &"\r\n".to_owned().into_bytes()[..]].concat()
                }
            }

            Value::String(ref s) => {
                [&b"+"[..], &s[..], &"\r\n".to_owned().into_bytes()[..]].concat()
            }

            Value::Blob(ref d) => [
                &b"$"[..],
                &format!("{}\r\n", d.len()).into_bytes()[..],
                &d[..],
                &"\r\n".to_owned().into_bytes()[..],
            ]
            .concat(),

            Value::Number(ref i) => [&b":"[..], &format!("{}\r\n", i).into_bytes()[..]].concat(),

            Value::Error(ref d) => {
                if resp_version == 2 {
                    [
                        &b"-"[..],
                        (*d).as_bytes(),
                        &"\r\n".to_owned().into_bytes()[..],
                    ]
                    .concat()
                } else {
                    [
                        &b"!"[..],
                        (*d).as_bytes(),
                        &"\r\n".to_owned().into_bytes()[..],
                    ]
                    .concat()
                }
            }

            Value::Array(ref a) => [
                &b"*"[..],
                &format!("{}\r\n", a.len()).into_bytes()[..],
                &(a.iter()
                    .map(|el| el.as_bytes(resp_version))
                    .collect::<Vec<_>>()[..]
                    .concat())[..],
            ]
            .concat(),

            Value::Double(ref f) => {
                let prefix = &b","[..];
                let suffix = &"\r\n".to_owned().into_bytes()[..];
                let d: f64 = f.to_f64();
                let value = match d {
                    std::f64::INFINITY => {
                        [prefix, &"inf".to_owned().into_bytes()[..], suffix].concat()
                    }
                    std::f64::NEG_INFINITY => {
                        [prefix, &"-inf".to_owned().into_bytes()[..], suffix].concat()
                    }
                    _ => {
                        let s = f.to_string();
                        [prefix, &s.as_bytes()[..], suffix].concat()
                    }
                };
                value
            }

            Value::Boolean(ref b) => {
                if *b {
                    [&b"#t"[..], &"\r\n".to_owned().into_bytes()[..]].concat()
                } else {
                    [&b"#f"[..], &"\r\n".to_owned().into_bytes()[..]].concat()
                }
            }

            Value::BlobError(ref s) => [
                &b"!"[..],
                &format!("{}\r\n", s.len()).into_bytes()[..],
                (*s).as_bytes(),
                &"\r\n".to_owned().into_bytes()[..],
            ]
            .concat(),

            Value::Bigint(ref b) => [&b"("[..], &format!("{}\r\n", &b).into_bytes()[..]].concat(),

            Value::Map(ref m) => {
                let mut content: Vec<u8> = Vec::new();
                for x in m.iter() {
                    for k in x.0.as_bytes(resp_version).iter() {
                        content.push(*k);
                    }
                    for v in x.1.as_bytes(resp_version).iter() {
                        content.push(*v);
                    }
                }
                [
                    &b"%"[..],
                    &format!("{}\r\n", m.len()).into_bytes()[..],
                    &content[..],
                ]
                .concat()
            }

            Value::Set(ref v) => {
                let mut content: Vec<u8> = Vec::new();
                for x in v.iter() {
                    for c in x.as_bytes(resp_version).iter() {
                        content.push(*c);
                    }
                }
                [&b"~"[..], &content[..]].concat()
            }

            Value::Attribute(ref m) => {
                let mut content: Vec<u8> = Vec::new();
                for x in m.iter() {
                    for k in x.0.as_bytes(resp_version).iter() {
                        content.push(*k);
                    }
                    for v in x.1.as_bytes(resp_version).iter() {
                        content.push(*v);
                    }
                }
                [
                    &b"|"[..],
                    &format!("{}\r\n", m.len()).into_bytes()[..],
                    &content[..],
                ]
                .concat()
            }

            Value::Push(ref a) => {
                let mut content: Vec<u8> = Vec::new();
                for i in a.iter() {
                    for x in i.as_bytes(resp_version).iter() {
                        content.push(*x);
                    }
                }
                [
                    &b">"[..],
                    &format!("{}\r\n", a.len()).into_bytes()[..],
                    &content[..],
                ]
                .concat()
            }
        };
    }

    /// Returns true if and only if the Value is an error.
    pub fn is_error(&self) -> bool {
        if let Value::Error(_) = *self {
            true
        } else {
            false
        }
    }

    /// Is the Value a status
    pub fn is_status(&self) -> bool {
        if let Value::String(_) = *self {
            true
        } else {
            false
        }
    }

    /// Is the Value Nil
    pub fn is_nil(&self) -> bool {
        if let Value::Null = *self {
            true
        } else {
            false
        }
    }
}

/// convert from Parser :: Command to Value
impl From<Command<'_>> for Value {
    fn from(c: Command) -> Self {
        Value::Null {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }
}
