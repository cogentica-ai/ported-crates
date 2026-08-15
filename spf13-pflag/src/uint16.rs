// go: file uint16.go decls: uint16Value, newUint16Value, uint16Value.Set, uint16Value.Type, uint16Value.String, uint16Conv, FlagSet.GetUint16, FlagSet.Uint16Var, FlagSet.Uint16VarP, Uint16Var, Uint16VarP, FlagSet.Uint16, FlagSet.Uint16P, Uint16, Uint16P
//
// uint16.go — same shape as int8.go, which carries the commentary for the
// whole per-type family.

use crate::*;
// Explicit: `use crate::*` also pulls in the module named `uint16`, and a
// glob-imported module shadows the glob-imported TYPE of the same name.
use goish::types::uint16;

// go: github.com/spf13/pflag@v1.0.10 uint16.go:6-6 uint16Value
pub struct uint16Value {
    ptr: *mut uint16,
}
unsafe impl Send for uint16Value {}
unsafe impl Sync for uint16Value {}

// go: github.com/spf13/pflag@v1.0.10 uint16.go:8-11 newUint16Value
pub fn newUint16Value(val: uint16, p: *mut uint16) -> uint16Value {
    unsafe {
        *p = val;
    }
    return uint16Value { ptr: p };
}

impl Value for uint16Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(uint16Value { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 uint16.go:23-23 uint16Value.String
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        return strconv::FormatUint(uint(v as u64), 10);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint16.go:13-17 uint16Value.Set
    /// Go assigns before returning err, so a parse failure still stores
    /// the parser's zero — kept, because the deviation would be silent.
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseUint(s, 0, 16);
        unsafe {
            *self.ptr = v as uint16;
        }
        return err;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint16.go:19-21 uint16Value.Type
    fn Type(&self) -> string {
        return string("uint16");
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint16.go:25-31 uint16Conv
/// Go's error arm returns an untyped `0`, which lands in the interface
/// as an `int`; this returns the zero of the flag's own type so the
/// caller's `As::<uint16>()` cannot mis-read a discarded value.
pub fn uint16Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseUint(sval, 0, 16);
    if err != nil {
        return (goish::goany::Any::new(0u16), err);
    }
    return (goish::goany::Any::new(v as uint16), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 uint16.go:34-40 FlagSet.GetUint16
    pub fn GetUint16<S: Into<string>>(&self, name: S) -> (uint16, error) {
        let (val, err) = self.getFlagType(name.into(), string("uint16"), uint16Conv);
        if err != nil {
            return (0u16, err);
        }
        return (*val.As::<uint16>().unwrap_or(&0u16), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 uint16.go:44-46 FlagSet.Uint16Var
    pub fn Uint16Var(&mut self, p: *mut uint16, name: string, value: uint16, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUint16Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint16.go:49-51 FlagSet.Uint16VarP
    pub fn Uint16VarP(&mut self, p: *mut uint16, name: string, shorthand: string, value: uint16, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUint16Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint16.go:66-70 FlagSet.Uint16
    pub fn Uint16(&mut self, name: string, value: uint16, usage: string) -> *mut uint16 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0u16));
        self.Uint16VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint16.go:73-77 FlagSet.Uint16P
    pub fn Uint16P(&mut self, name: string, shorthand: string, value: uint16, usage: string) -> *mut uint16 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0u16));
        self.Uint16VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint16.go:55-57 Uint16Var
pub fn Uint16Var(p: *mut uint16, name: string, value: uint16, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUint16Value(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint16.go:60-62 Uint16VarP
pub fn Uint16VarP(p: *mut uint16, name: string, shorthand: string, value: uint16, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUint16Value(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint16.go:81-83 Uint16
pub fn Uint16(name: string, value: uint16, usage: string) -> *mut uint16 {
    return COMMAND_LINE.Lock().Uint16P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint16.go:86-88 Uint16P
pub fn Uint16P(name: string, shorthand: string, value: uint16, usage: string) -> *mut uint16 {
    return COMMAND_LINE.Lock().Uint16P(name, shorthand, value, usage);
}
