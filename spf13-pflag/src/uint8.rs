// go: file uint8.go decls: uint8Value, newUint8Value, uint8Value.Set, uint8Value.Type, uint8Value.String, uint8Conv, FlagSet.GetUint8, FlagSet.Uint8Var, FlagSet.Uint8VarP, Uint8Var, Uint8VarP, FlagSet.Uint8, FlagSet.Uint8P, Uint8, Uint8P
//
// uint8.go — same shape as int8.go, which carries the commentary for the
// whole per-type family.

use crate::*;
// Explicit: `use crate::*` also pulls in the module named `uint8`, and a
// glob-imported module shadows the glob-imported TYPE of the same name.
use goish::types::uint8;

// go: github.com/spf13/pflag@v1.0.10 uint8.go:6-6 uint8Value
pub struct uint8Value {
    ptr: *mut uint8,
}
unsafe impl Send for uint8Value {}
unsafe impl Sync for uint8Value {}

// go: github.com/spf13/pflag@v1.0.10 uint8.go:8-11 newUint8Value
pub fn newUint8Value(val: uint8, p: *mut uint8) -> uint8Value {
    unsafe {
        *p = val;
    }
    return uint8Value { ptr: p };
}

impl Value for uint8Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(uint8Value { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 uint8.go:23-23 uint8Value.String
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        return strconv::FormatUint(uint(v as u64), 10);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint8.go:13-17 uint8Value.Set
    /// Go assigns before returning err, so a parse failure still stores
    /// the parser's zero — kept, because the deviation would be silent.
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseUint(s, 0, 8);
        unsafe {
            *self.ptr = v as uint8;
        }
        return err;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint8.go:19-21 uint8Value.Type
    fn Type(&self) -> string {
        return string("uint8");
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint8.go:25-31 uint8Conv
/// Go's error arm returns an untyped `0`, which lands in the interface
/// as an `int`; this returns the zero of the flag's own type so the
/// caller's `As::<uint8>()` cannot mis-read a discarded value.
pub fn uint8Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseUint(sval, 0, 8);
    if err != nil {
        return (goish::goany::Any::new(0u8), err);
    }
    return (goish::goany::Any::new(v as uint8), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 uint8.go:34-40 FlagSet.GetUint8
    pub fn GetUint8<S: Into<string>>(&self, name: S) -> (uint8, error) {
        let (val, err) = self.getFlagType(name.into(), string("uint8"), uint8Conv);
        if err != nil {
            return (0u8, err);
        }
        return (*val.As::<uint8>().unwrap_or(&0u8), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 uint8.go:44-46 FlagSet.Uint8Var
    pub fn Uint8Var(&mut self, p: *mut uint8, name: string, value: uint8, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUint8Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint8.go:49-51 FlagSet.Uint8VarP
    pub fn Uint8VarP(&mut self, p: *mut uint8, name: string, shorthand: string, value: uint8, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUint8Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint8.go:66-70 FlagSet.Uint8
    pub fn Uint8(&mut self, name: string, value: uint8, usage: string) -> *mut uint8 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0u8));
        self.Uint8VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint8.go:73-77 FlagSet.Uint8P
    pub fn Uint8P(&mut self, name: string, shorthand: string, value: uint8, usage: string) -> *mut uint8 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0u8));
        self.Uint8VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint8.go:55-57 Uint8Var
pub fn Uint8Var(p: *mut uint8, name: string, value: uint8, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUint8Value(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint8.go:60-62 Uint8VarP
pub fn Uint8VarP(p: *mut uint8, name: string, shorthand: string, value: uint8, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUint8Value(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint8.go:81-83 Uint8
pub fn Uint8(name: string, value: uint8, usage: string) -> *mut uint8 {
    return COMMAND_LINE.Lock().Uint8P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint8.go:86-88 Uint8P
pub fn Uint8P(name: string, shorthand: string, value: uint8, usage: string) -> *mut uint8 {
    return COMMAND_LINE.Lock().Uint8P(name, shorthand, value, usage);
}
