// go: file bytes.go decls: bytesHexValue, bytesHexValue.String, bytesHexValue.Set, bytesHexValue.Type, newBytesHexValue, bytesHexConv, FlagSet.GetBytesHex, FlagSet.BytesHexVar, FlagSet.BytesHexVarP, BytesHexVar, BytesHexVarP, FlagSet.BytesHex, FlagSet.BytesHexP, BytesHex, BytesHexP, bytesBase64Value, bytesBase64Value.String, bytesBase64Value.Set, bytesBase64Value.Type, newBytesBase64Value, bytesBase64ValueConv, FlagSet.GetBytesBase64, FlagSet.BytesBase64Var, FlagSet.BytesBase64VarP, BytesBase64Var, BytesBase64VarP, FlagSet.BytesBase64, FlagSet.BytesBase64P, BytesBase64, BytesBase64P
//
// bytes.go — two []byte flags in one file, differing only in encoding:
// hex (uppercase on the way out) and standard base64.

use crate::*;
use goish::encoding::base64;
use goish::encoding::hex;

// go: github.com/spf13/pflag@v1.0.10 bytes.go:11-11 bytesHexValue
pub struct bytesHexValue {
    ptr: *mut slice<byte>,
}
unsafe impl Send for bytesHexValue {}
unsafe impl Sync for bytesHexValue {}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:36-39 newBytesHexValue
pub fn newBytesHexValue(val: slice<byte>, p: *mut slice<byte>) -> bytesHexValue {
    unsafe {
        *p = val;
    }
    return bytesHexValue { ptr: p };
}

impl Value for bytesHexValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(bytesHexValue { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:14-16 bytesHexValue.String
    /// Go renders with `%X` — UPPERCASE hex. goish's hex::EncodeToString
    /// emits lowercase, so the case is applied explicitly; without it the
    /// flag's printed default would differ from Go's by case alone.
    fn String(&self) -> string {
        let v = unsafe { (*self.ptr).clone() };
        return strings::ToUpper(hex::EncodeToString(v.as_ref()));
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:19-29 bytesHexValue.Set
    fn Set_str(&mut self, value: string) -> error {
        let trimmed = strings::TrimSpace(value);
        let (bin, err) = hex::DecodeString(trimmed.as_ref());
        if err != nil {
            return err;
        }
        unsafe {
            *self.ptr = bin;
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:32-34 bytesHexValue.Type
    fn Type(&self) -> string {
        return string("bytesHex");
    }
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:41-50 bytesHexConv
pub fn bytesHexConv(sval: string) -> (goish::goany::Any, error) {
    let (bin, err) = hex::DecodeString(sval.as_ref());
    if err == nil {
        return (goish::goany::Any::new(bin), nil.into());
    }
    return (
        goish::goany::Any::from(nil),
        fmt::Errorf!("invalid string being converted to Bytes: %s %s", sval, err),
    );
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:112-112 bytesBase64Value
pub struct bytesBase64Value {
    ptr: *mut slice<byte>,
}
unsafe impl Send for bytesBase64Value {}
unsafe impl Sync for bytesBase64Value {}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:137-140 newBytesBase64Value
pub fn newBytesBase64Value(val: slice<byte>, p: *mut slice<byte>) -> bytesBase64Value {
    unsafe {
        *p = val;
    }
    return bytesBase64Value { ptr: p };
}

impl Value for bytesBase64Value {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(bytesBase64Value { ptr: self.ptr })
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:115-117 bytesBase64Value.String
    fn String(&self) -> string {
        let v = unsafe { (*self.ptr).clone() };
        return base64::StdEncoding.EncodeToString(v.as_ref());
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:120-130 bytesBase64Value.Set
    fn Set_str(&mut self, value: string) -> error {
        let trimmed = strings::TrimSpace(value);
        let (bin, err) = base64::StdEncoding.DecodeString(trimmed.as_ref());
        if err != nil {
            return err;
        }
        unsafe {
            *self.ptr = bin;
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:133-135 bytesBase64Value.Type
    fn Type(&self) -> string {
        return string("bytesBase64");
    }
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:142-150 bytesBase64ValueConv
pub fn bytesBase64ValueConv(sval: string) -> (goish::goany::Any, error) {
    let (bin, err) = base64::StdEncoding.DecodeString(sval.as_ref());
    if err == nil {
        return (goish::goany::Any::new(bin), nil.into());
    }
    return (
        goish::goany::Any::from(nil),
        fmt::Errorf!("invalid string being converted to Bytes: %s %s", sval, err),
    );
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 bytes.go:53-61 FlagSet.GetBytesHex
    pub fn GetBytesHex<S: Into<string>>(&self, name: S) -> (slice<byte>, error) {
        let (val, err) = self.getFlagType(name.into(), string("bytesHex"), bytesHexConv);
        if err != nil {
            return (make!([]byte, 0), err);
        }
        return (val.As::<slice<byte>>().cloned().unwrap_or(make!([]byte, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:65-67 FlagSet.BytesHexVar
    pub fn BytesHexVar(&mut self, p: *mut slice<byte>, name: string, value: slice<byte>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newBytesHexValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:70-72 FlagSet.BytesHexVarP
    pub fn BytesHexVarP(&mut self, p: *mut slice<byte>, name: string, shorthand: string, value: slice<byte>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newBytesHexValue(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:87-91 FlagSet.BytesHex
    pub fn BytesHex(&mut self, name: string, value: slice<byte>, usage: string) -> *mut slice<byte> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]byte, 0)));
        self.BytesHexVarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:94-98 FlagSet.BytesHexP
    pub fn BytesHexP(&mut self, name: string, shorthand: string, value: slice<byte>, usage: string) -> *mut slice<byte> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]byte, 0)));
        self.BytesHexVarP(p, name, shorthand, value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:153-161 FlagSet.GetBytesBase64
    pub fn GetBytesBase64<S: Into<string>>(&self, name: S) -> (slice<byte>, error) {
        let (val, err) = self.getFlagType(name.into(), string("bytesBase64"), bytesBase64ValueConv);
        if err != nil {
            return (make!([]byte, 0), err);
        }
        return (val.As::<slice<byte>>().cloned().unwrap_or(make!([]byte, 0)), nil.into());
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:165-167 FlagSet.BytesBase64Var
    pub fn BytesBase64Var(&mut self, p: *mut slice<byte>, name: string, value: slice<byte>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newBytesBase64Value(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:170-172 FlagSet.BytesBase64VarP
    pub fn BytesBase64VarP(&mut self, p: *mut slice<byte>, name: string, shorthand: string, value: slice<byte>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newBytesBase64Value(value, p)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:187-191 FlagSet.BytesBase64
    pub fn BytesBase64(&mut self, name: string, value: slice<byte>, usage: string) -> *mut slice<byte> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]byte, 0)));
        self.BytesBase64VarP(p, name, string(""), value, usage);
        return p;
    }

    // go: github.com/spf13/pflag@v1.0.10 bytes.go:194-198 FlagSet.BytesBase64P
    pub fn BytesBase64P(&mut self, name: string, shorthand: string, value: slice<byte>, usage: string) -> *mut slice<byte> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(make!([]byte, 0)));
        self.BytesBase64VarP(p, name, shorthand, value, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:76-78 BytesHexVar
pub fn BytesHexVar(p: *mut slice<byte>, name: string, value: slice<byte>, usage: string) {
    COMMAND_LINE.Lock().BytesHexVarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:81-83 BytesHexVarP
pub fn BytesHexVarP(p: *mut slice<byte>, name: string, shorthand: string, value: slice<byte>, usage: string) {
    COMMAND_LINE.Lock().BytesHexVarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:102-104 BytesHex
pub fn BytesHex(name: string, value: slice<byte>, usage: string) -> *mut slice<byte> {
    return COMMAND_LINE.Lock().BytesHexP(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:107-109 BytesHexP
pub fn BytesHexP(name: string, shorthand: string, value: slice<byte>, usage: string) -> *mut slice<byte> {
    return COMMAND_LINE.Lock().BytesHexP(name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:176-178 BytesBase64Var
pub fn BytesBase64Var(p: *mut slice<byte>, name: string, value: slice<byte>, usage: string) {
    COMMAND_LINE.Lock().BytesBase64VarP(p, name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:181-183 BytesBase64VarP
pub fn BytesBase64VarP(p: *mut slice<byte>, name: string, shorthand: string, value: slice<byte>, usage: string) {
    COMMAND_LINE.Lock().BytesBase64VarP(p, name, shorthand, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:202-204 BytesBase64
pub fn BytesBase64(name: string, value: slice<byte>, usage: string) -> *mut slice<byte> {
    return COMMAND_LINE.Lock().BytesBase64P(name, string(""), value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 bytes.go:207-209 BytesBase64P
pub fn BytesBase64P(name: string, shorthand: string, value: slice<byte>, usage: string) -> *mut slice<byte> {
    return COMMAND_LINE.Lock().BytesBase64P(name, shorthand, value, usage);
}
