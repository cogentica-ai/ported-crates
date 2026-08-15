// go: file int_slice.go decls: intSliceValue, newIntSliceValue, intSliceValue.Set, intSliceValue.Type, intSliceValue.String, intSliceValue.Append, intSliceValue.Replace, intSliceValue.GetSlice, intSliceConv, FlagSet.GetIntSlice, FlagSet.IntSliceVar, FlagSet.IntSliceVarP, IntSliceVar, IntSliceVarP, FlagSet.IntSlice, FlagSet.IntSliceP, IntSlice, IntSliceP
//
// int_slice.go — 18 decls, two fewer than its siblings: this file predates
// the fromString/toString split, so Append/Replace/GetSlice each call
// strconv directly. Kept that way rather than refactored into the newer
// shape, so the port diffs against upstream line for line.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 int_slice.go:10-13 intSliceValue
pub struct intSliceValue {
    value: *mut slice<int>,
    changed: bool,
}
unsafe impl Send for intSliceValue {}
unsafe impl Sync for intSliceValue {}

// go: github.com/spf13/pflag@v1.0.10 int_slice.go:15-20 newIntSliceValue
pub fn newIntSliceValue(val: slice<int>, p: *mut slice<int>) -> intSliceValue {
    let isv = intSliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl SliceValue for intSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:54-61 intSliceValue.Append
    fn Append(&mut self, val: string) -> error {
        let (i, err) = strconv::Atoi(val);
        if err != nil {
            return err;
        }
        unsafe {
            *self.value = append!((*self.value).clone(), i);
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:63-74 intSliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<int> = make!([]int, val.Len());
        for i in 0..val.Len() {
            let (v, err) = strconv::Atoi(val[i].clone());
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

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:76-82 intSliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = strconv::Itoa(v[i]);
        }
        return out;
    }
}

impl Value for intSliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(intSliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:46-52 intSliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = fmt::Sprintf!("%d", v[i]);
        }
        return string("[") + strings::Join(out, string(",")) + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:22-40 intSliceValue.Set
    /// First Set replaces the default, later ones append — see
    /// int64_slice.rs and the TestSliceReplaceThenAppend tripwire.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: slice<int> = make!([]int, ss.Len());
        for i in 0..ss.Len() {
            let (v, err) = strconv::Atoi(ss[i].clone());
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

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:42-44 intSliceValue.Type
    fn Type(&self) -> string {
        return string("intSlice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 int_slice.go:84-101 intSliceConv
pub fn intSliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        return (goish::goany::Any::new(make!([]int, 0)), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<int> = make!([]int, ss.Len());
    for i in 0..ss.Len() {
        let (v, err) = strconv::Atoi(ss[i].clone());
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out[i] = v;
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:104-110 FlagSet.GetIntSlice
    pub fn GetIntSlice<S: Into<string>>(&self, name: S) -> (slice<int>, error) {
        let (val, err) = self.getFlagType(name.into(), string("intSlice"), intSliceConv);
        if err != nil {
            return (make!([]int, 0), err);
        }
        return (val.As::<slice<int>>().cloned().unwrap_or(make!([]int, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:114-116 FlagSet.IntSliceVar
    pub fn IntSliceVar(&mut self, p: *mut slice<int>, name: string, value: slice<int>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIntSliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:119-121 FlagSet.IntSliceVarP
    pub fn IntSliceVarP(&mut self, p: *mut slice<int>, name: string, shorthand: string, value: slice<int>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIntSliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:136-140 FlagSet.IntSlice
    pub fn IntSlice(&mut self, name: string, value: slice<int>, usage: string) -> *mut slice<int> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]int, 0)));
        self.IntSliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 int_slice.go:143-147 FlagSet.IntSliceP
    pub fn IntSliceP(&mut self, name: string, shorthand: string, value: slice<int>, usage: string) -> *mut slice<int> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]int, 0)));
        self.IntSliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 int_slice.go:125-127 IntSliceVar
pub fn IntSliceVar(p: *mut slice<int>, name: string, value: slice<int>, usage: string) {
    COMMAND_LINE.Lock().IntSliceVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int_slice.go:130-132 IntSliceVarP
pub fn IntSliceVarP(p: *mut slice<int>, name: string, shorthand: string, value: slice<int>, usage: string) {
    COMMAND_LINE.Lock().IntSliceVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int_slice.go:151-153 IntSlice
pub fn IntSlice(name: string, value: slice<int>, usage: string) -> *mut slice<int> {
    return COMMAND_LINE.Lock().IntSliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 int_slice.go:156-158 IntSliceP
pub fn IntSliceP(name: string, shorthand: string, value: slice<int>, usage: string) -> *mut slice<int> {
    return COMMAND_LINE.Lock().IntSliceP(name, shorthand, value, usage);
}
