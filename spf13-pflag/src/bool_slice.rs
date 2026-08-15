// go: file bool_slice.go decls: boolSliceValue, newBoolSliceValue, boolSliceValue.Set, boolSliceValue.Type, boolSliceValue.String, boolSliceValue.fromString, boolSliceValue.toString, boolSliceValue.Append, boolSliceValue.Replace, boolSliceValue.GetSlice, boolSliceConv, FlagSet.GetBoolSlice, FlagSet.BoolSliceVar, FlagSet.BoolSliceVarP, BoolSliceVar, BoolSliceVarP, FlagSet.BoolSlice, FlagSet.BoolSliceP, BoolSlice, BoolSliceP
//
// bool_slice.go — the one slice file whose Set does not just Split on
// commas: it strips quote characters through a Replacer and reads the
// rest with the CSV parser, so `--b='true, false'` works.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 bool_slice.go:10-13 boolSliceValue
pub struct boolSliceValue {
    value: *mut slice<bool>,
    changed: bool,
}
unsafe impl Send for boolSliceValue {}
unsafe impl Sync for boolSliceValue {}

// go: github.com/spf13/pflag@v1.0.10 bool_slice.go:15-20 newBoolSliceValue
pub fn newBoolSliceValue(val: slice<bool>, p: *mut slice<bool>) -> boolSliceValue {
    let bsv = boolSliceValue { value: p, changed: false };
    unsafe {
        *bsv.value = val;
    }
    return bsv;
}

impl boolSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:74-76 boolSliceValue.fromString
    fn fromString(&self, val: string) -> (bool, error) {
        return strconv::ParseBool(val);
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:78-80 boolSliceValue.toString
    fn toString(&self, val: bool) -> string {
        return strconv::FormatBool(val);
    }
}

impl SliceValue for boolSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:82-89 boolSliceValue.Append
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

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:91-102 boolSliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<bool> = make!([]bool, val.Len());
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

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:104-110 boolSliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = self.toString(v[i]);
        }
        return out;
    }
}

impl Value for boolSliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(boolSliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:62-72 boolSliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut bool_str_slice: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            bool_str_slice[i] = strconv::FormatBool(v[i]);
        }
        let out = write_as_csv(bool_str_slice);
        return string("[") + out + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:24-54 boolSliceValue.Set
    /// Go strips `"`, `'` and backtick before the CSV read, so a shell
    /// that leaves quotes on the value still parses. Splitting on commas
    /// instead — as every other slice file does — would take `"true"`
    /// literally and fail ParseBool.
    fn Set_str(&mut self, val: string) -> error {
        // Go: strings.NewReplacer(`"`, "", `'`, "", "`", "")
        let rm_quote = strings::NewReplacer(slice!([]string {
            string("\""), string(""),
            string("'"), string(""),
            string("`"), string(""),
        }));
        let (bool_str_slice, err) = read_as_csv(rm_quote.Replace(val));
        if err != nil && !errors::Is(err.clone(), io::EOF) {
            return err;
        }
        let mut out: slice<bool> = make!([]bool, 0);
        for i in 0..bool_str_slice.Len() {
            let (b, err) = strconv::ParseBool(strings::TrimSpace(bool_str_slice[i].clone()));
            if err != nil {
                return err;
            }
            out = append!(out, b);
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

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:57-59 boolSliceValue.Type
    fn Type(&self) -> string {
        return string("boolSlice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 bool_slice.go:112-128 boolSliceConv
pub fn boolSliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        return (goish::goany::Any::new(make!([]bool, 0)), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<bool> = make!([]bool, ss.Len());
    for i in 0..ss.Len() {
        let (v, err) = strconv::ParseBool(ss[i].clone());
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out[i] = v;
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:131-137 FlagSet.GetBoolSlice
    pub fn GetBoolSlice<S: Into<string>>(&self, name: S) -> (slice<bool>, error) {
        let (val, err) = self.getFlagType(name.into(), string("boolSlice"), boolSliceConv);
        if err != nil {
            return (make!([]bool, 0), err);
        }
        return (val.As::<slice<bool>>().cloned().unwrap_or(make!([]bool, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:141-143 FlagSet.BoolSliceVar
    pub fn BoolSliceVar(&mut self, p: *mut slice<bool>, name: string, value: slice<bool>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newBoolSliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:146-148 FlagSet.BoolSliceVarP
    pub fn BoolSliceVarP(&mut self, p: *mut slice<bool>, name: string, shorthand: string, value: slice<bool>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newBoolSliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:163-167 FlagSet.BoolSlice
    pub fn BoolSlice(&mut self, name: string, value: slice<bool>, usage: string) -> *mut slice<bool> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]bool, 0)));
        self.BoolSliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_slice.go:170-174 FlagSet.BoolSliceP
    pub fn BoolSliceP(&mut self, name: string, shorthand: string, value: slice<bool>, usage: string) -> *mut slice<bool> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]bool, 0)));
        self.BoolSliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 bool_slice.go:152-154 BoolSliceVar
pub fn BoolSliceVar(p: *mut slice<bool>, name: string, value: slice<bool>, usage: string) {
    COMMAND_LINE.Lock().BoolSliceVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bool_slice.go:157-159 BoolSliceVarP
pub fn BoolSliceVarP(p: *mut slice<bool>, name: string, shorthand: string, value: slice<bool>, usage: string) {
    COMMAND_LINE.Lock().BoolSliceVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bool_slice.go:178-180 BoolSlice
pub fn BoolSlice(name: string, value: slice<bool>, usage: string) -> *mut slice<bool> {
    return COMMAND_LINE.Lock().BoolSliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bool_slice.go:183-185 BoolSliceP
pub fn BoolSliceP(name: string, shorthand: string, value: slice<bool>, usage: string) -> *mut slice<bool> {
    return COMMAND_LINE.Lock().BoolSliceP(name, shorthand, value, usage);
}
