// go: file int.go decls: newIntValue, intConv, IntVar, IntVarP, Int, IntP
//
// PARTIAL file: int.go's Value impl and FlagSet methods are in
// lib.rs, so the manifest above lists only what lives here.
//
// int.go — the Value impl and the FlagSet methods live in lib.rs (this
// crate predates the one-module-per-file split); what lands here is the
// rest of the Go file: the constructor under its Go name, the conv func
// getFlagType drives, and the four package-level entry points.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 int.go:8-11 newIntValue
pub fn newIntValue(val: int, p: *mut int) -> intValue {
    return intValue::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 int.go:25-27 intConv
pub fn intConv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::Atoi(sval);
    return (goish::goany::Any::new(v), err);
}

// go: github.com/spf13/pflag@v1.0.10 int.go:51-53 IntVar
pub fn IntVar(p: *mut int, name: string, value: int, usage: string) {
    COMMAND_LINE.Lock().IntVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int.go:56-58 IntVarP
pub fn IntVarP(p: *mut int, name: string, shorthand: string, value: int, usage: string) {
    COMMAND_LINE.Lock().IntVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int.go:77-79 Int
pub fn Int(name: string, value: int, usage: string) -> *mut int {
    return COMMAND_LINE.Lock().IntP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int.go:82-84 IntP
pub fn IntP(name: string, shorthand: string, value: int, usage: string) -> *mut int {
    return COMMAND_LINE.Lock().IntP(name, shorthand, value, usage);
}
