// go: file string_slice.go decls: newStringSliceValue, stringSliceConv
//
// PARTIAL file: stringSliceValue's impl and the FlagSet methods are in
// lib.rs, and readAsCSV/writeAsCSV live there too. This holds the rest.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 string_slice.go:15-20 newStringSliceValue
pub fn newStringSliceValue(val: slice<string>, p: *mut slice<string>) -> stringSliceValue {
    return stringSliceValue::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 string_slice.go:79-86 stringSliceConv
/// Go slices off the surrounding brackets with `sval[1 : len(sval)-1]`,
/// which assumes they are there — it is fed Value.String() output.
pub fn stringSliceConv(sval: string) -> (goish::goany::Any, error) {
    let raw: &str = sval.as_ref();
    let inner = if raw.len() >= 2 {
        string::from_bytes(&raw.as_bytes()[1..raw.len() - 1])
    } else {
        string("")
    };
    // Go: "An empty string would cause a slice with one (empty) string"
    if inner.Len() == 0 {
        let empty: slice<string> = make!([]string, 0);
        return (goish::goany::Any::new(empty), nil.into());
    }
    let (v, err) = readAsCSV(inner);
    return (goish::goany::Any::new(v), err);
}
