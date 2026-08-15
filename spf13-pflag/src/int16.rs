// go: file int16.go decls: int16Value, newInt16Value, int16Value.Set, int16Value.Type, int16Value.String, int16Conv, FlagSet.GetInt16, FlagSet.Int16Var, FlagSet.Int16VarP, Int16Var, Int16VarP, FlagSet.Int16, FlagSet.Int16P, Int16, Int16P
//
// int16.go — same shape as int8.go, which carries the commentary for the
// whole per-type family.

use crate::*;
// Explicit: `use crate::*` also pulls in the module named `int16`, and a
// glob-imported module shadows the glob-imported TYPE of the same name.
use goish::types::int16;

// go: github.com/spf13/pflag@v1.0.10 int16.go:6-6 int16Value
pub struct int16Value {
    ptr: *mut int16,
}
unsafe impl Send for int16Value {}
unsafe impl Sync for int16Value {}

// go: github.com/spf13/pflag@v1.0.10 int16.go:8-11 newInt16Value
pub fn newInt16Value(val: int16, p: *mut int16) -> int16Value {
    unsafe {
        *p = val;
    }
    return int16Value { ptr: p };
}

impl Value for int16Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(int16Value { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 int16.go:23-23 int16Value.String
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        return strconv::FormatInt(int(v as i64), 10);
    }

    // go: github.com/spf13/pflag@v1.0.10 int16.go:13-17 int16Value.Set
    /// Go assigns before returning err, so a parse failure still stores
    /// the parser's zero — kept, because the deviation would be silent.
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseInt(s, 0, 16);
        unsafe {
            *self.ptr = v as int16;
        }
        return err;
    }

    // go: github.com/spf13/pflag@v1.0.10 int16.go:19-21 int16Value.Type
    fn Type(&self) -> string {
        return string("int16");
    }
}

// go: github.com/spf13/pflag@v1.0.10 int16.go:25-31 int16Conv
/// Go's error arm returns an untyped `0`, which lands in the interface
/// as an `int`; this returns the zero of the flag's own type so the
/// caller's `As::<int16>()` cannot mis-read a discarded value.
pub fn int16Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseInt(sval, 0, 16);
    if err != nil {
        return (goish::goany::Any::new(0i16), err);
    }
    return (goish::goany::Any::new(v as int16), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 int16.go:34-40 FlagSet.GetInt16
    pub fn GetInt16<S: Into<string>>(&self, name: S) -> (int16, error) {
        let (val, err) = self.getFlagType(name.into(), string("int16"), int16Conv);
        if err != nil {
            return (0i16, err);
        }
        return (*val.As::<int16>().unwrap_or(&0i16), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 int16.go:44-46 FlagSet.Int16Var
    pub fn Int16Var(&mut self, p: *mut int16, name: string, value: int16, usage: string) {
        self.VarP(alloc::boxed::Box::new(newInt16Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int16.go:49-51 FlagSet.Int16VarP
    pub fn Int16VarP(&mut self, p: *mut int16, name: string, shorthand: string, value: int16, usage: string) {
        self.VarP(alloc::boxed::Box::new(newInt16Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int16.go:66-70 FlagSet.Int16
    pub fn Int16(&mut self, name: string, value: int16, usage: string) -> *mut int16 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0i16));
        self.Int16VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 int16.go:73-77 FlagSet.Int16P
    pub fn Int16P(&mut self, name: string, shorthand: string, value: int16, usage: string) -> *mut int16 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0i16));
        self.Int16VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 int16.go:55-57 Int16Var
pub fn Int16Var(p: *mut int16, name: string, value: int16, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newInt16Value(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 int16.go:60-62 Int16VarP
pub fn Int16VarP(p: *mut int16, name: string, shorthand: string, value: int16, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newInt16Value(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int16.go:81-83 Int16
pub fn Int16(name: string, value: int16, usage: string) -> *mut int16 {
    return COMMAND_LINE.Lock().Int16P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int16.go:86-88 Int16P
pub fn Int16P(name: string, shorthand: string, value: int16, usage: string) -> *mut int16 {
    return COMMAND_LINE.Lock().Int16P(name, shorthand, value, usage);
}
