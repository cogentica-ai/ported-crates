// go: file ip.go decls: ipValue, newIPValue, ipValue.String, ipValue.Set, ipValue.Type, ipConv, FlagSet.GetIP, FlagSet.IPVar, FlagSet.IPVarP, IPVar, IPVarP, FlagSet.IP, FlagSet.IPP, IP, IPP
//
// ip.go — a net.IP flag.

use crate::*;
use goish::net;

// go: github.com/spf13/pflag@v1.0.10 ip.go:10-10 ipValue
pub struct ipValue {
    ptr: *mut net::IP,
}
unsafe impl Send for ipValue {}
unsafe impl Sync for ipValue {}

// go: github.com/spf13/pflag@v1.0.10 ip.go:12-15 newIPValue
pub fn newIPValue(val: net::IP, p: *mut net::IP) -> ipValue {
    unsafe {
        *p = val;
    }
    return ipValue { ptr: p };
}

impl Value for ipValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(ipValue { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 ip.go:17-17 ipValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.ptr).clone() };
        return v.String();
    }

    // go: github.com/spf13/pflag@v1.0.10 ip.go:18-28 ipValue.Set
    /// Go returns nil for the EMPTY string — an unset IP flag is not an
    /// error — and only rejects a non-empty value that fails to parse.
    fn Set_str(&mut self, s: string) -> error {
        if s == "" {
            return nil.into();
        }
        let ip = net::ParseIP(strings::TrimSpace(s.clone()));
        if ip.IsNil() {
            return fmt::Errorf!("failed to parse IP: %q", s);
        }
        unsafe {
            *self.ptr = ip;
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 ip.go:30-32 ipValue.Type
    fn Type(&self) -> string {
        return string("ip");
    }
}

// go: github.com/spf13/pflag@v1.0.10 ip.go:34-40 ipConv
pub fn ipConv(sval: string) -> (goish::goany::Any, error) {
    let ip = net::ParseIP(sval.clone());
    if !ip.IsNil() {
        return (// goish's net::IP implements neither PartialEq nor Reflect, so
        // Any::new/new_opaque do not apply; new_fn is the constructor
        // whose bound (Send + Sync only) this payload satisfies.
        goish::goany::Any::new_fn(ip), nil.into());
    }
    return (
        goish::goany::Any::from(nil),
        fmt::Errorf!("invalid string being converted to IP address: %s", sval),
    );
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 ip.go:43-49 FlagSet.GetIP
    pub fn GetIP<S: Into<string>>(&self, name: S) -> (net::IP, error) {
        let (val, err) = self.getFlagType(name.into(), string("ip"), ipConv);
        if err != nil {
            return (net::IP::default(), err);
        }
        return (val.As::<net::IP>().cloned().unwrap_or(net::IP::default()), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 ip.go:53-55 FlagSet.IPVar
    pub fn IPVar(&mut self, p: *mut net::IP, name: string, value: net::IP, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ip.go:58-60 FlagSet.IPVarP
    pub fn IPVarP(&mut self, p: *mut net::IP, name: string, shorthand: string, value: net::IP, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ip.go:75-79 FlagSet.IP
    pub fn IP(&mut self, name: string, value: net::IP, usage: string) -> *mut net::IP {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(net::IP::default()));
        self.IPVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 ip.go:82-86 FlagSet.IPP
    pub fn IPP(&mut self, name: string, shorthand: string, value: net::IP, usage: string) -> *mut net::IP {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(net::IP::default()));
        self.IPVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 ip.go:64-66 IPVar
pub fn IPVar(p: *mut net::IP, name: string, value: net::IP, usage: string) {
    COMMAND_LINE.Lock().IPVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ip.go:69-71 IPVarP
pub fn IPVarP(p: *mut net::IP, name: string, shorthand: string, value: net::IP, usage: string) {
    COMMAND_LINE.Lock().IPVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ip.go:90-92 IP
pub fn IP(name: string, value: net::IP, usage: string) -> *mut net::IP {
    return COMMAND_LINE.Lock().IPP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ip.go:95-97 IPP
pub fn IPP(name: string, shorthand: string, value: net::IP, usage: string) -> *mut net::IP {
    return COMMAND_LINE.Lock().IPP(name, shorthand, value, usage);
}
