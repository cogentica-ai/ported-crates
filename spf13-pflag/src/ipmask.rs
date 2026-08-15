// go: file ipmask.go decls: ipMaskValue, newIPMaskValue, ipMaskValue.String, ipMaskValue.Set, ipMaskValue.Type, ParseIPv4Mask, parseIPv4Mask, FlagSet.GetIPv4Mask, FlagSet.IPMaskVar, FlagSet.IPMaskVarP, IPMaskVar, IPMaskVarP, FlagSet.IPMask, FlagSet.IPMaskP, IPMask, IPMaskP
//
// ipmask.go — a net.IPMask flag, accepting either dotted form
// (255.255.255.0) or the 8-hex-digit form IPMask.String() emits
// (ffffff00).

use crate::*;
use goish::net;

// go: github.com/spf13/pflag@v1.0.10 ipmask.go:10-10 ipMaskValue
pub struct ipMaskValue {
    ptr: *mut net::IPMask,
}
unsafe impl Send for ipMaskValue {}
unsafe impl Sync for ipMaskValue {}

// go: github.com/spf13/pflag@v1.0.10 ipmask.go:12-15 newIPMaskValue
pub fn newIPMaskValue(val: net::IPMask, p: *mut net::IPMask) -> ipMaskValue {
    unsafe {
        *p = val;
    }
    return ipMaskValue { ptr: p };
}

impl Value for ipMaskValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(ipMaskValue { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 ipmask.go:17-17 ipMaskValue.String
    fn String(&self) -> string {
        let v = unsafe { (*self.ptr).clone() };
        return v.String();
    }

    // go: github.com/spf13/pflag@v1.0.10 ipmask.go:18-25 ipMaskValue.Set
    fn Set_str(&mut self, s: string) -> error {
        let ip = ParseIPv4Mask(s.clone());
        if ip.bytes.Len() == 0 {
            return fmt::Errorf!("failed to parse IP mask: %q", s);
        }
        unsafe {
            *self.ptr = ip;
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 ipmask.go:27-29 ipMaskValue.Type
    fn Type(&self) -> string {
        return string("ipMask");
    }
}

// go: github.com/spf13/pflag@v1.0.10 ipmask.go:33-57 ParseIPv4Mask
/// Go: "ParseIPv4Mask written in IP form (e.g. 255.255.255.0). This
/// function should really belong to the net package."
///
/// DEVIATION, forced by representation: Go indexes `mask[12..16]`
/// because net.ParseIP returns the 16-byte IPv4-in-IPv6 form, whose
/// last four bytes are the address. goish's net::IP is IPv4-only with
/// a 4-byte backing, so the same four bytes are at 0..4. Indexing 12
/// here would panic on every input.
pub fn ParseIPv4Mask(s: string) -> net::IPMask {
    let mut mask = net::ParseIP(s.clone());
    if mask.IsNil() {
        if s.Len() != 8 {
            return net::IPMask::default();
        }
        // Go: "net.IPMask.String() actually outputs things like ffffff00
        // so write a horrible parser for that as well :-("
        let raw: &str = s.as_ref();
        let mut m: slice<int> = make!([]int, 0);
        for i in 0..4usize {
            let b = string("0x") + string::from_bytes(&raw.as_bytes()[2 * i..2 * i + 2]);
            let (d, err) = strconv::ParseInt(b, 0, 0);
            if err != nil {
                return net::IPMask::default();
            }
            m = append!(m, d);
        }
        let dotted = fmt::Sprintf!("%d.%d.%d.%d", m[0], m[1], m[2], m[3]);
        mask = net::ParseIP(dotted);
        if mask.IsNil() {
            return net::IPMask::default();
        }
    }
    let b = mask.bytes.clone();
    return net::IPv4Mask(b[0usize], b[1usize], b[2usize], b[3usize]);
}

// go: github.com/spf13/pflag@v1.0.10 ipmask.go:59-65 parseIPv4Mask
pub fn parseIPv4Mask(sval: string) -> (goish::goany::Any, error) {
    let mask = ParseIPv4Mask(sval.clone());
    if mask.bytes.Len() == 0 {
        return (
            goish::goany::Any::from(nil),
            fmt::Errorf!("unable to parse %s as net.IPMask", sval),
        );
    }
    // net::IPMask implements neither PartialEq nor Reflect; new_fn is
    // the constructor whose bound this payload satisfies.
    return (goish::goany::Any::new_fn(mask), nil.into());
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 ipmask.go:68-74 FlagSet.GetIPv4Mask
    pub fn GetIPv4Mask<S: Into<string>>(&self, name: S) -> (net::IPMask, error) {
        let (val, err) = self.getFlagType(name.into(), string("ipMask"), parseIPv4Mask);
        if err != nil {
            return (net::IPMask::default(), err);
        }
        return (val.As::<net::IPMask>().cloned().unwrap_or(net::IPMask::default()), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 ipmask.go:78-80 FlagSet.IPMaskVar
    pub fn IPMaskVar(&mut self, p: *mut net::IPMask, name: string, value: net::IPMask, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPMaskValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ipmask.go:83-85 FlagSet.IPMaskVarP
    pub fn IPMaskVarP(&mut self, p: *mut net::IPMask, name: string, shorthand: string, value: net::IPMask, usage: string) {
        self.VarP(alloc::boxed::Box::new(newIPMaskValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 ipmask.go:100-104 FlagSet.IPMask
    pub fn IPMask(&mut self, name: string, value: net::IPMask, usage: string) -> *mut net::IPMask {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(net::IPMask::default()));
        self.IPMaskVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 ipmask.go:107-111 FlagSet.IPMaskP
    pub fn IPMaskP(&mut self, name: string, shorthand: string, value: net::IPMask, usage: string) -> *mut net::IPMask {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(net::IPMask::default()));
        self.IPMaskVarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 ipmask.go:89-91 IPMaskVar
pub fn IPMaskVar(p: *mut net::IPMask, name: string, value: net::IPMask, usage: string) {
    COMMAND_LINE.Lock().IPMaskVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipmask.go:94-96 IPMaskVarP
pub fn IPMaskVarP(p: *mut net::IPMask, name: string, shorthand: string, value: net::IPMask, usage: string) {
    COMMAND_LINE.Lock().IPMaskVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipmask.go:115-117 IPMask
pub fn IPMask(name: string, value: net::IPMask, usage: string) -> *mut net::IPMask {
    return COMMAND_LINE.Lock().IPMaskP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 ipmask.go:120-122 IPMaskP
pub fn IPMaskP(name: string, shorthand: string, value: net::IPMask, usage: string) -> *mut net::IPMask {
    return COMMAND_LINE.Lock().IPMaskP(name, shorthand, value, usage);
}
