// go: file text.go decls: textValue, newTextValue, textValue.Set, textValue.Get, textValue.String, textValue.Type, FlagSet.GetText, FlagSet.TextVar, FlagSet.TextVarP, TextVar, TextVarP
//
// text.go — a flag backed by encoding.TextUnmarshaler, copied in Go from
// go 1.23.4's flag.go.
//
// DEVIATION, structural: Go drives this with reflect — newTextValue
// panics if the default's dynamic type differs from the pointee's, and
// Type() answers reflect.ValueOf(v.p).Type().Name(). goish's reflect is
// a value tree with no Type().Name(), and Rust's type system already
// rejects the mismatch newTextValue guards against at COMPILE time. So
// the type name is captured from core::any::type_name at construction
// (last path segment, matching Go's unqualified Name()), and the panic
// arm is unreachable rather than ported.

use crate::*;
use goish::encoding::{TextMarshaler, TextUnmarshaler};

// go: github.com/spf13/pflag@v1.0.10 text.go:10-10 textValue
pub struct textValue {
    p: alloc::boxed::Box<dyn TextUnmarshaler + Send + Sync>,
    type_name: string,
    marshal: Option<alloc::boxed::Box<dyn Fn() -> (slice<byte>, error) + Send + Sync>>,
}

// go: github.com/spf13/pflag@v1.0.10 text.go:12-26 newTextValue
/// Go's reflect checks collapse into the signature: `p` is the pointee
/// and `val` its default, so a mismatch cannot be expressed.
pub fn newTextValue<T>(val: slice<byte>, p: alloc::boxed::Box<T>) -> textValue
where T: TextUnmarshaler + Send + Sync + 'static {
    let mut p = p;
    // Go: ptrVal.Elem().Set(defVal) — seed the variable with the default.
    let _ = p.UnmarshalText(val);
    let full = core::any::type_name::<T>();
    let short = full.rsplit("::").next().unwrap_or(full);
    return textValue {
        p,
        type_name: string::from_bytes(short.as_bytes()),
        marshal: None,
    };
}

impl Value for textValue {
    // go: none — Goish glue. The boxed TextUnmarshaler cannot be cloned.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        panic!("pflag: a text flag's Value cannot be cloned into another FlagSet")
    }

    // go: github.com/spf13/pflag@v1.0.10 text.go:36-43 textValue.String
    /// Go asks whether p also implements TextMarshaler and returns "" if
    /// it does not, or if MarshalText errors.
    fn String(&self) -> string {
        match self.marshal {
            None => string(""),
            Some(ref m) => {
                let (b, err) = m();
                if err != nil {
                    return string("");
                }
                return string::from_bytes(b.as_ref());
            }
        }
    }

    // go: github.com/spf13/pflag@v1.0.10 text.go:28-30 textValue.Set
    fn Set_str(&mut self, s: string) -> error {
        return self.p.UnmarshalText(bytes(s));
    }

    // go: github.com/spf13/pflag@v1.0.10 text.go:47-49 textValue.Type
    fn Type(&self) -> string {
        return self.type_name.clone();
    }
}

impl textValue {
    // go: github.com/spf13/pflag@v1.0.10 text.go:32-34 textValue.Get
    /// Go returns the TextUnmarshaler as interface{}; the port reports
    /// its type name, since a Box<dyn Trait> cannot be handed out by
    /// value without cloning it.
    pub fn Get(&self) -> string {
        return self.type_name.clone();
    }

    // go: none — Goish glue: lets a caller attach the TextMarshaler half
    // that Go recovers with a type assertion inside String().
    pub fn __set_marshaler<M>(&mut self, m: M)
    where M: Fn() -> (slice<byte>, error) + Send + Sync + 'static {
        self.marshal = Some(alloc::boxed::Box::new(m));
    }
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 text.go:52-61 FlagSet.GetText
    pub fn GetText<S: Into<string>>(&self, name: S, out: &mut dyn TextUnmarshaler) -> error {
        let name = name.into();
        let flag = match self.Lookup(name.clone()) {
            None => return fmt::Errorf!("flag accessed but not defined: %s", name),
            Some(f) => f,
        };
        return out.UnmarshalText(bytes(flag.Value.String()));
    }

    // go: github.com/spf13/pflag@v1.0.10 text.go:64-66 FlagSet.TextVar
    pub fn TextVar<T>(&mut self, p: alloc::boxed::Box<T>, name: string, value: slice<byte>, usage: string)
    where T: TextUnmarshaler + Send + Sync + 'static {
        self.VarP(alloc::boxed::Box::new(newTextValue(value, p)), name, string(""), usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 text.go:69-71 FlagSet.TextVarP
    pub fn TextVarP<T>(&mut self, p: alloc::boxed::Box<T>, name: string, shorthand: string, value: slice<byte>, usage: string)
    where T: TextUnmarshaler + Send + Sync + 'static {
        self.VarP(alloc::boxed::Box::new(newTextValue(value, p)), name, shorthand, usage);
    }
}

// go: github.com/spf13/pflag@v1.0.10 text.go:74-76 TextVar
pub fn TextVar<T>(p: alloc::boxed::Box<T>, name: string, value: slice<byte>, usage: string)
where T: TextUnmarshaler + Send + Sync + 'static {
    COMMAND_LINE.Lock().TextVar(p, name, value, usage);
}

// go: github.com/spf13/pflag@v1.0.10 text.go:79-81 TextVarP
pub fn TextVarP<T>(p: alloc::boxed::Box<T>, name: string, shorthand: string, value: slice<byte>, usage: string)
where T: TextUnmarshaler + Send + Sync + 'static {
    COMMAND_LINE.Lock().TextVarP(p, name, shorthand, value, usage);
}
