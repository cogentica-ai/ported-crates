// go: file string_to_int.go decls: stringToIntValue, newStringToIntValue, stringToIntValue.Set, stringToIntValue.Type, stringToIntValue.String, stringToIntConv, FlagSet.GetStringToInt, FlagSet.StringToIntVar, FlagSet.StringToIntVarP, StringToIntVar, StringToIntVarP, FlagSet.StringToInt, FlagSet.StringToIntP, StringToInt, StringToIntP
//
// string_to_int.go — a map-valued flag, `--m=a=1,b=2`.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 string_to_int.go:11-14 stringToIntValue
pub struct stringToIntValue {
    value: *mut map<string, int>,
    changed: bool,
}
unsafe impl Send for stringToIntValue {}
unsafe impl Sync for stringToIntValue {}

// go: github.com/spf13/pflag@v1.0.10 string_to_int.go:16-21 newStringToIntValue
pub fn newStringToIntValue(val: map<string, int>, p: *mut map<string, int>) -> stringToIntValue {
    let ssv = stringToIntValue { value: p, changed: false };
    unsafe {
        *ssv.value = val;
    }
    return ssv;
}

impl Value for stringToIntValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(stringToIntValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int.go:53-66 stringToIntValue.String
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
            let _ = buf.WriteString(strconv::Itoa(v));
            i += 1;
        }
        return string("[") + buf.String() + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int.go:24-47 stringToIntValue.Set
    /// Go: "Format: a=1,b=2". A later --flag MERGES into the map rather
    /// than replacing it, unlike the slice family's append.
    fn Set_str(&mut self, val: string) -> error {
        let ss = strings::Split(val, string(","));
        let mut out: map<string, int> = map::new();
        for i in 0..ss.Len() {
            let pair = ss[i].clone();
            let kv = strings::SplitN(pair.clone(), string("="), 2);
            if kv.Len() != 2 {
                return fmt::Errorf!("%s must be formatted as key=value", pair);
            }
            let (v, err) = strconv::Atoi(kv[1].clone());
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

    // go: github.com/spf13/pflag@v1.0.10 string_to_int.go:49-51 stringToIntValue.Type
    fn Type(&self) -> string {
        return string("stringToInt");
    }
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int.go:68-88 stringToIntConv
pub fn stringToIntConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "An empty string would cause an empty map"
    if val.Len() == 0 {
        let empty: map<string, int> = map::new();
        return (goish::goany::Any::new(empty), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: map<string, int> = map::new();
    for i in 0..ss.Len() {
        let pair = ss[i].clone();
        let kv = strings::SplitN(pair.clone(), string("="), 2);
        if kv.Len() != 2 {
            return (goish::goany::Any::from(nil),
                    fmt::Errorf!("%s must be formatted as key=value", pair));
        }
        let (v, err) = strconv::Atoi(kv[1].clone());
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        out.Set(kv[0].clone(), v);
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 string_to_int.go:91-97 FlagSet.GetStringToInt
    pub fn GetStringToInt<S: Into<string>>(&self, name: S) -> (map<string, int>, error) {
        let (val, err) = self.getFlagType(name.into(), string("stringToInt"), stringToIntConv);
        if err != nil {
            return (map::new(), err);
        }
        return (val.As::<map<string, int>>().cloned().unwrap_or(map::new()), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int.go:102-104 FlagSet.StringToIntVar
    pub fn StringToIntVar(&mut self, p: *mut map<string, int>, name: string, value: map<string, int>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newStringToIntValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int.go:107-109 FlagSet.StringToIntVarP
    pub fn StringToIntVarP(&mut self, p: *mut map<string, int>, name: string, shorthand: string, value: map<string, int>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newStringToIntValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int.go:126-130 FlagSet.StringToInt
    pub fn StringToInt(&mut self, name: string, value: map<string, int>, usage: string) -> *mut map<string, int> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(map::new()));
        self.StringToIntVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_int.go:133-137 FlagSet.StringToIntP
    pub fn StringToIntP(&mut self, name: string, shorthand: string, value: map<string, int>, usage: string) -> *mut map<string, int> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(map::new()));
        self.StringToIntVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int.go:114-116 StringToIntVar
pub fn StringToIntVar(p: *mut map<string, int>, name: string, value: map<string, int>, usage: string) {
    COMMAND_LINE.Lock().StringToIntVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int.go:119-121 StringToIntVarP
pub fn StringToIntVarP(p: *mut map<string, int>, name: string, shorthand: string, value: map<string, int>, usage: string) {
    COMMAND_LINE.Lock().StringToIntVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int.go:142-144 StringToInt
pub fn StringToInt(name: string, value: map<string, int>, usage: string) -> *mut map<string, int> {
    return COMMAND_LINE.Lock().StringToIntP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_int.go:147-149 StringToIntP
pub fn StringToIntP(name: string, shorthand: string, value: map<string, int>, usage: string) -> *mut map<string, int> {
    return COMMAND_LINE.Lock().StringToIntP(name, shorthand, value, usage);
}
