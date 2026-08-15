// go: file float32.go decls: float32Value, newFloat32Value, float32Value.Set, float32Value.Type, float32Value.String, float32Conv, FlagSet.GetFloat32, FlagSet.Float32Var, FlagSet.Float32VarP, Float32Var, Float32VarP, FlagSet.Float32, FlagSet.Float32P, Float32, Float32P
//
// float32.go — same shape as int8.go, which carries the commentary for the
// whole per-type family.

use crate::*;
// Explicit: `use crate::*` also pulls in the module named `float32`, and a
// glob-imported module shadows the glob-imported TYPE of the same name.
use goish::types::float32;

// go: github.com/spf13/pflag@v1.0.10 float32.go:6-6 float32Value
pub struct float32Value {
    ptr: *mut float32,
}
unsafe impl Send for float32Value {}
unsafe impl Sync for float32Value {}

// go: github.com/spf13/pflag@v1.0.10 float32.go:8-11 newFloat32Value
pub fn newFloat32Value(val: float32, p: *mut float32) -> float32Value {
    unsafe {
        *p = val;
    }
    return float32Value { ptr: p };
}

impl Value for float32Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(float32Value { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 float32.go:23-23 float32Value.String
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        return strconv::FormatFloat(v as f64, b'g', -1, 32);
    }

    // go: github.com/spf13/pflag@v1.0.10 float32.go:13-17 float32Value.Set
    /// Go assigns before returning err, so a parse failure still stores
    /// the parser's zero — kept, because the deviation would be silent.
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseFloat(s, 32);
        unsafe {
            *self.ptr = v as float32;
        }
        return err;
    }

    // go: github.com/spf13/pflag@v1.0.10 float32.go:19-21 float32Value.Type
    fn Type(&self) -> string {
        return string("float32");
    }
}

// go: github.com/spf13/pflag@v1.0.10 float32.go:25-31 float32Conv
/// Go's error arm returns an untyped `0`, which lands in the interface
/// as an `int`; this returns the zero of the flag's own type so the
/// caller's `As::<float32>()` cannot mis-read a discarded value.
pub fn float32Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseFloat(sval, 32);
    if err != nil {
        return (goish::goany::Any::new(0.0f32), err);
    }
    return (goish::goany::Any::new(v as float32), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 float32.go:34-40 FlagSet.GetFloat32
    pub fn GetFloat32<S: Into<string>>(&self, name: S) -> (float32, error) {
        let (val, err) = self.getFlagType(name.into(), string("float32"), float32Conv);
        if err != nil {
            return (0.0f32, err);
        }
        return (*val.As::<float32>().unwrap_or(&0.0f32), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 float32.go:44-46 FlagSet.Float32Var
    pub fn Float32Var(&mut self, p: *mut float32, name: string, value: float32, usage: string) {
        self.VarP(alloc::boxed::Box::new(newFloat32Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 float32.go:49-51 FlagSet.Float32VarP
    pub fn Float32VarP(&mut self, p: *mut float32, name: string, shorthand: string, value: float32, usage: string) {
        self.VarP(alloc::boxed::Box::new(newFloat32Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 float32.go:66-70 FlagSet.Float32
    pub fn Float32(&mut self, name: string, value: float32, usage: string) -> *mut float32 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0.0f32));
        self.Float32VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 float32.go:73-77 FlagSet.Float32P
    pub fn Float32P(&mut self, name: string, shorthand: string, value: float32, usage: string) -> *mut float32 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0.0f32));
        self.Float32VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 float32.go:55-57 Float32Var
pub fn Float32Var(p: *mut float32, name: string, value: float32, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newFloat32Value(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 float32.go:60-62 Float32VarP
pub fn Float32VarP(p: *mut float32, name: string, shorthand: string, value: float32, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newFloat32Value(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float32.go:81-83 Float32
pub fn Float32(name: string, value: float32, usage: string) -> *mut float32 {
    return COMMAND_LINE.Lock().Float32P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float32.go:86-88 Float32P
pub fn Float32P(name: string, shorthand: string, value: float32, usage: string) -> *mut float32 {
    return COMMAND_LINE.Lock().Float32P(name, shorthand, value, usage);
}
