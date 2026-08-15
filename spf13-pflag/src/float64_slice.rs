// go: file float64_slice.go decls: float64SliceValue, newFloat64SliceValue, float64SliceValue.Set, float64SliceValue.Type, float64SliceValue.String, float64SliceValue.fromString, float64SliceValue.toString, float64SliceValue.Append, float64SliceValue.Replace, float64SliceValue.GetSlice, float64SliceConv, FlagSet.GetFloat64Slice, FlagSet.Float64SliceVar, FlagSet.Float64SliceVarP, Float64SliceVar, Float64SliceVarP, FlagSet.Float64Slice, FlagSet.Float64SliceP, Float64Slice, Float64SliceP
//
// float64_slice.go — the slice family shares this shape; see int64_slice.rs for
// the commentary on the replace-then-append Set semantics.

use crate::*;
use goish::types::float64;

// go: github.com/spf13/pflag@v1.0.10 float64_slice.go:10-13 float64SliceValue
pub struct float64SliceValue {
    value: *mut slice<float64>,
    changed: bool,
}
unsafe impl Send for float64SliceValue {}
unsafe impl Sync for float64SliceValue {}

// go: github.com/spf13/pflag@v1.0.10 float64_slice.go:15-20 newFloat64SliceValue
pub fn newFloat64SliceValue(val: slice<float64>, p: *mut slice<float64>) -> float64SliceValue {
    let isv = float64SliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl float64SliceValue {
    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:54-56 float64SliceValue.fromString
    fn fromString(&self, val: string) -> (float64, error) {
        let (v, err) = strconv::ParseFloat(val, 64);
        if err != nil {
            return (0.0f64, err);
        }
        return (v, nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:58-60 float64SliceValue.toString
    fn toString(&self, val: float64) -> string {
        let v = val;
        return fmt::Sprintf!("%f", v);
    }
}

impl SliceValue for float64SliceValue {
    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:62-69 float64SliceValue.Append
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

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:71-82 float64SliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<float64> = make!([]float64, val.Len());
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

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:84-90 float64SliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = self.toString(v[i]);
        }
        return out;
    }
}

impl Value for float64SliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(float64SliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:46-52 float64SliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            let v = v[i];
            out[i] = fmt::Sprintf!("%f", v);
        }
        return string("[") + strings::Join(out, string(",")) + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:22-40 float64SliceValue.Set
    /// Go: first Set REPLACES the default, later ones APPEND — that is
    /// what `changed` tracks, and dropping it would silently make
    /// `--x=1 --x=2` mean `[2]` instead of `[1,2]`.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: slice<float64> = make!([]float64, ss.Len());
        for i in 0..ss.Len() {
            let d = ss[i].clone();
            let (v, err) = strconv::ParseFloat(d.clone(), 64);
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

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:42-44 float64SliceValue.Type
    fn Type(&self) -> string {
        return string("float64Slice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 float64_slice.go:92-109 float64SliceConv
pub fn float64SliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        return (goish::goany::Any::new(make!([]float64, 0)), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<float64> = make!([]float64, ss.Len());
    for i in 0..ss.Len() {
        let d = ss[i].clone();
        let (v, err) = strconv::ParseFloat(d.clone(), 64);
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out[i] = v;
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:112-118 FlagSet.GetFloat64Slice
    pub fn GetFloat64Slice<S: Into<string>>(&self, name: S) -> (slice<float64>, error) {
        let (val, err) = self.getFlagType(name.into(), string("float64Slice"), float64SliceConv);
        if err != nil {
            return (make!([]float64, 0), err);
        }
        return (val.As::<slice<float64>>().cloned().unwrap_or(make!([]float64, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:122-124 FlagSet.Float64SliceVar
    pub fn Float64SliceVar(&mut self, p: *mut slice<float64>, name: string, value: slice<float64>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newFloat64SliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:127-129 FlagSet.Float64SliceVarP
    pub fn Float64SliceVarP(&mut self, p: *mut slice<float64>, name: string, shorthand: string, value: slice<float64>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newFloat64SliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:144-148 FlagSet.Float64Slice
    pub fn Float64Slice(&mut self, name: string, value: slice<float64>, usage: string) -> *mut slice<float64> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]float64, 0)));
        self.Float64SliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 float64_slice.go:151-155 FlagSet.Float64SliceP
    pub fn Float64SliceP(&mut self, name: string, shorthand: string, value: slice<float64>, usage: string) -> *mut slice<float64> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]float64, 0)));
        self.Float64SliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 float64_slice.go:133-135 Float64SliceVar
pub fn Float64SliceVar(p: *mut slice<float64>, name: string, value: slice<float64>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newFloat64SliceValue(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 float64_slice.go:138-140 Float64SliceVarP
pub fn Float64SliceVarP(p: *mut slice<float64>, name: string, shorthand: string, value: slice<float64>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newFloat64SliceValue(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float64_slice.go:159-161 Float64Slice
pub fn Float64Slice(name: string, value: slice<float64>, usage: string) -> *mut slice<float64> {
    return COMMAND_LINE.Lock().Float64SliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float64_slice.go:164-166 Float64SliceP
pub fn Float64SliceP(name: string, shorthand: string, value: slice<float64>, usage: string) -> *mut slice<float64> {
    return COMMAND_LINE.Lock().Float64SliceP(name, shorthand, value, usage);
}
