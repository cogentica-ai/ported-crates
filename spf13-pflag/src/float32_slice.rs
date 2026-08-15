// go: file float32_slice.go decls: float32SliceValue, newFloat32SliceValue, float32SliceValue.Set, float32SliceValue.Type, float32SliceValue.String, float32SliceValue.fromString, float32SliceValue.toString, float32SliceValue.Append, float32SliceValue.Replace, float32SliceValue.GetSlice, float32SliceConv, FlagSet.GetFloat32Slice, FlagSet.Float32SliceVar, FlagSet.Float32SliceVarP, Float32SliceVar, Float32SliceVarP, FlagSet.Float32Slice, FlagSet.Float32SliceP, Float32Slice, Float32SliceP
//
// float32_slice.go — the slice family shares this shape; see int64_slice.rs for
// the commentary on the replace-then-append Set semantics.

use crate::*;
use goish::types::float32;

// go: github.com/spf13/pflag@v1.0.10 float32_slice.go:10-13 float32SliceValue
pub struct float32SliceValue {
    value: *mut slice<float32>,
    changed: bool,
}
unsafe impl Send for float32SliceValue {}
unsafe impl Sync for float32SliceValue {}

// go: github.com/spf13/pflag@v1.0.10 float32_slice.go:15-20 newFloat32SliceValue
pub fn newFloat32SliceValue(val: slice<float32>, p: *mut slice<float32>) -> float32SliceValue {
    let isv = float32SliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl float32SliceValue {
    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:56-62 float32SliceValue.fromString
    fn fromString(&self, val: string) -> (float32, error) {
        let (v, err) = strconv::ParseFloat(val, 32);
        if err != nil {
            return (0.0f32, err);
        }
        return (v as float32, nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:64-66 float32SliceValue.toString
    fn toString(&self, val: float32) -> string {
        let v = val;
        return fmt::Sprintf!("%f", v as f64);
    }
}

impl SliceValue for float32SliceValue {
    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:68-75 float32SliceValue.Append
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

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:77-88 float32SliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<float32> = make!([]float32, val.Len());
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

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:90-96 float32SliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = self.toString(v[i]);
        }
        return out;
    }
}

impl Value for float32SliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(float32SliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:48-54 float32SliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            let v = v[i];
            out[i] = fmt::Sprintf!("%f", v as f64);
        }
        return string("[") + strings::Join(out, string(",")) + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:22-42 float32SliceValue.Set
    /// Go: first Set REPLACES the default, later ones APPEND — that is
    /// what `changed` tracks, and dropping it would silently make
    /// `--x=1 --x=2` mean `[2]` instead of `[1,2]`.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: slice<float32> = make!([]float32, ss.Len());
        for i in 0..ss.Len() {
            let d = ss[i].clone();
            let (v, err) = strconv::ParseFloat(d.clone(), 32);
            if err != nil {
                return err;
            }
            out[i] = v as float32;
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

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:44-46 float32SliceValue.Type
    fn Type(&self) -> string {
        return string("float32Slice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 float32_slice.go:98-117 float32SliceConv
pub fn float32SliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        return (goish::goany::Any::new(make!([]float32, 0)), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<float32> = make!([]float32, ss.Len());
    for i in 0..ss.Len() {
        let d = ss[i].clone();
        let (v, err) = strconv::ParseFloat(d.clone(), 32);
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out[i] = v as float32;
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:120-126 FlagSet.GetFloat32Slice
    pub fn GetFloat32Slice<S: Into<string>>(&self, name: S) -> (slice<float32>, error) {
        let (val, err) = self.getFlagType(name.into(), string("float32Slice"), float32SliceConv);
        if err != nil {
            return (make!([]float32, 0), err);
        }
        return (val.As::<slice<float32>>().cloned().unwrap_or(make!([]float32, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:130-132 FlagSet.Float32SliceVar
    pub fn Float32SliceVar(&mut self, p: *mut slice<float32>, name: string, value: slice<float32>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newFloat32SliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:135-137 FlagSet.Float32SliceVarP
    pub fn Float32SliceVarP(&mut self, p: *mut slice<float32>, name: string, shorthand: string, value: slice<float32>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newFloat32SliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:152-156 FlagSet.Float32Slice
    pub fn Float32Slice(&mut self, name: string, value: slice<float32>, usage: string) -> *mut slice<float32> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]float32, 0)));
        self.Float32SliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 float32_slice.go:159-163 FlagSet.Float32SliceP
    pub fn Float32SliceP(&mut self, name: string, shorthand: string, value: slice<float32>, usage: string) -> *mut slice<float32> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]float32, 0)));
        self.Float32SliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 float32_slice.go:141-143 Float32SliceVar
pub fn Float32SliceVar(p: *mut slice<float32>, name: string, value: slice<float32>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newFloat32SliceValue(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 float32_slice.go:146-148 Float32SliceVarP
pub fn Float32SliceVarP(p: *mut slice<float32>, name: string, shorthand: string, value: slice<float32>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newFloat32SliceValue(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float32_slice.go:167-169 Float32Slice
pub fn Float32Slice(name: string, value: slice<float32>, usage: string) -> *mut slice<float32> {
    return COMMAND_LINE.Lock().Float32SliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 float32_slice.go:172-174 Float32SliceP
pub fn Float32SliceP(name: string, shorthand: string, value: slice<float32>, usage: string) -> *mut slice<float32> {
    return COMMAND_LINE.Lock().Float32SliceP(name, shorthand, value, usage);
}
