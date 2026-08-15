// go: file string_to_string.go decls: stringToStringValue, newStringToStringValue, stringToStringValue.Set, stringToStringValue.Type, stringToStringValue.String, stringToStringConv, FlagSet.GetStringToString, FlagSet.StringToStringVar, FlagSet.StringToStringVarP, StringToStringVar, StringToStringVarP, FlagSet.StringToString, FlagSet.StringToStringP, StringToString, StringToStringP
//
// string_to_string.go — the map-valued flag whose VALUES are strings, so
// it cannot just split on commas: a value may legitimately contain one.
// Set counts '=' to decide, and String sorts its keys — the only file in
// this family with a deterministic rendering.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 string_to_string.go:12-15 stringToStringValue
pub struct stringToStringValue {
    value: *mut map<string, string>,
    changed: bool,
}
unsafe impl Send for stringToStringValue {}
unsafe impl Sync for stringToStringValue {}

// go: github.com/spf13/pflag@v1.0.10 string_to_string.go:17-22 newStringToStringValue
pub fn newStringToStringValue(val: map<string, string>, p: *mut map<string, string>) -> stringToStringValue {
    let ssv = stringToStringValue { value: p, changed: false };
    unsafe {
        *ssv.value = val;
    }
    return ssv;
}

impl Value for stringToStringValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(stringToStringValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_string.go:65-85 stringToStringValue.String
    /// Unlike its integer siblings this SORTS the keys before rendering,
    /// so the output is deterministic; the CSV writer then quotes any
    /// value containing a comma.
    fn String(&self) -> string {
        let m = unsafe { (*self.value).clone() };
        let mut keys: slice<string> = make!([]string, 0);
        for (k, _) in m.__iter() {
            keys = append!(keys, k.clone());
        }
        sort::Strings!(&mut keys);

        let mut records: slice<string> = make!([]string, 0);
        for i in 0..keys.Len() {
            let k = keys[i].clone();
            let v = m.Get(k.clone()).0;
            records = append!(records, k + string("=") + v);
        }
        let (out, _) = writeAsCSV(records);
        return string("[") + strings::TrimSpace(out) + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_string.go:25-59 stringToStringValue.Set
    /// Go switches on the number of '=' in the input: exactly one means a
    /// single pair whose value may hold commas, so it is taken whole
    /// (quotes trimmed); more than one goes through the CSV reader.
    /// Splitting on commas unconditionally would corrupt `--m=k=a,b`.
    fn Set_str(&mut self, val: string) -> error {
        let mut ss: slice<string> = make!([]string, 0);
        let n = strings::Count(val.clone(), string("="));
        if n == 0 {
            return fmt::Errorf!("%s must be formatted as key=value", val);
        } else if n == 1 {
            ss = append!(ss, strings::Trim(val.clone(), string("\"")));
        } else {
            let (parsed, err) = readAsCSV(val.clone());
            if err != nil && !errors::Is(err.clone(), io::EOF) {
                return err;
            }
            ss = parsed;
        }

        let mut out: map<string, string> = map::new();
        for i in 0..ss.Len() {
            let pair = ss[i].clone();
            let kv = strings::SplitN(pair.clone(), string("="), 2);
            if kv.Len() != 2 {
                return fmt::Errorf!("%s must be formatted as key=value", pair);
            }
            out.Set(kv[0].clone(), kv[1].clone());
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

    // go: github.com/spf13/pflag@v1.0.10 string_to_string.go:61-63 stringToStringValue.Type
    fn Type(&self) -> string {
        return string("stringToString");
    }
}

// go: github.com/spf13/pflag@v1.0.10 string_to_string.go:87-107 stringToStringConv
pub fn stringToStringConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "An empty string would cause an empty map"
    if val.Len() == 0 {
        let empty: map<string, string> = map::new();
        return (goish::goany::Any::new(empty), nil.into());
    }
    let (ss, err) = readAsCSV(val);
    if err != nil && !errors::Is(err.clone(), io::EOF) {
        return (goish::goany::Any::from(nil), err);
    }
    let mut out: map<string, string> = map::new();
    for i in 0..ss.Len() {
        let pair = ss[i].clone();
        let kv = strings::SplitN(pair.clone(), string("="), 2);
        if kv.Len() != 2 {
            return (goish::goany::Any::from(nil),
                    fmt::Errorf!("%s must be formatted as key=value", pair));
        }
        out.Set(kv[0].clone(), kv[1].clone());
    }
    return (goish::goany::Any::new(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 string_to_string.go:110-116 FlagSet.GetStringToString
    pub fn GetStringToString<S: Into<string>>(&self, name: S) -> (map<string, string>, error) {
        let (val, err) = self.getFlagType(name.into(), string("stringToString"), stringToStringConv);
        if err != nil {
            return (map::new(), err);
        }
        return (val.As::<map<string, string>>().cloned().unwrap_or(map::new()), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_string.go:121-123 FlagSet.StringToStringVar
    pub fn StringToStringVar(&mut self, p: *mut map<string, string>, name: string, value: map<string, string>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newStringToStringValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_string.go:126-128 FlagSet.StringToStringVarP
    pub fn StringToStringVarP(&mut self, p: *mut map<string, string>, name: string, shorthand: string, value: map<string, string>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newStringToStringValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_string.go:145-149 FlagSet.StringToString
    pub fn StringToString(&mut self, name: string, value: map<string, string>, usage: string) -> *mut map<string, string> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(map::new()));
        self.StringToStringVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 string_to_string.go:152-156 FlagSet.StringToStringP
    pub fn StringToStringP(&mut self, name: string, shorthand: string, value: map<string, string>, usage: string) -> *mut map<string, string> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(map::new()));
        self.StringToStringVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 string_to_string.go:133-135 StringToStringVar
pub fn StringToStringVar(p: *mut map<string, string>, name: string, value: map<string, string>, usage: string) {
    COMMAND_LINE.Lock().StringToStringVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_string.go:138-140 StringToStringVarP
pub fn StringToStringVarP(p: *mut map<string, string>, name: string, shorthand: string, value: map<string, string>, usage: string) {
    COMMAND_LINE.Lock().StringToStringVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_string.go:161-163 StringToString
pub fn StringToString(name: string, value: map<string, string>, usage: string) -> *mut map<string, string> {
    return COMMAND_LINE.Lock().StringToStringP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 string_to_string.go:166-168 StringToStringP
pub fn StringToStringP(name: string, shorthand: string, value: map<string, string>, usage: string) -> *mut map<string, string> {
    return COMMAND_LINE.Lock().StringToStringP(name, shorthand, value, usage);
}
