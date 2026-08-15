// go: file ipnet.go decls: ipNetValue, ipNetValue.String, ipNetValue.Set, ipNetValue.Type, newIPNetValue, ipNetConv, FlagSet.GetIPNet, FlagSet.IPNetVar, FlagSet.IPNetVarP, IPNetVar, IPNetVarP, FlagSet.IPNet, FlagSet.IPNetP, IPNet, IPNetP
//
// ipnet.go — a net.IPNet (CIDR) flag.

use crate::*;
use goish::net;

// go: github.com/spf13/pflag@v1.0.10 ipnet.go:10-10 ipNetValue
pub struct ipNetValue {
    ptr: *mut net::IPNet,
}
unsafe impl Send for ipNetValue {}
unsafe impl Sync for ipNetValue {}

// go: github.com/spf13/pflag@v1.0.10 ipnet.go:30-33 newIPNetValue
pub fn newIPNetValue(val: net::IPNet, p: *mut net::IPNet) -> ipNetValue {
    unsafe {
        *p = val;
    }
    return ipNetValue { ptr: p };
}

impl Value for ipNetValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(ipNetValue { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet.go:12-15 ipNetValue.String
    fn String(&self) -> string {
        let n = unsafe { (*self.ptr).clone() };
        return n.String();
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet.go:17-24 ipNetValue.Set
    /// Go keeps ParseCIDR's SECOND return (the masked network), not the
    /// first (the address), so `--n=10.0.0.7/8` stores 10.0.0.0/8.
    fn Set_str(&mut self, value: string) -> error {
        let (_, n, err) = net::ParseCIDR(strings::TrimSpace(value));
        if err != nil {
            return err;
        }
        unsafe {
            *self.ptr = n;
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet.go:26-28 ipNetValue.Type
    fn Type(&self) -> string {
        return string("ipNet");
    }
}

// go: github.com/spf13/pflag@v1.0.10 ipnet.go:35-41 ipNetConv
pub fn ipNetConv(sval: string) -> (goish::goany::Any, error) {
    let (_, n, err) = net::ParseCIDR(strings::TrimSpace(sval.clone()));
    if err == nil {
        return (// goish's net::IPNet implements neither PartialEq nor Reflect, so
        // Any::new/new_opaque do not apply; new_fn is the constructor
        // whose bound (Send + Sync only) this payload satisfies.
        goish::goany::Any::new_fn(n), nil.into());
    }
    return (
        goish::goany::Any::from(nil),
        fmt::Errorf!("invalid string being converted to IPNet: %s", sval),
    );
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 ipnet.go:44-50 FlagSet.GetIPNet
    pub fn GetIPNet<S: Into<string>>(&self, name: S) -> (net::IPNet, error) {
        let (val, err) = self.getFlagType(name.into(), string("ipNet"), ipNetConv);
        if err != nil {
            return (net::IPNet::default(), err);
        }
        return (val.As::<net::IPNet>().cloned().unwrap_or(net::IPNet::default()), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet.go:54-56 FlagSet.IPNetVar
    pub fn IPNetVar(&mut self, p: *mut net::IPNet, name: string, value: net::IPNet, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPNetValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet.go:59-61 FlagSet.IPNetVarP
    pub fn IPNetVarP(&mut self, p: *mut net::IPNet, name: string, shorthand: string, value: net::IPNet, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPNetValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet.go:76-80 FlagSet.IPNet
    pub fn IPNet(&mut self, name: string, value: net::IPNet, usage: string) -> *mut net::IPNet {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(net::IPNet::default()));
        self.IPNetVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 ipnet.go:83-87 FlagSet.IPNetP
    pub fn IPNetP(&mut self, name: string, shorthand: string, value: net::IPNet, usage: string) -> *mut net::IPNet {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(net::IPNet::default()));
        self.IPNetVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 ipnet.go:65-67 IPNetVar
pub fn IPNetVar(p: *mut net::IPNet, name: string, value: net::IPNet, usage: string) {
    COMMAND_LINE.Lock().IPNetVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipnet.go:70-72 IPNetVarP
pub fn IPNetVarP(p: *mut net::IPNet, name: string, shorthand: string, value: net::IPNet, usage: string) {
    COMMAND_LINE.Lock().IPNetVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipnet.go:91-93 IPNet
pub fn IPNet(name: string, value: net::IPNet, usage: string) -> *mut net::IPNet {
    return COMMAND_LINE.Lock().IPNetP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipnet.go:96-98 IPNetP
pub fn IPNetP(name: string, shorthand: string, value: net::IPNet, usage: string) -> *mut net::IPNet {
    return COMMAND_LINE.Lock().IPNetP(name, shorthand, value, usage);
}
