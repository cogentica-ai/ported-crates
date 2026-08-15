// go: file duration_slice.go decls: durationSliceValue, newDurationSliceValue, durationSliceValue.Set, durationSliceValue.Type, durationSliceValue.String, durationSliceValue.fromString, durationSliceValue.toString, durationSliceValue.Append, durationSliceValue.Replace, durationSliceValue.GetSlice, durationSliceConv, FlagSet.GetDurationSlice, FlagSet.DurationSliceVar, FlagSet.DurationSliceVarP, DurationSliceVar, DurationSliceVarP, FlagSet.DurationSlice, FlagSet.DurationSliceP, DurationSlice, DurationSliceP
//
// duration_slice.go — the slice family shares this shape; see int64_slice.rs for
// the commentary on the replace-then-append Set semantics.

use crate::*;
use goish::time::Duration;

// go: github.com/spf13/pflag@v1.0.10 duration_slice.go:10-13 durationSliceValue
pub struct durationSliceValue {
    value: *mut slice<Duration>,
    changed: bool,
}
unsafe impl Send for durationSliceValue {}
unsafe impl Sync for durationSliceValue {}

// go: github.com/spf13/pflag@v1.0.10 duration_slice.go:15-20 newDurationSliceValue
pub fn newDurationSliceValue(val: slice<Duration>, p: *mut slice<Duration>) -> durationSliceValue {
    let isv = durationSliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl durationSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:54-56 durationSliceValue.fromString
    fn fromString(&self, val: string) -> (Duration, error) {
        let (v, err) = time::ParseDuration(val);
        if err != nil {
            return (time::Duration(0), err);
        }
        return (v, nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:58-60 durationSliceValue.toString
    fn toString(&self, val: Duration) -> string {
        let v = val;
        return fmt::Sprintf!("%s", v);
    }
}

impl SliceValue for durationSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:62-69 durationSliceValue.Append
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

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:71-82 durationSliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<Duration> = make!([]Duration, val.Len());
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

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:84-90 durationSliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = self.toString(v[i]);
        }
        return out;
    }
}

impl Value for durationSliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(durationSliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:46-52 durationSliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            let v = v[i];
            out[i] = fmt::Sprintf!("%s", v);
        }
        return string("[") + strings::Join(out, string(",")) + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:22-40 durationSliceValue.Set
    /// Go: first Set REPLACES the default, later ones APPEND — that is
    /// what `changed` tracks, and dropping it would silently make
    /// `--x=1 --x=2` mean `[2]` instead of `[1,2]`.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: slice<Duration> = make!([]Duration, ss.Len());
        for i in 0..ss.Len() {
            let d = ss[i].clone();
            let (v, err) = time::ParseDuration(d.clone());
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

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:42-44 durationSliceValue.Type
    fn Type(&self) -> string {
        return string("durationSlice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 duration_slice.go:92-109 durationSliceConv
pub fn durationSliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        return (goish::goany::Any::new(make!([]Duration, 0)), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<Duration> = make!([]Duration, ss.Len());
    for i in 0..ss.Len() {
        let d = ss[i].clone();
        let (v, err) = time::ParseDuration(d.clone());
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out[i] = v;
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:112-118 FlagSet.GetDurationSlice
    pub fn GetDurationSlice<S: Into<string>>(&self, name: S) -> (slice<Duration>, error) {
        let (val, err) = self.getFlagType(name.into(), string("durationSlice"), durationSliceConv);
        if err != nil {
            return (make!([]Duration, 0), err);
        }
        return (val.As::<slice<Duration>>().cloned().unwrap_or(make!([]Duration, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:122-124 FlagSet.DurationSliceVar
    pub fn DurationSliceVar(&mut self, p: *mut slice<Duration>, name: string, value: slice<Duration>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newDurationSliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:127-129 FlagSet.DurationSliceVarP
    pub fn DurationSliceVarP(&mut self, p: *mut slice<Duration>, name: string, shorthand: string, value: slice<Duration>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newDurationSliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:144-148 FlagSet.DurationSlice
    pub fn DurationSlice(&mut self, name: string, value: slice<Duration>, usage: string) -> *mut slice<Duration> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]Duration, 0)));
        self.DurationSliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 duration_slice.go:151-155 FlagSet.DurationSliceP
    pub fn DurationSliceP(&mut self, name: string, shorthand: string, value: slice<Duration>, usage: string) -> *mut slice<Duration> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]Duration, 0)));
        self.DurationSliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 duration_slice.go:133-135 DurationSliceVar
pub fn DurationSliceVar(p: *mut slice<Duration>, name: string, value: slice<Duration>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newDurationSliceValue(value, p)), name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 duration_slice.go:138-140 DurationSliceVarP
pub fn DurationSliceVarP(p: *mut slice<Duration>, name: string, shorthand: string, value: slice<Duration>, usage: string) {
    COMMAND_LINE.Lock().VarP(alloc::boxed::Box::new(newDurationSliceValue(value, p)), name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 duration_slice.go:159-161 DurationSlice
pub fn DurationSlice(name: string, value: slice<Duration>, usage: string) -> *mut slice<Duration> {
    return COMMAND_LINE.Lock().DurationSliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 duration_slice.go:164-166 DurationSliceP
pub fn DurationSliceP(name: string, shorthand: string, value: slice<Duration>, usage: string) -> *mut slice<Duration> {
    return COMMAND_LINE.Lock().DurationSliceP(name, shorthand, value, usage);
}
