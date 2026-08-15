// go: file float64.go decls: newFloat64Value, float64Conv, Float64Var, Float64VarP, Float64, Float64P
//
// PARTIAL file: float64.go's Value impl and FlagSet methods are in
// lib.rs, so the manifest above lists only what lives here.
//
// float64.go — the Value impl and the FlagSet methods live in lib.rs (this
// crate predates the one-module-per-file split); what lands here is the
// rest of the Go file: the constructor under its Go name, the conv func
// getFlagType drives, and the four package-level entry points.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 float64.go:8-11 newFloat64Value
pub fn newFloat64Value(val: float64, p: *mut float64) -> float64Value {
    return float64Value::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 float64.go:25-27 float64Conv
pub fn float64Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseFloat(sval, 64);
    return (goish::goany::Any::new(v), err);
}

// go: github.com/spf13/pflag@v1.0.10 float64.go:51-53 Float64Var
pub fn Float64Var(p: *mut float64, name: string, value: float64, usage: string) {
    COMMAND_LINE.Lock().Float64VarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float64.go:56-58 Float64VarP
pub fn Float64VarP(p: *mut float64, name: string, shorthand: string, value: float64, usage: string) {
    COMMAND_LINE.Lock().Float64VarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float64.go:77-79 Float64
pub fn Float64(name: string, value: float64, usage: string) -> *mut float64 {
    return COMMAND_LINE.Lock().Float64P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float64.go:82-84 Float64P
pub fn Float64P(name: string, shorthand: string, value: float64, usage: string) -> *mut float64 {
    return COMMAND_LINE.Lock().Float64P(name, shorthand, value, usage);
}
