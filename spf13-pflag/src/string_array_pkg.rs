// go: file string_array.go decls: newStringArrayValue, stringArrayConv
//
// PARTIAL file: stringArrayValue's impl and the FlagSet methods are in
// lib.rs.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 string_array.go:9-14 newStringArrayValue
pub fn newStringArrayValue(val: slice<string>, p: *mut slice<string>) -> stringArrayValue {
    return stringArrayValue::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 string_array.go:57-64 stringArrayConv
pub fn stringArrayConv(sval: string) -> (goish::goany::Any, error) {
    let raw: &str = sval.as_ref();
    let inner = if raw.len() >= 2 {
        string::from_bytes(&raw.as_bytes()[1..raw.len() - 1])
    } else {
        string("")
    };
    // Go: "An empty string would cause a array with one (empty) string"
    if inner.Len() == 0 {
        let empty: slice<string> = make!([]string, 0);
        return (goish::goany::Any::new(empty), nil.into());
    }
    let (v, err) = readAsCSV(inner);
    return (goish::goany::Any::new(v), err);
}
