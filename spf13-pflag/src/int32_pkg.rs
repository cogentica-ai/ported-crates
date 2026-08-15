// go: file int32.go decls: newInt32Value, int32Conv, Int32Var, Int32VarP, Int32, Int32P
//
// PARTIAL file: int32.go's Value impl and FlagSet methods are in
// lib.rs, so the manifest above lists only what lives here.
//
// int32.go — the Value impl and the FlagSet methods live in lib.rs (this
// crate predates the one-module-per-file split); what lands here is the
// rest of the Go file: the constructor under its Go name, the conv func
// getFlagType drives, and the four package-level entry points.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 int32.go:8-11 newInt32Value
pub fn newInt32Value(val: i32, p: *mut i32) -> int32Value {
    return int32Value::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 int32.go:25-31 int32Conv
pub fn int32Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseInt(sval, 0, 32);
    if err != nil {
        return (goish::goany::Any::new(0i32), err);
    }
    return (goish::goany::Any::new(v as i32), nil.into());
}

// go: github.com/spf13/pflag@v1.0.10 int32.go:55-57 Int32Var
pub fn Int32Var(p: *mut i32, name: string, value: i32, usage: string) {
    COMMAND_LINE.Lock().Int32VarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int32.go:60-62 Int32VarP
pub fn Int32VarP(p: *mut i32, name: string, shorthand: string, value: i32, usage: string) {
    COMMAND_LINE.Lock().Int32VarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int32.go:81-83 Int32
pub fn Int32(name: string, value: i32, usage: string) -> *mut i32 {
    return COMMAND_LINE.Lock().Int32P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int32.go:86-88 Int32P
pub fn Int32P(name: string, shorthand: string, value: i32, usage: string) -> *mut i32 {
    return COMMAND_LINE.Lock().Int32P(name, shorthand, value, usage);
}
