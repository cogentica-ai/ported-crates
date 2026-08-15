// go: file uint_slice.go decls: uintSliceValue, newUintSliceValue, uintSliceValue.Set, uintSliceValue.Type, uintSliceValue.String, uintSliceValue.fromString, uintSliceValue.toString, uintSliceValue.Append, uintSliceValue.Replace, uintSliceValue.GetSlice, uintSliceConv, FlagSet.GetUintSlice, FlagSet.UintSliceVar, FlagSet.UintSliceVarP, UintSliceVar, UintSliceVarP, FlagSet.UintSlice, FlagSet.UintSliceP, UintSlice, UintSliceP
//
// uint_slice.go — the slice family shares this shape; see int64_slice.rs for
// the commentary on the replace-then-append Set semantics.

use crate::*;
use goish::types::uint;

// go: github.com/spf13/pflag@v1.0.10 uint_slice.go:10-13 uintSliceValue
pub struct uintSliceValue {
    value: *mut slice<uint>,
    changed: bool,
}
unsafe impl Send for uintSliceValue {}
unsafe impl Sync for uintSliceValue {}

// go: github.com/spf13/pflag@v1.0.10 uint_slice.go:15-20 newUintSliceValue
pub fn newUintSliceValue(val: slice<uint>, p: *mut slice<uint>) -> uintSliceValue {
    let isv = uintSliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl uintSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:53-59 uintSliceValue.fromString
    fn fromString(&self, val: string) -> (uint, error) {
        let (v, err) = strconv::ParseUint(val, 10, 0);
        if err != nil {
            return (0u64, err);
        }
        return (v as uint, nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:61-63 uintSliceValue.toString
    fn toString(&self, val: uint) -> string {
        let v = val;
        return fmt::Sprintf!("%d", v as i64);
    }
}

impl SliceValue for uintSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:65-72 uintSliceValue.Append
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

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:74-85 uintSliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<uint> = make!([]uint, val.Len());
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

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:87-93 uintSliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = self.toString(v[i]);
        }
        return out;
    }
}

impl Value for uintSliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(uintSliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:45-51 uintSliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            let v = v[i];
            out[i] = fmt::Sprintf!("%d", v as i64);
        }
        return string("[") + strings::Join(out, string(",")) + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:22-39 uintSliceValue.Set
    /// Go: first Set REPLACES the default, later ones APPEND — that is
    /// what `changed` tracks, and dropping it would silently make
    /// `--x=1 --x=2` mean `[2]` instead of `[1,2]`.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: slice<uint> = make!([]uint, ss.Len());
        for i in 0..ss.Len() {
            let d = ss[i].clone();
            let (v, err) = strconv::ParseUint(d.clone(), 10, 0);
            if err != nil {
                return err;
            }
            out[i] = v as uint;
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

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:41-43 uintSliceValue.Type
    fn Type(&self) -> string {
        return string("uintSlice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint_slice.go:95-111 uintSliceConv
pub fn uintSliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        return (goish::goany::Any::new(make!([]uint, 0)), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<uint> = make!([]uint, ss.Len());
    for i in 0..ss.Len() {
        let d = ss[i].clone();
        let (v, err) = strconv::ParseUint(d.clone(), 10, 0);
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out[i] = v as uint;
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:114-120 FlagSet.GetUintSlice
    pub fn GetUintSlice<S: Into<string>>(&self, name: S) -> (slice<uint>, error) {
        let (val, err) = self.getFlagType(name.into(), string("uintSlice"), uintSliceConv);
        if err != nil {
            return (make!([]uint, 0), err);
        }
        return (val.As::<slice<uint>>().cloned().unwrap_or(make!([]uint, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:124-126 FlagSet.UintSliceVar
    pub fn UintSliceVar(&mut self, p: *mut slice<uint>, name: string, value: slice<uint>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUintSliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:129-131 FlagSet.UintSliceVarP
    pub fn UintSliceVarP(&mut self, p: *mut slice<uint>, name: string, shorthand: string, value: slice<uint>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newUintSliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:146-150 FlagSet.UintSlice
    pub fn UintSlice(&mut self, name: string, value: slice<uint>, usage: string) -> *mut slice<uint> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]uint, 0)));
        self.UintSliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 uint_slice.go:153-157 FlagSet.UintSliceP
    pub fn UintSliceP(&mut self, name: string, shorthand: string, value: slice<uint>, usage: string) -> *mut slice<uint> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]uint, 0)));
        self.UintSliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 uint_slice.go:135-137 UintSliceVar
pub fn UintSliceVar(p: *mut slice<uint>, name: string, value: slice<uint>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUintSliceValue(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint_slice.go:140-142 UintSliceVarP
pub fn UintSliceVarP(p: *mut slice<uint>, name: string, shorthand: string, value: slice<uint>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newUintSliceValue(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint_slice.go:161-163 UintSlice
pub fn UintSlice(name: string, value: slice<uint>, usage: string) -> *mut slice<uint> {
    return COMMAND_LINE.Lock().UintSliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 uint_slice.go:166-168 UintSliceP
pub fn UintSliceP(name: string, shorthand: string, value: slice<uint>, usage: string) -> *mut slice<uint> {
    return COMMAND_LINE.Lock().UintSliceP(name, shorthand, value, usage);
}
