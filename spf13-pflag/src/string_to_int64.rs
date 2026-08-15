// go: file string_to_int64.go decls: stringToInt64Value, newStringToInt64Value, stringToInt64Value.Set, stringToInt64Value.Type, stringToInt64Value.String, stringToInt64Conv, FlagSet.GetStringToInt64, FlagSet.StringToInt64Var, FlagSet.StringToInt64VarP, StringToInt64Var, StringToInt64VarP, FlagSet.StringToInt64, FlagSet.StringToInt64P, StringToInt64, StringToInt64P
//
// string_to_int64.go — a map-valued flag, `--m=a=1,b=2`.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:11-14 stringToInt64Value
pub struct stringToInt64Value {
    value: *mut map<string, i64>,
    changed: bool,
}
unsafe impl Send for stringToInt64Value {}
unsafe impl Sync for stringToInt64Value {}

// go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:16-21 newStringToInt64Value
pub fn newStringToInt64Value(val: map<string, i64>, p: *mut map<string, i64>) -> stringToInt64Value {
    let ssv = stringToInt64Value { value: p, changed: false };
    unsafe {
        *ssv.value = val;
    }
    return ssv;
}

impl Value for stringToInt64Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(stringToInt64Value { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:53-66 stringToInt64Value.String
    /// Go builds this by ranging the map, whose order is randomised;
    /// goish's map iterates in its own order. Neither is a stable
    /// contract, so no test pins the ordering.
    fn String(&self) -> string {
        let m = unsafe { (*self.value).clone() };
        let mut buf = strings::Builder::new();
        let mut i = 0;
        for (k, v) in m.__iter() {
            if i > 0 {
                let _ = buf.WriteString(string(","));
            }
            let _ = buf.WriteString(k.clone());
            let _ = buf.WriteString(string("="));
            let v = v.clone();
            let _ = buf.WriteString(strconv::FormatInt(int(v), 10));
            i += 1;
        }
        return string("[") + buf.String() + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:24-47 stringToInt64Value.Set
    /// Go: "Format: a=1,b=2". A later --flag MERGES into the map rather
    /// than replacing it, unlike the slice family's append.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: map<string, i64> = map::new();
        for i in 0..ss.Len() {
            let pair = ss[i].clone();
            let kv = strings::SplitN(pair.clone(), string("="), 2);
            if kv.Len() != 2 {
                return fmt::Errorf!("%s must be formatted as key=value", pair);
            }
            let (v, err) = strconv::ParseInt(kv[1].clone(), 0, 64);
            if err != nil {
                return err;
            }
            out.Set(kv[0].clone(), v);
        }
        unsafe {
            if !self.changed {
                *self.value = out;
            } else {
                for (k, v) in out.__iter() {
                    (*self.value).Set(k.clone(), v.clone());
                }
            }
        }
        self.changed = true;
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:49-51 stringToInt64Value.Type
    fn Type(&self) -> string {
        return string("stringToInt64");
    }
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:68-88 stringToInt64Conv
pub fn stringToInt64Conv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "An empty string would cause an empty map"
    if val.Len() == 0 {
        let empty: map<string, i64> = map::new();
        return (goish::goany::Any::new(empty), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: map<string, i64> = map::new();
    for i in 0..ss.Len() {
        let pair = ss[i].clone();
        let kv = strings::SplitN(pair.clone(), string("="), 2);
        if kv.Len() != 2 {
            return (goish::goany::Any::from(nil),
                    fmt::Errorf!("%s must be formatted as key=value", pair));
        }
        let (v, err) = strconv::ParseInt(kv[1].clone(), 0, 64);
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out.Set(kv[0].clone(), v);
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:91-97 FlagSet.GetStringToInt64
    pub fn GetStringToInt64<S: Into<string>>(&self, name: S) -> (map<string, i64>, error) {
        let (val, err) = self.getFlagType(name.into(), string("stringToInt64"), stringToInt64Conv);
        if err != nil {
            return (map::new(), err);
        }
        return (val.As::<map<string, i64>>().cloned().unwrap_or(map::new()), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:102-104 FlagSet.StringToInt64Var
    pub fn StringToInt64Var(&mut self, p: *mut map<string, i64>, name: string, value: map<string, i64>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newStringToInt64Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:107-109 FlagSet.StringToInt64VarP
    pub fn StringToInt64VarP(&mut self, p: *mut map<string, i64>, name: string, shorthand: string, value: map<string, i64>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newStringToInt64Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:126-130 FlagSet.StringToInt64
    pub fn StringToInt64(&mut self, name: string, value: map<string, i64>, usage: string) -> *mut map<string, i64> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(map::new()));
        self.StringToInt64VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:133-137 FlagSet.StringToInt64P
    pub fn StringToInt64P(&mut self, name: string, shorthand: string, value: map<string, i64>, usage: string) -> *mut map<string, i64> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(map::new()));
        self.StringToInt64VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:114-116 StringToInt64Var
pub fn StringToInt64Var(p: *mut map<string, i64>, name: string, value: map<string, i64>, usage: string) {
    COMMAND_LINE.Lock().StringToInt64VarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:119-121 StringToInt64VarP
pub fn StringToInt64VarP(p: *mut map<string, i64>, name: string, shorthand: string, value: map<string, i64>, usage: string) {
    COMMAND_LINE.Lock().StringToInt64VarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:142-144 StringToInt64
pub fn StringToInt64(name: string, value: map<string, i64>, usage: string) -> *mut map<string, i64> {
    return COMMAND_LINE.Lock().StringToInt64P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int64.go:147-149 StringToInt64P
pub fn StringToInt64P(name: string, shorthand: string, value: map<string, i64>, usage: string) -> *mut map<string, i64> {
    return COMMAND_LINE.Lock().StringToInt64P(name, shorthand, value, usage);
}
