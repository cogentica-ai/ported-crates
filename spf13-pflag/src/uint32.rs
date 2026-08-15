// go: file uint32.go decls: uint32Value, newUint32Value, uint32Value.Set, uint32Value.Type, uint32Value.String, uint32Conv, FlagSet.GetUint32, FlagSet.Uint32Var, FlagSet.Uint32VarP, Uint32Var, Uint32VarP, FlagSet.Uint32, FlagSet.Uint32P, Uint32, Uint32P
//
// uint32.go — same shape as int8.go, which carries the commentary for the
// whole per-type family.

use crate::*;
// Explicit: `use crate::*` also pulls in the module named `uint32`, and a
// glob-imported module shadows the glob-imported TYPE of the same name.
use goish::types::uint32;

// go: github.com/spf13/pflag@v1.0.10 uint32.go:6-6 uint32Value
pub struct uint32Value {
    ptr: *mut uint32,
}
unsafe impl Send for uint32Value {}
unsafe impl Sync for uint32Value {}

// go: github.com/spf13/pflag@v1.0.10 uint32.go:8-11 newUint32Value
pub fn newUint32Value(val: uint32, p: *mut uint32) -> uint32Value {
    unsafe {
        *p = val;
    }
    return uint32Value { ptr: p };
}

impl Value for uint32Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(uint32Value { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 uint32.go:23-23 uint32Value.String
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        return strconv::FormatUint(uint(v as u64), 10);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint32.go:13-17 uint32Value.Set
    /// Go assigns before returning err, so a parse failure still stores
    /// the parser's zero — kept, because the deviation would be silent.
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseUint(s, 0, 32);
        unsafe {
            *self.ptr = v as uint32;
        }
        return err;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint32.go:19-21 uint32Value.Type
    fn Type(&self) -> string {
        return string("uint32");
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint32.go:25-31 uint32Conv
/// Go's error arm returns an untyped `0`, which lands in the interface
/// as an `int`; this returns the zero of the flag's own type so the
/// caller's `As::<uint32>()` cannot mis-read a discarded value.
pub fn uint32Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseUint(sval, 0, 32);
    if err != nil {
        return (goish::goany::Any::new(0u32), err);
    }
    return (goish::goany::Any::new(v as uint32), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 uint32.go:34-40 FlagSet.GetUint32
    pub fn GetUint32<S: Into<string>>(&self, name: S) -> (uint32, error) {
        let (val, err) = self.getFlagType(name.into(), string("uint32"), uint32Conv);
        if err != nil {
            return (0u32, err);
        }
        return (*val.As::<uint32>().unwrap_or(&0u32), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 uint32.go:44-46 FlagSet.Uint32Var
    pub fn Uint32Var(&mut self, p: *mut uint32, name: string, value: uint32, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUint32Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint32.go:49-51 FlagSet.Uint32VarP
    pub fn Uint32VarP(&mut self, p: *mut uint32, name: string, shorthand: string, value: uint32, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUint32Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint32.go:66-70 FlagSet.Uint32
    pub fn Uint32(&mut self, name: string, value: uint32, usage: string) -> *mut uint32 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0u32));
        self.Uint32VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint32.go:73-77 FlagSet.Uint32P
    pub fn Uint32P(&mut self, name: string, shorthand: string, value: uint32, usage: string) -> *mut uint32 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0u32));
        self.Uint32VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint32.go:55-57 Uint32Var
pub fn Uint32Var(p: *mut uint32, name: string, value: uint32, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUint32Value(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint32.go:60-62 Uint32VarP
pub fn Uint32VarP(p: *mut uint32, name: string, shorthand: string, value: uint32, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUint32Value(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint32.go:81-83 Uint32
pub fn Uint32(name: string, value: uint32, usage: string) -> *mut uint32 {
    return COMMAND_LINE.Lock().Uint32P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint32.go:86-88 Uint32P
pub fn Uint32P(name: string, shorthand: string, value: uint32, usage: string) -> *mut uint32 {
    return COMMAND_LINE.Lock().Uint32P(name, shorthand, value, usage);
}
