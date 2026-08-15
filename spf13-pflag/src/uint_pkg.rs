// go: file uint.go decls: newUintValue, uintConv, UintVar, UintVarP, Uint, UintP
//
// PARTIAL file: uint.go's Value impl and FlagSet methods are in
// lib.rs, so the manifest above lists only what lives here.
//
// uint.go — the Value impl and the FlagSet methods live in lib.rs (this
// crate predates the one-module-per-file split); what lands here is the
// rest of the Go file: the constructor under its Go name, the conv func
// getFlagType drives, and the four package-level entry points.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 uint.go:8-11 newUintValue
pub fn newUintValue(val: uint, p: *mut uint) -> uintValue {
    return uintValue::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 uint.go:25-31 uintConv
pub fn uintConv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseUint(sval, 0, 0);
    if err != nil {
        return (goish::goany::Any::new(0u64), err);
    }
    return (goish::goany::Any::new(v as uint), nil.into());
}

// go: github.com/spf13/pflag@v1.0.10 uint.go:55-57 UintVar
pub fn UintVar(p: *mut uint, name: string, value: uint, usage: string) {
    COMMAND_LINE.Lock().UintVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint.go:60-62 UintVarP
pub fn UintVarP(p: *mut uint, name: string, shorthand: string, value: uint, usage: string) {
    COMMAND_LINE.Lock().UintVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint.go:81-83 Uint
pub fn Uint(name: string, value: uint, usage: string) -> *mut uint {
    return COMMAND_LINE.Lock().UintP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint.go:86-88 UintP
pub fn UintP(name: string, shorthand: string, value: uint, usage: string) -> *mut uint {
    return COMMAND_LINE.Lock().UintP(name, shorthand, value, usage);
}
