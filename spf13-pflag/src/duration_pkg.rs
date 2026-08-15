// go: file duration.go decls: newDurationValue, durationConv, DurationVar, DurationVarP, Duration, DurationP
//
// PARTIAL file: duration.go's Value impl and FlagSet methods are in lib.rs.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 duration.go:10-13 newDurationValue
pub fn newDurationValue(val: time::Duration, p: *mut time::Duration) -> durationValue {
    return durationValue::new(p, val);
}

// go: github.com/spf13/pflag@v1.0.10 duration.go:27-29 durationConv
pub fn durationConv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = time::ParseDuration(sval);
    return (goish::goany::Any::new(v), err);
}

// go: github.com/spf13/pflag@v1.0.10 duration.go:53-55 DurationVar
pub fn DurationVar(p: *mut time::Duration, name: string, value: time::Duration, usage: string) {
    COMMAND_LINE.Lock().DurationVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 duration.go:58-60 DurationVarP
pub fn DurationVarP(p: *mut time::Duration, name: string, shorthand: string, value: time::Duration, usage: string) {
    COMMAND_LINE.Lock().DurationVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 duration.go:79-81 Duration
pub fn Duration(name: string, value: time::Duration, usage: string) -> *mut time::Duration {
    return COMMAND_LINE.Lock().DurationP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 duration.go:84-86 DurationP
pub fn DurationP(name: string, shorthand: string, value: time::Duration, usage: string) -> *mut time::Duration {
    return COMMAND_LINE.Lock().DurationP(name, shorthand, value, usage);
}
