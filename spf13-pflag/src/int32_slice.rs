// go: file int32_slice.go decls: int32SliceValue, newInt32SliceValue, int32SliceValue.Set, int32SliceValue.Type, int32SliceValue.String, int32SliceValue.fromString, int32SliceValue.toString, int32SliceValue.Append, int32SliceValue.Replace, int32SliceValue.GetSlice, int32SliceConv, FlagSet.GetInt32Slice, FlagSet.Int32SliceVar, FlagSet.Int32SliceVarP, Int32SliceVar, Int32SliceVarP, FlagSet.Int32Slice, FlagSet.Int32SliceP, Int32Slice, Int32SliceP
//
// int32_slice.go — the slice family shares this shape; see int64_slice.rs for
// the commentary on the replace-then-append Set semantics.

use crate::*;
use goish::types::int32;

// go: github.com/spf13/pflag@v1.0.10 int32_slice.go:10-13 int32SliceValue
pub struct int32SliceValue {
    value: *mut slice<int32>,
    changed: bool,
}
unsafe impl Send for int32SliceValue {}
unsafe impl Sync for int32SliceValue {}

// go: github.com/spf13/pflag@v1.0.10 int32_slice.go:15-20 newInt32SliceValue
pub fn newInt32SliceValue(val: slice<int32>, p: *mut slice<int32>) -> int32SliceValue {
    let isv = int32SliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl int32SliceValue {
    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:56-62 int32SliceValue.fromString
    fn fromString(&self, val: string) -> (int32, error) {
        let (v, err) = strconv::ParseInt(val, 0, 32);
        if err != nil {
            return (0i32, err);
        }
        return (v as int32, nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:64-66 int32SliceValue.toString
    fn toString(&self, val: int32) -> string {
        let v = val;
        return fmt::Sprintf!("%d", v as i64);
    }
}

impl SliceValue for int32SliceValue {
    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:68-75 int32SliceValue.Append
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

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:77-88 int32SliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<int32> = make!([]int32, val.Len());
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

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:90-96 int32SliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = self.toString(v[i]);
        }
        return out;
    }
}

impl Value for int32SliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(int32SliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:48-54 int32SliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            let v = v[i];
            out[i] = fmt::Sprintf!("%d", v as i64);
        }
        return string("[") + strings::Join(out, string(",")) + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:22-42 int32SliceValue.Set
    /// Go: first Set REPLACES the default, later ones APPEND — that is
    /// what `changed` tracks, and dropping it would silently make
    /// `--x=1 --x=2` mean `[2]` instead of `[1,2]`.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: slice<int32> = make!([]int32, ss.Len());
        for i in 0..ss.Len() {
            let d = ss[i].clone();
            let (v, err) = strconv::ParseInt(d.clone(), 0, 32);
            if err != nil {
                return err;
            }
            out[i] = v as int32;
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

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:44-46 int32SliceValue.Type
    fn Type(&self) -> string {
        return string("int32Slice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 int32_slice.go:98-117 int32SliceConv
pub fn int32SliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        return (goish::goany::Any::new(make!([]int32, 0)), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<int32> = make!([]int32, ss.Len());
    for i in 0..ss.Len() {
        let d = ss[i].clone();
        let (v, err) = strconv::ParseInt(d.clone(), 0, 32);
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out[i] = v as int32;
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:120-126 FlagSet.GetInt32Slice
    pub fn GetInt32Slice<S: Into<string>>(&self, name: S) -> (slice<int32>, error) {
        let (val, err) = self.getFlagType(name.into(), string("int32Slice"), int32SliceConv);
        if err != nil {
            return (make!([]int32, 0), err);
        }
        return (val.As::<slice<int32>>().cloned().unwrap_or(make!([]int32, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:130-132 FlagSet.Int32SliceVar
    pub fn Int32SliceVar(&mut self, p: *mut slice<int32>, name: string, value: slice<int32>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newInt32SliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:135-137 FlagSet.Int32SliceVarP
    pub fn Int32SliceVarP(&mut self, p: *mut slice<int32>, name: string, shorthand: string, value: slice<int32>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newInt32SliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:152-156 FlagSet.Int32Slice
    pub fn Int32Slice(&mut self, name: string, value: slice<int32>, usage: string) -> *mut slice<int32> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]int32, 0)));
        self.Int32SliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 int32_slice.go:159-163 FlagSet.Int32SliceP
    pub fn Int32SliceP(&mut self, name: string, shorthand: string, value: slice<int32>, usage: string) -> *mut slice<int32> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]int32, 0)));
        self.Int32SliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 int32_slice.go:141-143 Int32SliceVar
pub fn Int32SliceVar(p: *mut slice<int32>, name: string, value: slice<int32>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newInt32SliceValue(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 int32_slice.go:146-148 Int32SliceVarP
pub fn Int32SliceVarP(p: *mut slice<int32>, name: string, shorthand: string, value: slice<int32>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newInt32SliceValue(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int32_slice.go:167-169 Int32Slice
pub fn Int32Slice(name: string, value: slice<int32>, usage: string) -> *mut slice<int32> {
    return COMMAND_LINE.Lock().Int32SliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int32_slice.go:172-174 Int32SliceP
pub fn Int32SliceP(name: string, shorthand: string, value: slice<int32>, usage: string) -> *mut slice<int32> {
    return COMMAND_LINE.Lock().Int32SliceP(name, shorthand, value, usage);
}
