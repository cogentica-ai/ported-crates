// go: file uint64.go decls: uint64Value, newUint64Value, uint64Value.Set, uint64Value.Type, uint64Value.String, uint64Conv, FlagSet.GetUint64, FlagSet.Uint64Var, FlagSet.Uint64VarP, Uint64Var, Uint64VarP, FlagSet.Uint64, FlagSet.Uint64P, Uint64, Uint64P
//
// uint64.go — same shape as int8.go, which carries the commentary for the
// whole per-type family.

use crate::*;
// Explicit: `use crate::*` also pulls in the module named `uint64`, and a
// glob-imported module shadows the glob-imported TYPE of the same name.
use goish::types::uint64;

// go: github.com/spf13/pflag@v1.0.10 uint64.go:6-6 uint64Value
pub struct uint64Value {
    ptr: *mut uint64,
}
unsafe impl Send for uint64Value {}
unsafe impl Sync for uint64Value {}

// go: github.com/spf13/pflag@v1.0.10 uint64.go:8-11 newUint64Value
pub fn newUint64Value(val: uint64, p: *mut uint64) -> uint64Value {
    unsafe {
        *p = val;
    }
    return uint64Value { ptr: p };
}

impl Value for uint64Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(uint64Value { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 uint64.go:23-23 uint64Value.String
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        return strconv::FormatUint(uint(v as u64), 10);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint64.go:13-17 uint64Value.Set
    /// Go assigns before returning err, so a parse failure still stores
    /// the parser's zero — kept, because the deviation would be silent.
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseUint(s, 0, 64);
        unsafe {
            *self.ptr = v as uint64;
        }
        return err;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint64.go:19-21 uint64Value.Type
    fn Type(&self) -> string {
        return string("uint64");
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint64.go:25-31 uint64Conv
/// Go's error arm returns an untyped `0`, which lands in the interface
/// as an `int`; this returns the zero of the flag's own type so the
/// caller's `As::<uint64>()` cannot mis-read a discarded value.
pub fn uint64Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseUint(sval, 0, 64);
    if err != nil {
        return (goish::goany::Any::new(0u64), err);
    }
    return (goish::goany::Any::new(v as uint64), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 uint64.go:34-40 FlagSet.GetUint64
    pub fn GetUint64<S: Into<string>>(&self, name: S) -> (uint64, error) {
        let (val, err) = self.getFlagType(name.into(), string("uint64"), uint64Conv);
        if err != nil {
            return (0u64, err);
        }
        return (*val.As::<uint64>().unwrap_or(&0u64), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 uint64.go:44-46 FlagSet.Uint64Var
    pub fn Uint64Var(&mut self, p: *mut uint64, name: string, value: uint64, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUint64Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint64.go:49-51 FlagSet.Uint64VarP
    pub fn Uint64VarP(&mut self, p: *mut uint64, name: string, shorthand: string, value: uint64, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUint64Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint64.go:66-70 FlagSet.Uint64
    pub fn Uint64(&mut self, name: string, value: uint64, usage: string) -> *mut uint64 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0u64));
        self.Uint64VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint64.go:73-77 FlagSet.Uint64P
    pub fn Uint64P(&mut self, name: string, shorthand: string, value: uint64, usage: string) -> *mut uint64 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0u64));
        self.Uint64VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint64.go:55-57 Uint64Var
pub fn Uint64Var(p: *mut uint64, name: string, value: uint64, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUint64Value(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint64.go:60-62 Uint64VarP
pub fn Uint64VarP(p: *mut uint64, name: string, shorthand: string, value: uint64, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUint64Value(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint64.go:81-83 Uint64
pub fn Uint64(name: string, value: uint64, usage: string) -> *mut uint64 {
    return COMMAND_LINE.Lock().Uint64P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint64.go:86-88 Uint64P
pub fn Uint64P(name: string, shorthand: string, value: uint64, usage: string) -> *mut uint64 {
    return COMMAND_LINE.Lock().Uint64P(name, shorthand, value, usage);
}
