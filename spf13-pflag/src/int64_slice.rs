// go: file int64_slice.go decls: int64SliceValue, newInt64SliceValue, int64SliceValue.Set, int64SliceValue.Type, int64SliceValue.String, int64SliceValue.fromString, int64SliceValue.toString, int64SliceValue.Append, int64SliceValue.Replace, int64SliceValue.GetSlice, int64SliceConv, FlagSet.GetInt64Slice, FlagSet.Int64SliceVar, FlagSet.Int64SliceVarP, Int64SliceVar, Int64SliceVarP, FlagSet.Int64Slice, FlagSet.Int64SliceP, Int64Slice, Int64SliceP
//
// int64_slice.go — the slice family shares this shape; see int64_slice.rs for
// the commentary on the replace-then-append Set semantics.

use crate::*;
use goish::types::int64;

// go: github.com/spf13/pflag@v1.0.10 int64_slice.go:10-13 int64SliceValue
pub struct int64SliceValue {
    value: *mut slice<int64>,
    changed: bool,
}
unsafe impl Send for int64SliceValue {}
unsafe impl Sync for int64SliceValue {}

// go: github.com/spf13/pflag@v1.0.10 int64_slice.go:15-20 newInt64SliceValue
pub fn newInt64SliceValue(val: slice<int64>, p: *mut slice<int64>) -> int64SliceValue {
    let isv = int64SliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl int64SliceValue {
    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:54-56 int64SliceValue.fromString
    fn fromString(&self, val: string) -> (int64, error) {
        let (v, err) = strconv::ParseInt(val, 0, 64);
        if err != nil {
            return (0i64, err);
        }
        return (v, nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:58-60 int64SliceValue.toString
    fn toString(&self, val: int64) -> string {
        let v = val;
        return fmt::Sprintf!("%d", v);
    }
}

impl SliceValue for int64SliceValue {
    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:62-69 int64SliceValue.Append
    fn Append(&mut self, val: string) -> error {
        let (i, err) = self.fromString(val);
        if err != nil {
            return err;
        }
        unsafe {
            *self.value = append!((*self.value).clone(), i);
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:71-82 int64SliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<int64> = make!([]int64, val.Len());
        for i in 0..val.Len() {
            let (v, err) = self.fromString(val[i].clone());
            if err != nil {
                return err;
            }
            out[i] = v;
        }
        unsafe {
            *self.value = out;
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:84-90 int64SliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = self.toString(v[i]);
        }
        return out;
    }
}

impl Value for int64SliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(int64SliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:46-52 int64SliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            let v = v[i];
            out[i] = fmt::Sprintf!("%d", v);
        }
        return string("[") + strings::Join(out, string(",")) + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:22-40 int64SliceValue.Set
    /// Go: first Set REPLACES the default, later ones APPEND — that is
    /// what `changed` tracks, and dropping it would silently make
    /// `--x=1 --x=2` mean `[2]` instead of `[1,2]`.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: slice<int64> = make!([]int64, ss.Len());
        for i in 0..ss.Len() {
            let d = ss[i].clone();
            let (v, err) = strconv::ParseInt(d.clone(), 0, 64);
            if err != nil {
                return err;
            }
            out[i] = v;
        }
        unsafe {
            if !self.changed {
                *self.value = out;
            } else {
                *self.value = append!((*self.value).clone(), out...);
            }
        }
        self.changed = true;
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:42-44 int64SliceValue.Type
    fn Type(&self) -> string {
        return string("int64Slice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 int64_slice.go:92-109 int64SliceConv
pub fn int64SliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        return (goish::goany::Any::new(make!([]int64, 0)), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<int64> = make!([]int64, ss.Len());
    for i in 0..ss.Len() {
        let d = ss[i].clone();
        let (v, err) = strconv::ParseInt(d.clone(), 0, 64);
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out[i] = v;
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:112-118 FlagSet.GetInt64Slice
    pub fn GetInt64Slice<S: Into<string>>(&self, name: S) -> (slice<int64>, error) {
        let (val, err) = self.getFlagType(name.into(), string("int64Slice"), int64SliceConv);
        if err != nil {
            return (make!([]int64, 0), err);
        }
        return (val.As::<slice<int64>>().cloned().unwrap_or(make!([]int64, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:122-124 FlagSet.Int64SliceVar
    pub fn Int64SliceVar(&mut self, p: *mut slice<int64>, name: string, value: slice<int64>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newInt64SliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:127-129 FlagSet.Int64SliceVarP
    pub fn Int64SliceVarP(&mut self, p: *mut slice<int64>, name: string, shorthand: string, value: slice<int64>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newInt64SliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:144-148 FlagSet.Int64Slice
    pub fn Int64Slice(&mut self, name: string, value: slice<int64>, usage: string) -> *mut slice<int64> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]int64, 0)));
        self.Int64SliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 int64_slice.go:151-155 FlagSet.Int64SliceP
    pub fn Int64SliceP(&mut self, name: string, shorthand: string, value: slice<int64>, usage: string) -> *mut slice<int64> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]int64, 0)));
        self.Int64SliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 int64_slice.go:133-135 Int64SliceVar
pub fn Int64SliceVar(p: *mut slice<int64>, name: string, value: slice<int64>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newInt64SliceValue(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 int64_slice.go:138-140 Int64SliceVarP
pub fn Int64SliceVarP(p: *mut slice<int64>, name: string, shorthand: string, value: slice<int64>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newInt64SliceValue(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int64_slice.go:159-161 Int64Slice
pub fn Int64Slice(name: string, value: slice<int64>, usage: string) -> *mut slice<int64> {
    return COMMAND_LINE.Lock().Int64SliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int64_slice.go:164-166 Int64SliceP
pub fn Int64SliceP(name: string, shorthand: string, value: slice<int64>, usage: string) -> *mut slice<int64> {
    return COMMAND_LINE.Lock().Int64SliceP(name, shorthand, value, usage);
}
