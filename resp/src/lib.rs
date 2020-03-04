extern crate linked_hash_map;
extern crate num_bigint;

use linked_hash_map::LinkedHashMap;
use num_bigint::BigInt;
use std::fmt::Write;
use std::hash::{Hash, Hasher};

/// Need automatic matching $expr to generate ValuePair::new()
// macro_rules! value {
//     () => {};
// }

pub mod resp_event_type {
    // simple types
    pub const BLOB_STRING: char = '$';
    // $<length>\r\n<bytes>\r\n
    pub const SIMPLE_STRING: char = '+';
    // +<string>\r\n
    pub const SIMPLE_ERROR: char = '-';
    // -<string>\r\n
    pub const NUMBER: char = ':';
    // :<number>\r\n
    pub const NULL: char = '_';
    // _\r\n  resp2: -1\r\n
    pub const DOUBLE: char = ',';
    // ,<floating-point-number>\r\n
    pub const BOOLEAN: char = '#';
    // #t\r\n or #f\r\n
    pub const BLOB_ERROR: char = '!';
    // !<length>\r\n<bytes>\r\n
    pub const VERBATIM_STRING: char = '=';
    // =<length>\r\n<format(3 bytes):><bytes>\r\n
    pub const BIG_INT: char = '(';

    // Aggregate data types

    // (<big number>\n
    pub const ARRAY: char = '*';
    // *<elements number>\r\n... numelements other types ...
    pub const MAP: char = '%';
    // %<elements number>\r\n... numelements key/value pair of other types ...
    pub const SET: char = '~';
    // ~<elements number>\r\n... numelements other types ...
    pub const ATTRIBUTE: char = '|';
    // |~<elements number>\r\n... numelements map type ...
    pub const PUSH: char = '>';
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
#[derive(Debug)]
pub struct ValuePair {
    value: Value,
    attrs: LinkedHashMap<Value, Value>,
}
impl ValuePair {
    pub fn new(value: Value, attrs: LinkedHashMap<Value, Value>) -> Self {
        ValuePair { value, attrs }
    }
    /// to resp string
    /// ```
    /// # extern crate num_bigint;
    /// # use resp::{Value,Float64,ValuePair};
    /// # use num_bigint::BigInt;
    /// # use linked_hash_map::LinkedHashMap;
    ///
    /// // Array + Attribute
    /// let mut keyPopularityMap = LinkedHashMap::new();
    /// keyPopularityMap.insert(Value::Blob(b"a".to_vec()),Value::Double(Float64::from(0.1923_f64)));
    /// keyPopularityMap.insert(Value::Blob(b"b".to_vec()),Value::Double(Float64::from(0.0012_f64)));
    ///
    /// let mut attrMap:LinkedHashMap<Value,Value> = LinkedHashMap::new();
    /// attrMap.insert(Value::String(b"key-popularity".to_vec()),Value::Map(keyPopularityMap));
    ///
    /// let vp = ValuePair::new(Value::Array(vec![Value::Number(2039123),Value::Number(9543892)]),attrMap).to_resp_string().unwrap();
    /// assert_eq!("|1\r\n+key-popularity\r\n%2\r\n$1\r\na\r\n,0.1923\r\n$1\r\nb\r\n,0.0012\r\n*2\r\n:2039123\r\n:9543892\r\n",vp);
    ///
    /// // Bulk
    /// let vp2 = ValuePair::new(
    ///       Value::Array(vec![
    ///       Value::Blob(b"set".to_vec()),
    ///       Value::Blob(b"a".to_vec()),
    ///       Value::Blob(b"123".to_vec())]),
    ///     LinkedHashMap::new()).
    ///     to_resp_string().
    ///     unwrap();
    /// assert_eq!("*3\r\n$3\r\nset\r\n$1\r\na\r\n$3\r\n123\r\n",vp2);
    ///
    /// // Simple
    /// let vp3 = ValuePair::new(Value::String(b"FULLRESYNC 6344847b8f323346bcd72a64a9d8a1a47a6b1249 18004".to_vec()),
    ///      LinkedHashMap::new()).
    ///     to_resp_string().
    ///     unwrap();
    /// assert_eq!("+FULLRESYNC 6344847b8f323346bcd72a64a9d8a1a47a6b1249 18004\r\n",vp3);
    ///
    /// // Map
    /// let mut map = LinkedHashMap::new();
    /// map.insert(
    ///         Value::Blob(b"I am key".to_vec()),
    ///          Value::Array(vec![
    ///             Value::Blob(b"I".to_vec()),
    ///             Value::Blob(b"am".to_vec()),
    ///             Value::Blob(b"Value".to_vec())]));
    /// let vp4 = ValuePair::new(
    ///     Value::Map(map),
    ///     LinkedHashMap::new(),
    ///     ).to_resp_string().unwrap();
    /// assert_eq!("%1\r\n$8\r\nI am key\r\n*3\r\n$1\r\nI\r\n$2\r\nam\r\n$5\r\nValue\r\n",vp4);
    /// ```
    pub fn to_resp_string(&self) -> Result<String, std::fmt::Error> {
        let mut buf = String::new();
        // check attributes
        if self.attrs.len() > 0 {
            buf.write_char(resp_event_type::ATTRIBUTE as char)?;
            buf.write_str(&format!("{}", self.attrs.len()))?;
            buf.write_str("\r\n")?;

            for item in self.attrs.iter() {
                let (k, v) = (item.0, item.1);
                buf.write_str(
                    std::str::from_utf8(&k.as_bytes()).expect("Key found invalid UTF-8"),
                )?;
                buf.write_str(
                    std::str::from_utf8(&v.as_bytes()).expect("Value found invalid UTF-8"),
                )?;
            }
        }
        buf.write_str(
            std::str::from_utf8(&self.value.as_bytes()).expect("Value found invalid UTF-8"),
        )?;

        Ok(buf)
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
    /// VerbatimString =<length>\r\n<format(3 bytes):><bytes>\r\n
    Verbatimstring(String),
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
    // / A streaming long connection transmission
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
    /// assert_eq!(b":1\r\n".to_vec(), number_value.as_bytes());
    ///
    /// let blob_value = Value::Blob(b"123".to_vec());
    /// assert_eq!(b"$3\r\n123\r\n".to_vec(), blob_value.as_bytes());
    ///
    /// let boolean_value = Value::Boolean(true);
    /// assert_eq!(b"#t\r\n".to_vec(), boolean_value.as_bytes());
    ///
    /// let blob_error = Value::BlobError("woca".to_string());
    /// assert_eq!(b"!4\r\nwoca\r\n".to_vec(), blob_error.as_bytes());
    ///
    /// let double_inf_value = Value::Double(Float64::from(std::f64::INFINITY));
    /// assert_eq!(b",inf\r\n".to_vec(), double_inf_value.as_bytes());
    ///
    /// let double_neg_inf_value = Value::Double(Float64::from(std::f64::NEG_INFINITY));
    /// assert_eq!(b",-inf\r\n".to_vec(), double_neg_inf_value.as_bytes());
    ///
    /// let double_value = Value::Double(Float64::from(1.23_f64));
    /// assert_eq!(b",1.23\r\n".to_vec(), double_value.as_bytes());
    ///
    /// let bigint_type = BigInt::parse_bytes(b"3492890328409238509324850943850943825024385",10).unwrap();
    /// let bigint_value = Value::Bigint(bigint_type);
    /// assert_eq!(b"(3492890328409238509324850943850943825024385\r\n".to_vec(), bigint_value.as_bytes());
    ///
    /// let verstring = Value::Verbatimstring("Some string".to_string());
    /// assert_eq!(b"=11\r\ntxt:Some string\r\n".to_vec(),verstring.as_bytes());
    ///
    /// let mut map = LinkedHashMap::new();
    /// map.insert(Value::String(b"first".to_vec()),Value::Number(1));
    /// map.insert(Value::String(b"second".to_vec()),Value::Number(2));
    /// let value_map = Value::Map(map);
    /// assert_eq!(b"%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n".to_vec(), value_map.as_bytes());
    ///
    /// let value_set = Value::Set(vec![
    ///         Value::String(b"orange".to_vec()),
    ///         Value::String(b"apple".to_vec()),
    ///         Value::Boolean(true),
    ///         Value::Number(100),
    ///         Value::Number(999),
    ///     ]);
    /// assert_eq!(b"~+orange\r\n+apple\r\n#t\r\n:100\r\n:999\r\n".to_vec(),value_set.as_bytes());
    ///
    ///
    /// let push_value = Value::Push(
    ///     vec![
    ///         Value::String(b"pubsub".to_vec()),
    ///         Value::String(b"message".to_vec()),
    ///         Value::String(b"somechannel".to_vec()),
    ///         Value::String(b"this is the message".to_vec()),
    ///     ]);
    /// assert_eq!(b">4\r\n+pubsub\r\n+message\r\n+somechannel\r\n+this is the message\r\n".to_vec(),push_value.as_bytes());
    /// ```
    /// Serializes the value into an array of bytes using Redis protocol.
    pub fn as_bytes(&self) -> Vec<u8> {
        return match *self {
            Value::Null => [&b"_"[..], &"\r\n".to_owned().into_bytes()[..]].concat(),

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

            Value::Error(ref d) => [
                &b"-"[..],
                (*d).as_bytes(),
                &"\r\n".to_owned().into_bytes()[..],
            ]
            .concat(),

            Value::Array(ref a) => [
                &b"*"[..],
                &format!("{}\r\n", a.len()).into_bytes()[..],
                &(a.iter().map(|el| el.as_bytes()).collect::<Vec<_>>()[..].concat())[..],
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

            Value::Verbatimstring(ref s) => [
                &b"="[..],
                &format!("{}\r\n", s.len()).into_bytes()[..],
                &format!("txt:{}\r\n", &s[..]).into_bytes()[..],
            ]
            .concat(),

            Value::Bigint(ref b) => [&b"("[..], &format!("{}\r\n", &b).into_bytes()[..]].concat(),

            Value::Map(ref m) => {
                let mut content: Vec<u8> = Vec::new();
                for x in m.iter() {
                    for k in x.0.as_bytes().iter() {
                        content.push(*k);
                    }
                    for v in x.1.as_bytes().iter() {
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
                    for c in x.as_bytes().iter() {
                        content.push(*c);
                    }
                }
                [&b"~"[..], &content[..]].concat()
            }

            Value::Attribute(ref m) => {
                let mut content: Vec<u8> = Vec::new();
                for x in m.iter() {
                    for k in x.0.as_bytes().iter() {
                        content.push(*k);
                    }
                    for v in x.1.as_bytes().iter() {
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
                    for x in i.as_bytes().iter() {
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
    /// get type literal
    fn get_char(&self) -> char {
        return match *self {
            Value::Blob(_) => resp_event_type::BLOB_STRING,
            Value::String(_) => resp_event_type::SIMPLE_STRING,
            Value::Error(_) => resp_event_type::SIMPLE_ERROR,
            Value::Number(_) => resp_event_type::NUMBER,
            Value::Null => resp_event_type::NULL,
            Value::Double(_) => resp_event_type::DOUBLE,
            Value::Boolean(_) => resp_event_type::BOOLEAN,
            Value::BlobError(_) => resp_event_type::BLOB_ERROR,
            Value::Verbatimstring(_) => resp_event_type::VERBATIM_STRING,
            Value::Bigint(_) => resp_event_type::BIG_INT,
            Value::Array(_) => resp_event_type::ARRAY,
            Value::Map(_) => resp_event_type::MAP,
            Value::Set(_) => resp_event_type::SET,
            Value::Attribute(_) => resp_event_type::ATTRIBUTE,
            Value::Push(_) => resp_event_type::PUSH,
        };
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }
}
