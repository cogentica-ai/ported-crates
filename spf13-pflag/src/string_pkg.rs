// go: file string.go decls: newStringValue, stringConv, StringVar, StringVarP, String, StringP
//
// PARTIAL file: string.go's Value impl and FlagSet methods are in lib.rs.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 string.go:6-9 newStringValue
pub fn newStringValue(val: string, p: *mut string) -> stringValue {
    return stringValue::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 string.go:21-23 stringConv
/// Go returns the string unchanged — a string flag needs no parse.
pub fn stringConv(sval: string) -> (goish::goany::Any, error) {
    return (goish::goany::Any::new(sval), nil.into());
}

// go: github.com/spf13/pflag@v1.0.10 string.go:47-49 StringVar
pub fn StringVar(p: *mut string, name: string, value: string, usage: string) {
    COMMAND_LINE.Lock().StringVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string.go:52-54 StringVarP
pub fn StringVarP(p: *mut string, name: string, shorthand: string, value: string, usage: string) {
    COMMAND_LINE.Lock().StringVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string.go:73-75 String
pub fn String(name: string, value: string, usage: string) -> *mut string {
    return COMMAND_LINE.Lock().StringP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string.go:78-80 StringP
pub fn StringP(name: string, shorthand: string, value: string, usage: string) -> *mut string {
    return COMMAND_LINE.Lock().StringP(name, shorthand, value, usage);
}
