// go: file int64.go decls: newInt64Value, int64Conv, Int64Var, Int64VarP, Int64, Int64P
//
// PARTIAL file: int64.go's Value impl and FlagSet methods are in
// lib.rs, so the manifest above lists only what lives here.
//
// int64.go — the Value impl and the FlagSet methods live in lib.rs (this
// crate predates the one-module-per-file split); what lands here is the
// rest of the Go file: the constructor under its Go name, the conv func
// getFlagType drives, and the four package-level entry points.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 int64.go:8-11 newInt64Value
pub fn newInt64Value(val: i64, p: *mut i64) -> int64Value {
    return int64Value::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 int64.go:25-27 int64Conv
pub fn int64Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseInt(sval, 0, 64);
    return (goish::goany::Any::new(v), err);
}

// go: github.com/spf13/pflag@v1.0.10 int64.go:51-53 Int64Var
pub fn Int64Var(p: *mut i64, name: string, value: i64, usage: string) {
    COMMAND_LINE.Lock().Int64VarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int64.go:56-58 Int64VarP
pub fn Int64VarP(p: *mut i64, name: string, shorthand: string, value: i64, usage: string) {
    COMMAND_LINE.Lock().Int64VarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int64.go:77-79 Int64
pub fn Int64(name: string, value: i64, usage: string) -> *mut i64 {
    return COMMAND_LINE.Lock().Int64P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int64.go:82-84 Int64P
pub fn Int64P(name: string, shorthand: string, value: i64, usage: string) -> *mut i64 {
    return COMMAND_LINE.Lock().Int64P(name, shorthand, value, usage);
}
