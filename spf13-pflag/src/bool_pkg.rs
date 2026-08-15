// go: file bool.go decls: newBoolValue, boolValue.IsBoolFlag, boolConv, BoolVar, BoolVarP, Bool, BoolP
//
// PARTIAL file: bool.go's Value impl and FlagSet methods are in lib.rs,
// so the manifest above lists only what lives here.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 bool.go:15-18 newBoolValue
pub fn newBoolValue(val: bool, p: *mut bool) -> boolValue {
    return boolValue::new(p, val);
}

impl boolValue {
    // go: github.com/spf13/pflag@v1.0.10 bool.go:32-32 boolValue.IsBoolFlag
    /// Go: the marker that lets `--flag` stand alone with no value.
    pub fn IsBoolFlag(&self) -> bool {
        return true;
    }
}

// go: github.com/spf13/pflag@v1.0.10 bool.go:34-36 boolConv
pub fn boolConv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseBool(sval);
    return (goish::goany::Any::new(v), err);
}

// go: github.com/spf13/pflag@v1.0.10 bool.go:61-63 BoolVar
pub fn BoolVar(p: *mut bool, name: string, value: bool, usage: string) {
    COMMAND_LINE.Lock().BoolVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bool.go:66-69 BoolVarP
pub fn BoolVarP(p: *mut bool, name: string, shorthand: string, value: bool, usage: string) {
    COMMAND_LINE.Lock().BoolVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bool.go:86-88 Bool
pub fn Bool(name: string, value: bool, usage: string) -> *mut bool {
    return COMMAND_LINE.Lock().BoolP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bool.go:91-94 BoolP
pub fn BoolP(name: string, shorthand: string, value: bool, usage: string) -> *mut bool {
    return COMMAND_LINE.Lock().BoolP(name, shorthand, value, usage);
}
