// go: file int8.go decls: int8Value, newInt8Value, int8Value.Set, int8Value.Type, int8Value.String, int8Conv, FlagSet.GetInt8, FlagSet.Int8Var, FlagSet.Int8VarP, Int8Var, Int8VarP, FlagSet.Int8, FlagSet.Int8P, Int8, Int8P
//
// int8.go — one of pflag's ~40 per-type flag files. Every scalar file
// has this exact shape, so this one carries the commentary and the
// others cite their own lines without repeating it.

use crate::*;
// Explicit, and it must stay explicit: `use crate::*` also pulls in the
// module named `int8`, and a glob-imported module shadows the
// glob-imported TYPE of the same name. Every per-type file here has the
// same collision.
use goish::types::int8;

// go: github.com/spf13/pflag@v1.0.10 int8.go:6-6 int8Value
/// Go: `type int8Value int8` — a defined type over the flag's target,
/// converted to via pointer cast (`(*int8Value)(p)`). Rust has no
/// equivalent reinterpret of `*mut i8` as a distinct type carrying an
/// impl, so the port holds the pointer, matching every other Value in
/// this crate.
pub struct int8Value {
    ptr: *mut int8,
}
unsafe impl Send for int8Value {}
unsafe impl Sync for int8Value {}

// go: github.com/spf13/pflag@v1.0.10 int8.go:8-11 newInt8Value
pub fn newInt8Value(val: int8, p: *mut int8) -> int8Value {
    unsafe {
        *p = val;
    }
    return int8Value { ptr: p };
}

impl Value for int8Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(int8Value { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 int8.go:23-23 int8Value.String
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        return strconv::FormatInt(int(v as i64), 10);
    }

    // go: github.com/spf13/pflag@v1.0.10 int8.go:13-17 int8Value.Set
    /// Go assigns before returning err, so a parse failure still stores
    /// ParseInt's zero — the deviation would be silent, so it is kept.
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseInt(s, 0, 8);
        unsafe {
            *self.ptr = v as int8;
        }
        return err;
    }

    // go: github.com/spf13/pflag@v1.0.10 int8.go:19-21 int8Value.Type
    fn Type(&self) -> string {
        return string("int8");
    }
}

// go: github.com/spf13/pflag@v1.0.10 int8.go:25-31 int8Conv
pub fn int8Conv(sval: string) -> (goish::goany::Any, error) {
    let (v, err) = strconv::ParseInt(sval, 0, 8);
    if err != nil {
        return (goish::goany::Any::new(0i8), err);
    }
    return (goish::goany::Any::new(v as int8), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 int8.go:34-40 FlagSet.GetInt8
    pub fn GetInt8<S: Into<string>>(&self, name: S) -> (int8, error) {
        let (val, err) = self.getFlagType(name.into(), string("int8"), int8Conv);
        if err != nil {
            return (0, err);
        }
        return (*val.As::<int8>().unwrap_or(&0), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 int8.go:44-46 FlagSet.Int8Var
    pub fn Int8Var(&mut self, p: *mut int8, name: string, value: int8, usage: string) {
        self.VarP(alloc::boxed::Box::new(newInt8Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int8.go:49-51 FlagSet.Int8VarP
    pub fn Int8VarP(&mut self, p: *mut int8, name: string, shorthand: string, value: int8, usage: string) {
        self.VarP(alloc::boxed::Box::new(newInt8Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int8.go:66-70 FlagSet.Int8
    pub fn Int8(&mut self, name: string, value: int8, usage: string) -> *mut int8 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0 as int8));
        self.Int8VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 int8.go:73-77 FlagSet.Int8P
    pub fn Int8P(&mut self, name: string, shorthand: string, value: int8, usage: string) -> *mut int8 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0 as int8));
        self.Int8VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 int8.go:55-57 Int8Var
pub fn Int8Var(p: *mut int8, name: string, value: int8, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newInt8Value(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 int8.go:60-62 Int8VarP
pub fn Int8VarP(p: *mut int8, name: string, shorthand: string, value: int8, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newInt8Value(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int8.go:81-83 Int8
pub fn Int8(name: string, value: int8, usage: string) -> *mut int8 {
    return COMMAND_LINE.Lock().Int8P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int8.go:86-88 Int8P
pub fn Int8P(name: string, shorthand: string, value: int8, usage: string) -> *mut int8 {
    return COMMAND_LINE.Lock().Int8P(name, shorthand, value, usage);
}
