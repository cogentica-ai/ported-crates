// go: file ipnet_slice.go decls: ipNetSliceValue, newIPNetSliceValue, ipNetSliceValue.Set, ipNetSliceValue.Type, ipNetSliceValue.String, ipNetSliceConv, FlagSet.GetIPNetSlice, FlagSet.IPNetSliceVar, FlagSet.IPNetSliceVarP, IPNetSliceVar, IPNetSliceVarP, FlagSet.IPNetSlice, FlagSet.IPNetSliceP, IPNetSlice, IPNetSliceP
//
// ipnet_slice.go — 15 decls: unlike ip_slice it carries no
// fromString/toString and does not implement SliceValue.

use crate::*;
use goish::net;

// go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:11-14 ipNetSliceValue
pub struct ipNetSliceValue {
    value: *mut slice<net::IPNet>,
    changed: bool,
}
unsafe impl Send for ipNetSliceValue {}
unsafe impl Sync for ipNetSliceValue {}

// go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:16-21 newIPNetSliceValue
pub fn newIPNetSliceValue(val: slice<net::IPNet>, p: *mut slice<net::IPNet>) -> ipNetSliceValue {
    let isv = ipNetSliceValue { value: p, changed: false };
    unsafe {
        *isv.value = val;
    }
    return isv;
}

impl Value for ipNetSliceValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(ipNetSliceValue { value: self.value, changed: self.changed })
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:63-72 ipNetSliceValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.value).clone() };
        let mut s: slice<string> = make!([]string, v.Len());
        for i in 0..v.Len() {
            s[i] = v[i].String();
        }
        let (out, _) = writeAsCSV(s);
        return string("[") + out + string("]");
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:25-55 ipNetSliceValue.Set
    fn Set_str(&mut self, val: string) -> error {
        let rm_quote = strings::NewReplacer(slice!([]string {
            string("\""), string(""),
            string("'"), string(""),
            string("`"), string(""),
        }));
        let (str_slice, err) = readAsCSV(rm_quote.Replace(val));
        if err != nil && !errors::Is(err.clone(), io::EOF) {
            return err;
        }
        let mut out: slice<net::IPNet> = make!([]net::IPNet, 0);
        for i in 0..str_slice.Len() {
            let s = str_slice[i].clone();
            let (_, n, e) = net::ParseCIDR(strings::TrimSpace(s.clone()));
            if e != nil {
                return fmt::Errorf!("invalid string being converted to CIDR: %s", s);
            }
            out = append!(out, n);
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

    // go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:58-60 ipNetSliceValue.Type
    fn Type(&self) -> string {
        return string("ipNetSlice");
    }
}

// go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:74-90 ipNetSliceConv
pub fn ipNetSliceConv(val: string) -> (goish::goany::Any, error) {
    let val = strings::Trim(val, string("[]"));
    // Go: "Empty string would cause a slice with one (empty) entry"
    if val.Len() == 0 {
        let empty: slice<net::IPNet> = make!([]net::IPNet, 0);
        return (goish::goany::Any::new_fn(empty), nil.into());
    }
    let ss = strings::Split(val, string(","));
    let mut out: slice<net::IPNet> = make!([]net::IPNet, ss.Len());
    for i in 0..ss.Len() {
        let sval = ss[i].clone();
        let (_, n, e) = net::ParseCIDR(strings::TrimSpace(sval.clone()));
        if e != nil {
            return (goish::goany::Any::from(nil),
                    fmt::Errorf!("invalid string being converted to CIDR: %s", sval));
        }
        out[i] = n;
    }
    return (goish::goany::Any::new_fn(out), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:93-99 FlagSet.GetIPNetSlice
    pub fn GetIPNetSlice<S: Into<string>>(&self, name: S) -> (slice<net::IPNet>, error) {
        let (val, err) = self.getFlagType(name.into(), string("ipNetSlice"), ipNetSliceConv);
        if err != nil {
            return (make!([]net::IPNet, 0), err);
        }
        return (val.As::<slice<net::IPNet>>().cloned().unwrap_or(make!([]net::IPNet, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:103-105 FlagSet.IPNetSliceVar
    pub fn IPNetSliceVar(&mut self, p: *mut slice<net::IPNet>, name: string, value: slice<net::IPNet>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPNetSliceValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:108-110 FlagSet.IPNetSliceVarP
    pub fn IPNetSliceVarP(&mut self, p: *mut slice<net::IPNet>, name: string, shorthand: string, value: slice<net::IPNet>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPNetSliceValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:125-129 FlagSet.IPNetSlice
    pub fn IPNetSlice(&mut self, name: string, value: slice<net::IPNet>, usage: string) -> *mut slice<net::IPNet> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]net::IPNet, 0)));
        self.IPNetSliceVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:132-136 FlagSet.IPNetSliceP
    pub fn IPNetSliceP(&mut self, name: string, shorthand: string, value: slice<net::IPNet>, usage: string) -> *mut slice<net::IPNet> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]net::IPNet, 0)));
        self.IPNetSliceVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:114-116 IPNetSliceVar
pub fn IPNetSliceVar(p: *mut slice<net::IPNet>, name: string, value: slice<net::IPNet>, usage: string) {
    COMMAND_LINE.Lock().IPNetSliceVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:119-121 IPNetSliceVarP
pub fn IPNetSliceVarP(p: *mut slice<net::IPNet>, name: string, shorthand: string, value: slice<net::IPNet>, usage: string) {
    COMMAND_LINE.Lock().IPNetSliceVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:140-142 IPNetSlice
pub fn IPNetSlice(name: string, value: slice<net::IPNet>, usage: string) -> *mut slice<net::IPNet> {
    return COMMAND_LINE.Lock().IPNetSliceP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipnet_slice.go:145-147 IPNetSliceP
pub fn IPNetSliceP(name: string, shorthand: string, value: slice<net::IPNet>, usage: string) -> *mut slice<net::IPNet> {
    return COMMAND_LINE.Lock().IPNetSliceP(name, shorthand, value, usage);
}
