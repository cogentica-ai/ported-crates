// go: file ip_slice.go decls: ipSliceValue, newIPSliceValue, ipSliceValue.Set, ipSliceValue.Type, ipSliceValue.String, ipSliceValue.fromString, ipSliceValue.toString, ipSliceValue.Append, ipSliceValue.Replace, ipSliceValue.GetSlice, ipSliceConv, FlagSet.GetIPSlice, FlagSet.IPSliceVar, FlagSet.IPSliceVarP, IPSliceVar, IPSliceVarP, FlagSet.IPSlice, FlagSet.IPSliceP, IPSlice, IPSliceP
//
// ip_slice.go — like bool_slice, Set strips quotes and reads CSV.

use crate::*;
use goish::net;

// go: github.com/spf13/pflag@v1.0.10 ip_slice.go:11-14 ipSliceValue
pub struct ipSliceValue {
    value: *mut slice<net::IP>,
    changed: bool,
}
unsafe impl Send for ipSliceValue {}
unsafe impl Sync for ipSliceValue {}

// go: github.com/spf13/pflag@v1.0.10 ip_slice.go:16-21 newIPSliceValue
pub fn newIPSliceValue(val: slice<net::IP>, p: *mut slice<net::IP>) -> ipSliceValue {
    let isv = ipSliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl ipSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:75-77 ipSliceValue.fromString
    /// Go returns a nil error even when ParseIP fails, so a bad value
    /// reaches the caller as the nil IP rather than an error. Kept.
    fn fromString(&self, val: string) -> (net::IP, error) {
        return (net::ParseIP(strings::TrimSpace(val)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:79-81 ipSliceValue.toString
    fn toString(&self, val: net::IP) -> string {
        return val.String();
    }
}

impl SliceValue for ipSliceValue {
    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:83-90 ipSliceValue.Append
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

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:92-103 ipSliceValue.Replace
    fn Replace(&mut self, val: slice<string>) -> error {
        let mut out: slice<net::IP> = make!([]net::IP, val.Len());
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

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:105-111 ipSliceValue.GetSlice
    fn GetSlice(&self) -> slice<string> {
        let v = unsafe { (*self.value).clone() };
        let mut out: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            out[i] = self.toString(v[i].clone());
        }
        return out;
    }
}

impl Value for ipSliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(ipSliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:63-73 ipSliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut ip_str_slice: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            ip_str_slice[i] = v[i].String();
        }
        let out = write_as_csv(ip_str_slice);
        return string("[") + out + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:25-55 ipSliceValue.Set
    fn Set_str(&mut self, val: string) -> error {
        let rm_quote = strings::NewReplacer(slice!([]string {
            string("\""), string(""),
            string("'"), string(""),
            string("`"), string(""),
        }));
        let (ip_str_slice, err) = read_as_csv(rm_quote.Replace(val));
        if err != nil && !errors::Is(err.clone(), io::EOF) {
            return err;
        }
        let mut out: slice<net::IP> = make!([]net::IP, 0);
        for i in 0..ip_str_slice.Len() {
            let ip_str = ip_str_slice[i].clone();
            let ip = net::ParseIP(strings::TrimSpace(ip_str.clone()));
            if ip.IsNil() {
                return fmt::Errorf!("invalid string being converted to IP address: %s", ip_str);
            }
            out = append!(out, ip);
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

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:58-60 ipSliceValue.Type
    fn Type(&self) -> string {
        return string("ipSlice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 ip_slice.go:113-129 ipSliceConv
pub fn ipSliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        let empty: slice<net::IP> = make!([]net::IP, 0);
        return (goish::goany::Any::new_fn(empty), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<net::IP> = make!([]net::IP, ss.Len());
    for i in 0..ss.Len() {
        let sval = ss[i].clone();
        let ip = net::ParseIP(strings::TrimSpace(sval.clone()));
        if ip.IsNil() {
            return (goish::goany::Any::from(nil),
                    fmt::Errorf!("invalid string being converted to IP address: %s", sval));
        }
        out[i] = ip;
    }
    return (goish::goany::Any::new_fn(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:132-138 FlagSet.GetIPSlice
    pub fn GetIPSlice<S: Into<string>>(&self, name: S) -> (slice<net::IP>, error) {
        let (val, err) = self.getFlagType(name.into(), string("ipSlice"), ipSliceConv);
        if err != nil {
            return (make!([]net::IP, 0), err);
        }
        return (val.As::<slice<net::IP>>().cloned().unwrap_or(make!([]net::IP, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:142-144 FlagSet.IPSliceVar
    pub fn IPSliceVar(&mut self, p: *mut slice<net::IP>, name: string, value: slice<net::IP>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPSliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:147-149 FlagSet.IPSliceVarP
    pub fn IPSliceVarP(&mut self, p: *mut slice<net::IP>, name: string, shorthand: string, value: slice<net::IP>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPSliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:164-168 FlagSet.IPSlice
    pub fn IPSlice(&mut self, name: string, value: slice<net::IP>, usage: string) -> *mut slice<net::IP> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]net::IP, 0)));
        self.IPSliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 ip_slice.go:171-175 FlagSet.IPSliceP
    pub fn IPSliceP(&mut self, name: string, shorthand: string, value: slice<net::IP>, usage: string) -> *mut slice<net::IP> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]net::IP, 0)));
        self.IPSliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 ip_slice.go:153-155 IPSliceVar
pub fn IPSliceVar(p: *mut slice<net::IP>, name: string, value: slice<net::IP>, usage: string) {
    COMMAND_LINE.Lock().IPSliceVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ip_slice.go:158-160 IPSliceVarP
pub fn IPSliceVarP(p: *mut slice<net::IP>, name: string, shorthand: string, value: slice<net::IP>, usage: string) {
    COMMAND_LINE.Lock().IPSliceVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ip_slice.go:179-181 IPSlice
pub fn IPSlice(name: string, value: slice<net::IP>, usage: string) -> *mut slice<net::IP> {
    return COMMAND_LINE.Lock().IPSliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ip_slice.go:184-186 IPSliceP
pub fn IPSliceP(name: string, shorthand: string, value: slice<net::IP>, usage: string) -> *mut slice<net::IP> {
    return COMMAND_LINE.Lock().IPSliceP(name, shorthand, value, usage);
}
