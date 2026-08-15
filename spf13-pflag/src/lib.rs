// Port of github.com/spf13/pflag@v1.0.10
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

extern crate alloc;

// One module per Go file, per AGENTS.md §38 — the per-type flag files
// each carry their own decls manifest and anchors.
mod float32;
mod float32_slice;
mod float64_slice;
mod int16;
mod int32_slice;
mod int64_slice;
mod int_pkg;
mod int32_pkg;
mod int64_pkg;
mod uint_pkg;
mod float64_pkg;
mod bool_pkg;
mod string_pkg;
mod duration_pkg;
mod count_pkg;
mod int8;
mod uint_slice;
mod uint16;
mod uint32;
mod uint64;
mod uint8;

pub use crate::float32::*;
pub use crate::float32_slice::*;
pub use crate::float64_slice::*;
pub use crate::int16::*;
pub use crate::int32_slice::*;
pub use crate::int64_slice::*;
pub use crate::int_pkg::*;
pub use crate::int32_pkg::*;
pub use crate::int64_pkg::*;
pub use crate::uint_pkg::*;
pub use crate::float64_pkg::*;
pub use crate::bool_pkg::*;
pub use crate::string_pkg::*;
pub use crate::duration_pkg::*;
pub use crate::count_pkg::*;
pub use crate::int8::*;
pub use crate::uint_slice::*;
pub use crate::uint16::*;
pub use crate::uint32::*;
pub use crate::uint64::*;
pub use crate::uint8::*;

use goish::fmt;
use goish::strings;
use goish::strconv;
use goish::time;
use goish::encoding::csv;
use goish::os;
use goish::io;
use goish::sync;
use goish::errors::{self, ErrorTrait, error};
use goish::{string};
use goish::gomap::map;
use goish::types::{byte, float64};
use goish::lazy::Lazy;
use goish::{nil, Sprintf, append, make, bytes, int, int32, int64, uint, slice};

// ── ErrHelp ────────────────────────────────────────────────────────────────

goish::var! {
    pub ErrHelp: error = "pflag: help requested";
}

// ── ErrorHandling ──────────────────────────────────────────────────────────

pub type ErrorHandling = int;
pub const ContinueOnError: ErrorHandling = 0;
pub const ExitOnError: ErrorHandling = 1;
pub const PanicOnError: ErrorHandling = 2;

// ── NormalizedName ─────────────────────────────────────────────────────────

pub type NormalizedName = string;

// ── ParseErrorsAllowlist ───────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct ParseErrorsAllowlist {
    pub UnknownFlags: bool,
}

// ── Value trait ────────────────────────────────────────────────────────────

pub trait Value: Send + Sync {
    fn String(&self) -> string;
    fn Set_str(&mut self, s: string) -> error;
    fn Type(&self) -> string;
    // go: none — Goish glue. Go shares *Flag pointers across FlagSets
    // (flag.go:885); the port's arena owns flags per-set, so AddFlag must
    // clone the Flag, which needs a clonable Value. Built-in values hold a
    // raw pointer to the bound variable, so the clone still writes the same
    // target. External Value impls that never flow through AddFlag keep the
    // panicking default.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        panic!("pflag: CloneBox not implemented for this Value type");
    }
}

// go: github.com/spf13/pflag@v1.0.10 flag.go:219-226 SliceValue
/// Go: "SliceValue is a secondary interface to all flags which hold a
/// list of values."
///
/// pflag never type-asserts to it internally — it exists for consumers
/// (cobra, viper) to reach a flag's list form — so this is a plain
/// second trait rather than an optional-interface downcast on `Value`.
pub trait SliceValue {
    /// Go: "Append adds the specified value to the end of the flag value list."
    fn Append(&mut self, val: string) -> error;
    /// Go: "Replace will fully overwrite any data currently in the flag value list."
    fn Replace(&mut self, val: slice<string>) -> error;
    /// Go: "GetSlice returns the flag value list as an array of strings."
    fn GetSlice(&self) -> slice<string>;
}

// ── Flag struct ────────────────────────────────────────────────────────────

pub struct Flag {
    pub Name: string,
    pub Shorthand: string,
    pub Usage: string,
    pub Value: alloc::boxed::Box<dyn Value>,
    pub DefValue: string,
    pub Changed: bool,
    pub NoOptDefVal: string,
    pub Deprecated: string,
    pub Hidden: bool,
    pub ShorthandDeprecated: string,
    pub Annotations: map<string, slice<string>>,
}

// ── Error types ────────────────────────────────────────────────────────────

type notExistErrorMessageType = int;
const flagNotExistMessage: notExistErrorMessageType = 0;
const flagNotDefinedMessage: notExistErrorMessageType = 1;
const flagNoSuchFlagMessage: notExistErrorMessageType = 2;
const flagUnknownFlagMessage: notExistErrorMessageType = 3;
const flagUnknownShorthandFlagMessage: notExistErrorMessageType = 4;

#[derive(Clone, Default)]
struct NotExistError {
    name: string,
    specified_shorthands: string,
    message_type: notExistErrorMessageType,
}

impl ErrorTrait for NotExistError {
    fn Error(&self) -> string {
        match self.message_type {
            0 => fmt::Sprintf!("flag %q does not exist", self.name.clone()),
            1 => fmt::Sprintf!("flag accessed but not defined: %s", self.name.clone()),
            2 => fmt::Sprintf!("no such flag -%v", self.name.clone()),
            3 => fmt::Sprintf!("unknown flag: --%s", self.name.clone()),
            4 => {
                let c = self.name[0usize] as char;
                fmt::Sprintf!("unknown shorthand flag: %q in -%s", c, self.specified_shorthands.clone())
            }
            _ => panic!("unknown flagNotExistErrorMessageType"),
        }
    }
}

#[derive(Clone, Default)]
struct ValueRequiredError {
    flag_name: string,
    specified_name: string,
    specified_shorthands: string,
}

impl ErrorTrait for ValueRequiredError {
    fn Error(&self) -> string {
        if self.specified_shorthands.Len() > 0 {
            let c = self.specified_name[0usize] as char;
            fmt::Sprintf!("flag needs an argument: %q in -%s", c, self.specified_shorthands.clone())
        } else {
            fmt::Sprintf!("flag needs an argument: --%s", self.specified_name.clone())
        }
    }
}

#[derive(Clone, Default)]
struct InvalidValueError {
    flag_name: string,
    flag_shorthand: string,
    flag_shorthand_deprecated: string,
    value: string,
    cause: error,
}

impl ErrorTrait for InvalidValueError {
    fn Error(&self) -> string {
        let flag_name: string;
        if self.flag_shorthand.Len() > 0 && self.flag_shorthand_deprecated.Len() == 0 {
            flag_name = fmt::Sprintf!("-%s, --%s", self.flag_shorthand.clone(), self.flag_name.clone());
        } else {
            flag_name = fmt::Sprintf!("--%s", self.flag_name.clone());
        }
        fmt::Sprintf!("invalid argument %q for %q flag: %v", self.value.clone(), flag_name, self.cause.clone())
    }
    fn Unwrap(&self) -> error {
        self.cause.clone()
    }
}

#[derive(Clone, Default)]
struct InvalidSyntaxError {
    specified_flag: string,
}

impl ErrorTrait for InvalidSyntaxError {
    fn Error(&self) -> string {
        fmt::Sprintf!("bad flag syntax: %s", self.specified_flag.clone())
    }
}

// ── FlagSet struct ─────────────────────────────────────────────────────────

pub struct FlagSet {
    pub Usage: Option<alloc::boxed::Box<dyn Fn() + Send + Sync>>,
    pub SortFlags: bool,
    pub ParseErrorsAllowlist: ParseErrorsAllowlist,
    pub ParseErrorsWhitelist: ParseErrorsAllowlist,

    name: string,
    parsed: bool,
    // flags stored as Vec<Box<Flag>>
    flags: alloc::vec::Vec<alloc::boxed::Box<Flag>>,
    // name -> index in flags vec
    formal: map<string, usize>,
    ordered_formal: slice<usize>,
    // shorthands: byte -> index in flags vec
    shorthands: map<byte, usize>,
    // actual (changed) flags: name -> index in flags vec
    actual: map<string, usize>,
    ordered_actual: slice<usize>,
    args: slice<string>,
    args_len_at_dash: int,
    error_handling: ErrorHandling,
    interspersed: bool,
    normalize_name_fn: Option<alloc::boxed::Box<dyn Fn(&FlagSet, string) -> NormalizedName + Send + Sync>>,
}

impl FlagSet {
    fn output_write(&self, s: string) {
        let mut f = os::Stderr();
        let _ = f.Write(bytes(s));
    }

    fn normalize_flag_name(&self, name: string) -> NormalizedName {
        if let Some(ref f) = self.normalize_name_fn {
            f(self, name)
        } else {
            name
        }
    }

    pub fn Name_str(&self) -> string {
        self.name.clone()
    }

    pub fn SetNormalizeFunc(
        &mut self,
        f: alloc::boxed::Box<dyn Fn(&FlagSet, string) -> NormalizedName + Send + Sync>,
    ) {
        self.normalize_name_fn = Some(f);
    }

    pub fn SetInterspersed(&mut self, interspersed: bool) {
        self.interspersed = interspersed;
    }

    pub fn ArgsLenAtDash(&self) -> int {
        self.args_len_at_dash
    }

    pub fn HasFlags(&self) -> bool {
        !self.flags.is_empty()
    }

    pub fn HasAvailableFlags(&self) -> bool {
        for flag in &self.flags {
            if !flag.Hidden {
                return true;
            }
        }
        false
    }

    pub fn NFlag(&self) -> int {
        int(self.actual.Len())
    }

    pub fn NArg(&self) -> int {
        self.args.Len()
    }

    pub fn Args(&self) -> slice<string> {
        self.args.clone()
    }

    pub fn Arg(&self, i: int) -> string {
        if i < 0 || i >= self.args.Len() {
            return string("");
        }
        self.args[i as usize].clone()
    }

    pub fn Parsed(&self) -> bool {
        self.parsed
    }

    pub fn Init<S: Into<string>>(&mut self, name: S, error_handling: ErrorHandling) {
        self.name = name.into();
        self.error_handling = error_handling;
        self.args_len_at_dash = -1;
    }

    pub fn Lookup<S: Into<string>>(&self, name: S) -> Option<&Flag> {
        let name = name.into();
        let norm = self.normalize_flag_name(name);
        let (idx_ref, ok) = self.formal.GetRef(norm.clone()); let idx = idx_ref.copied().unwrap_or(0);
        if !ok {
            return None;
        }
        Some(&self.flags[idx])
    }

    fn lookup_mut<S: Into<string>>(&mut self, name: S) -> Option<&mut Flag> {
        let name = name.into();
        let norm = self.normalize_flag_name(name);
        let (idx_ref, ok) = self.formal.GetRef(norm.clone()); let idx = idx_ref.copied().unwrap_or(0);
        if !ok {
            return None;
        }
        Some(&mut self.flags[idx])
    }

    pub fn ShorthandLookup<S: Into<string>>(&self, name: S) -> Option<&Flag> {
        let name = name.into();
        if name.Len() == 0 {
            return None;
        }
        let c = name[0usize];
        let (idx_ref, ok) = self.shorthands.GetRef(c); let idx = idx_ref.copied().unwrap_or(0);
        if !ok {
            return None;
        }
        Some(&self.flags[idx])
    }

    pub fn Changed<S: Into<string>>(&self, name: S) -> bool {
        match self.Lookup(name) {
            None => false,
            Some(f) => f.Changed,
        }
    }

    pub fn VisitAll(&self, mut fn_: impl FnMut(&Flag)) {
        if self.flags.is_empty() {
            return;
        }
        if self.SortFlags {
            let mut names: alloc::vec::Vec<string> = alloc::vec::Vec::new();
            for flag in &self.flags {
                names.push(flag.Name.clone());
            }
            names.sort_by(|a, b| { let a_s: &str = a.as_ref(); let b_s: &str = b.as_ref(); a_s.cmp(b_s) });
            for name in &names {
                let (idx_ref, ok) = self.formal.GetRef(name.clone()); let idx = idx_ref.copied().unwrap_or(0);
                if ok {
                    fn_(&self.flags[idx]);
                }
            }
        } else {
            let mut i = 0usize;
            while i < self.ordered_formal.Len() as usize {
                let idx = self.ordered_formal[i];
                fn_(&self.flags[idx]);
                i += 1;
            }
        }
    }

    pub fn Visit(&self, mut fn_: impl FnMut(&Flag)) {
        if self.actual.Len() == 0 {
            return;
        }
        if self.SortFlags {
            let mut names: alloc::vec::Vec<string> = alloc::vec::Vec::new();
            let mut i = 0usize;
            while i < self.ordered_actual.Len() as usize {
                let idx = self.ordered_actual[i];
                names.push(self.flags[idx].Name.clone());
                i += 1;
            }
            names.sort_by(|a, b| { let a_s: &str = a.as_ref(); let b_s: &str = b.as_ref(); a_s.cmp(b_s) });
            for name in &names {
                let norm = self.normalize_flag_name(name.clone());
                let (idx_ref2, ok) = self.actual.GetRef(norm.clone()); let idx = idx_ref2.copied().unwrap_or(0);
                if ok {
                    fn_(&self.flags[idx]);
                }
            }
        } else {
            let mut i = 0usize;
            while i < self.ordered_actual.Len() as usize {
                let idx = self.ordered_actual[i];
                fn_(&self.flags[idx]);
                i += 1;
            }
        }
    }

    pub fn Set<N: Into<string>, V: Into<string>>(&mut self, name: N, value: V) -> error {
        let name = name.into();
        let value = value.into();
        let norm = self.normalize_flag_name(name.clone());
        let (idx_ref, ok) = self.formal.GetRef(norm.clone()); let idx = idx_ref.copied().unwrap_or(0);
        if !ok {
            return errors::Wrap(NotExistError {
                name: name,
                message_type: flagNoSuchFlagMessage,
                ..Default::default()
            });
        }
        let err = self.flags[idx].Value.Set_str(value.clone());
        if err != nil {
            return errors::Wrap(InvalidValueError {
                flag_name: self.flags[idx].Name.clone(),
                flag_shorthand: self.flags[idx].Shorthand.clone(),
                flag_shorthand_deprecated: self.flags[idx].ShorthandDeprecated.clone(),
                value: value,
                cause: err,
            });
        }
        if !self.flags[idx].Changed {
            let norm2 = self.normalize_flag_name(self.flags[idx].Name.clone());
            self.actual.Set(norm2.clone(), idx);
            self.ordered_actual = append!(self.ordered_actual.clone(), idx);
            self.flags[idx].Changed = true;
        }
        if self.flags[idx].Deprecated.Len() != 0 {
            let dep = self.flags[idx].Deprecated.clone();
            let fname = self.flags[idx].Name.clone();
            self.output_write(fmt::Sprintf!("Flag --%s has been deprecated, %s\n", fname, dep));
        }
        nil.into()
    }

    pub fn MarkDeprecated<S1: Into<string>, S2: Into<string>>(&mut self, name: S1, usage_message: S2) -> error {
        let name = name.into();
        let usage_message = usage_message.into();
        if usage_message.Len() == 0 {
            return fmt::Errorf!("deprecated message for flag %q must be set", name);
        }
        match self.lookup_mut(name.clone()) {
            None => errors::Wrap(NotExistError {
                name: name,
                message_type: flagNotExistMessage,
                ..Default::default()
            }),
            Some(flag) => {
                flag.Deprecated = usage_message;
                flag.Hidden = true;
                nil.into()
            }
        }
    }

    pub fn MarkHidden<S: Into<string>>(&mut self, name: S) -> error {
        let name = name.into();
        match self.lookup_mut(name.clone()) {
            None => errors::Wrap(NotExistError {
                name: name,
                message_type: flagNotExistMessage,
                ..Default::default()
            }),
            Some(flag) => {
                flag.Hidden = true;
                nil.into()
            }
        }
    }

    pub fn MarkShorthandDeprecated<S1: Into<string>, S2: Into<string>>(&mut self, name: S1, usage_message: S2) -> error {
        let name = name.into();
        let usage_message = usage_message.into();
        if usage_message.Len() == 0 {
            return fmt::Errorf!("deprecated message for flag %q must be set", name);
        }
        match self.lookup_mut(name.clone()) {
            None => errors::Wrap(NotExistError {
                name: name,
                message_type: flagNotExistMessage,
                ..Default::default()
            }),
            Some(flag) => {
                flag.ShorthandDeprecated = usage_message;
                nil.into()
            }
        }
    }

    pub fn SetAnnotation<S1: Into<string>, S2: Into<string>>(
        &mut self,
        name: S1,
        key: S2,
        values: slice<string>,
    ) -> error {
        let name = name.into();
        let key = key.into();
        match self.lookup_mut(name.clone()) {
            None => errors::Wrap(NotExistError {
                name: name,
                message_type: flagNoSuchFlagMessage,
                ..Default::default()
            }),
            Some(flag) => {
                flag.Annotations.Set(key, values);
                nil.into()
            }
        }
    }

    pub fn Var(&mut self, value: alloc::boxed::Box<dyn Value>, name: string, usage: string) {
        self.VarP(value, name, string(""), usage);
    }

    pub fn VarP(
        &mut self,
        value: alloc::boxed::Box<dyn Value>,
        name: string,
        shorthand: string,
        usage: string,
    ) {
        let _ = self.VarPF(value, name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 flag.go:404-422 FlagSet.getFlagType
    /// The shared spine of every `GetX`: look the flag up, reject a type
    /// mismatch, then run the caller's conv func over the string form.
    /// Go's `interface{}` result is `goany::Any` here, which is what lets
    /// the ~40 `xConv` functions port with their real signature instead
    /// of each `GetX` hand-inlining its own parse.
    pub fn getFlagType(
        &self,
        name: string,
        ftype: string,
        convFunc: fn(string) -> (goish::goany::Any, error),
    ) -> (goish::goany::Any, error) {
        let flag = match self.Lookup(name.clone()) {
            None => {
                return (
                    goish::goany::Any::from(nil),
                    errors::Wrap(NotExistError {
                        name,
                        message_type: flagNotDefinedMessage,
                        ..Default::default()
                    }),
                );
            }
            Some(f) => f,
        };
        if flag.Value.Type() != ftype {
            return (
                goish::goany::Any::from(nil),
                fmt::Errorf!(
                    "trying to get %s value of flag of type %s",
                    ftype,
                    flag.Value.Type()
                ),
            );
        }
        let sval = flag.Value.String();
        let (result, err) = convFunc(sval);
        if err != nil {
            return (goish::goany::Any::from(nil), err);
        }
        return (result, nil.into());
    }

    pub fn VarPF(
        &mut self,
        value: alloc::boxed::Box<dyn Value>,
        name: string,
        shorthand: string,
        usage: string,
    ) -> usize {
        let def_value = value.String();
        let flag = alloc::boxed::Box::new(Flag {
            Name: name,
            Shorthand: shorthand,
            Usage: usage,
            Value: value,
            DefValue: def_value,
            Changed: false,
            NoOptDefVal: string(""),
            Deprecated: string(""),
            Hidden: false,
            ShorthandDeprecated: string(""),
            Annotations: make!(map[string]slice<string>),
        });
        self.add_flag(flag)
    }

    fn add_flag(&mut self, flag: alloc::boxed::Box<Flag>) -> usize {
        let norm = self.normalize_flag_name(flag.Name.clone());
        let already = self.formal.Has(norm.clone());
        if already {
            let msg = fmt::Sprintf!("%s flag redefined: %s", self.name.clone(), flag.Name.clone());
            self.output_write(msg.clone());
            panic!("flag redefined");
        }
        let shorthand = flag.Shorthand.clone();
        self.flags.push(flag);
        let idx = self.flags.len() - 1;
        self.flags[idx].Name = norm.clone();
        self.formal.Set(norm.clone(), idx);
        self.ordered_formal = append!(self.ordered_formal.clone(), idx);
        if shorthand.Len() == 0 {
            return idx;
        }
        if shorthand.Len() > 1 {
            let msg = fmt::Sprintf!("%q shorthand is more than one ASCII character", shorthand.clone());
            self.output_write(msg.clone());
            panic!("shorthand too long");
        }
        let c = shorthand[0usize];
        let used = self.shorthands.Has(c);
        if used {
            let fname = self.flags[idx].Name.clone();
            let msg = fmt::Sprintf!(
                "unable to redefine %q shorthand in %q flagset: it's already used for %q flag",
                c as char,
                self.name.clone(),
                fname
            );
            self.output_write(msg);
            panic!("shorthand already used");
        }
        self.shorthands.Set(c, idx);
        idx
    }

    // go: flag.go:871 (AddFlag). Go stores the *same* *Flag pointer; the
    // port clones the Flag into this set's arena (metadata copy — the
    // Value's raw pointer keeps the bound variable shared, see CloneBox).
    pub fn AddFlag(&mut self, flag: &Flag) {
        let cloned = Flag {
            Name: flag.Name.clone(),
            Shorthand: flag.Shorthand.clone(),
            Usage: flag.Usage.clone(),
            Value: flag.Value.CloneBox(),
            DefValue: flag.DefValue.clone(),
            Changed: flag.Changed,
            NoOptDefVal: flag.NoOptDefVal.clone(),
            Deprecated: flag.Deprecated.clone(),
            Hidden: flag.Hidden,
            ShorthandDeprecated: flag.ShorthandDeprecated.clone(),
            Annotations: flag.Annotations.clone(),
        };
        let _ = self.add_flag(alloc::boxed::Box::new(cloned));
    }

    // go: flag.go:911 (AddFlagSet)
    pub fn AddFlagSet(&mut self, new_set: &FlagSet) {
        for (_, &idx) in goish::range!(new_set.ordered_formal) {
            let flag = &new_set.flags[idx];
            if self.Lookup(flag.Name.clone()).is_none() {
                self.AddFlag(flag);
            }
        }
    }

    pub fn PrintDefaults(&self) {
        let usages = self.FlagUsages();
        self.output_write(usages);
    }

    pub fn FlagUsages(&self) -> string {
        self.FlagUsagesWrapped(0)
    }

    pub fn FlagUsagesWrapped(&self, _cols: int) -> string {
        let mut buf = string("");
        self.VisitAll(|flag| {
            if flag.Hidden {
                return;
            }
            let line: string;
            if flag.Shorthand.Len() > 0 && flag.ShorthandDeprecated.Len() == 0 {
                line = fmt::Sprintf!("  -%s, --%s", flag.Shorthand.clone(), flag.Name.clone());
            } else {
                line = fmt::Sprintf!("      --%s", flag.Name.clone());
            }
            let (varname, usage) = unquote_usage(flag);
            let mut line2 = line;
            if varname.Len() > 0 {
                line2 = (line2) + (" ") + (varname);
            }
            if !default_is_zero_value(flag) {
                if flag.Value.Type() == "string" {
                    line2 = (line2) + (fmt::Sprintf!(" (default %q)", flag.DefValue.clone()));
                } else {
                    line2 = (line2) + (fmt::Sprintf!(" (default %s)", flag.DefValue.clone()));
                }
            }
            let full_line = (line2) + ("\t") + (usage);
            buf = (buf.clone()) + (full_line) + ("\n");
        });
        buf
    }

    fn usage_fn(&self) {
        if let Some(ref f) = self.Usage {
            f();
        } else {
            let name = self.name.clone();
            self.output_write(fmt::Sprintf!("Usage of %s:\n", name));
            self.PrintDefaults();
        }
    }

    fn fail(&self, err: error) -> error {
        if self.error_handling != ContinueOnError {
            self.usage_fn();
        }
        err
    }

    fn parse_long_arg<F>(&mut self, s: string, args: &mut alloc::vec::Vec<string>, fn_: &mut F) -> error
    where
        F: FnMut(&mut FlagSet, string, string) -> error,
    {
        // s starts with "--"
        let name_part = s.slice(2, s.Len());
        if name_part.Len() == 0 || name_part[0usize] == b'-' || name_part[0usize] == b'=' {
            return self.fail(errors::Wrap(InvalidSyntaxError {
                specified_flag: s,
            }));
        }
        // split on '='
        let eq_pos = strings::Index(name_part.clone(), "=");
        let (name, value_str, has_value): (string, string, bool) = if eq_pos >= 0 {
            (
                name_part.slice(0, eq_pos),
                name_part.slice(eq_pos + 1, name_part.Len()),
                true,
            )
        } else {
            (name_part, string(""), false)
        };

        let norm = self.normalize_flag_name(name.clone());
        let (fi_ref, exists) = self.formal.GetRef(norm.clone()); let flag_idx = fi_ref.copied().unwrap_or(0);

        if !exists {
            if name == "help" {
                self.usage_fn();
                return ErrHelp.into();
            }
            if self.ParseErrorsAllowlist.UnknownFlags || self.ParseErrorsWhitelist.UnknownFlags {
                if !has_value {
                    strip_unknown_flag_value(args);
                }
                return nil.into();
            }
            return self.fail(errors::Wrap(NotExistError {
                name: name,
                message_type: flagUnknownFlagMessage,
                ..Default::default()
            }));
        }

        let value: string;
        if has_value {
            value = value_str;
        } else if self.flags[flag_idx].NoOptDefVal.Len() > 0 {
            value = self.flags[flag_idx].NoOptDefVal.clone();
        } else if !args.is_empty() {
            value = args.remove(0);
        } else {
            let fname = self.flags[flag_idx].Name.clone();
            return self.fail(errors::Wrap(ValueRequiredError {
                flag_name: fname.clone(),
                specified_name: fname,
                ..Default::default()
            }));
        }

        let flag_name = self.flags[flag_idx].Name.clone();
        let err = fn_(self, flag_name, value);
        if err != nil {
            return self.fail(err);
        }
        nil.into()
    }

    fn parse_single_short_arg<F>(
        &mut self,
        shorthands: string,
        args: &mut alloc::vec::Vec<string>,
        fn_: &mut F,
    ) -> (string, error)
    where
        F: FnMut(&mut FlagSet, string, string) -> error,
    {
        // skip go test shorthands
        if strings::HasPrefix(shorthands.clone(), "test.") {
            return (string(""), nil.into());
        }
        let rest = shorthands.slice(1, shorthands.Len());
        let c = shorthands[0usize];
        let (fi_ref, exists) = self.shorthands.GetRef(c); let flag_idx = fi_ref.copied().unwrap_or(0);

        if !exists {
            if c == b'h' {
                self.usage_fn();
                return (string(""), ErrHelp.into());
            }
            if self.ParseErrorsAllowlist.UnknownFlags || self.ParseErrorsWhitelist.UnknownFlags {
                if shorthands.Len() > 2 && shorthands[1usize] == b'=' {
                    return (string(""), nil.into());
                }
                strip_unknown_flag_value(args);
                return (rest, nil.into());
            }
            return (
                string(""),
                self.fail(errors::Wrap(NotExistError {
                    name: string::from_bytes(&[c]),
                    specified_shorthands: shorthands,
                    message_type: flagUnknownShorthandFlagMessage,
                })),
            );
        }

        let value: string;
        let new_rest: string;
        if shorthands.Len() > 2 && shorthands[1usize] == b'=' {
            // '-f=arg'
            value = shorthands.slice(2, shorthands.Len());
            new_rest = string("");
        } else if self.flags[flag_idx].NoOptDefVal.Len() > 0 {
            value = self.flags[flag_idx].NoOptDefVal.clone();
            new_rest = rest;
        } else if rest.Len() > 0 {
            // '-farg'
            value = rest.clone();
            new_rest = string("");
        } else if !args.is_empty() {
            value = args.remove(0);
            new_rest = string("");
        } else {
            let fname = self.flags[flag_idx].Name.clone();
            let c_str = string::from_bytes(&[c]);
            return (
                string(""),
                self.fail(errors::Wrap(ValueRequiredError {
                    flag_name: fname,
                    specified_name: c_str,
                    specified_shorthands: shorthands,
                })),
            );
        }

        if self.flags[flag_idx].ShorthandDeprecated.Len() > 0 {
            let sh = self.flags[flag_idx].Shorthand.clone();
            let dep = self.flags[flag_idx].ShorthandDeprecated.clone();
            self.output_write(fmt::Sprintf!("Flag shorthand -%s has been deprecated, %s\n", sh, dep));
        }

        let flag_name = self.flags[flag_idx].Name.clone();
        let err = fn_(self, flag_name, value);
        if err != nil {
            return (string(""), self.fail(err));
        }
        (new_rest, nil.into())
    }

    fn parse_args<F>(&mut self, mut args: alloc::vec::Vec<string>, fn_: &mut F) -> error
    where
        F: FnMut(&mut FlagSet, string, string) -> error,
    {
        while !args.is_empty() {
            let s = args.remove(0);
            if s.Len() == 0 || s[0usize] != b'-' || s.Len() == 1 {
                if !self.interspersed {
                    self.args = append!(self.args.clone(), s.clone());
                    for a in &args {
                        self.args = append!(self.args.clone(), a.clone());
                    }
                    return nil.into();
                }
                self.args = append!(self.args.clone(), s);
                continue;
            }
            if s[1usize] == b'-' {
                if s.Len() == 2 {
                    // "--" terminates flags
                    self.args_len_at_dash = self.args.Len();
                    for a in &args {
                        self.args = append!(self.args.clone(), a.clone());
                    }
                    return nil.into();
                }
                let err = self.parse_long_arg(s, &mut args, fn_);
                if err != nil {
                    return err;
                }
            } else {
                let mut shorthands = s.slice(1, s.Len());
                while shorthands.Len() > 0 {
                    let (new_sh, err) = self.parse_single_short_arg(shorthands, &mut args, fn_);
                    if err != nil {
                        return err;
                    }
                    shorthands = new_sh;
                }
            }
        }
        nil.into()
    }

    pub fn Parse(&mut self, arguments: slice<string>) -> error {
        self.parsed = true;
        self.args = slice::<string>::from(nil);
        if arguments.Len() == 0 {
            return nil.into();
        }
        let mut args_vec: alloc::vec::Vec<string> = alloc::vec::Vec::new();
        let mut i = 0usize;
        while i < arguments.Len() as usize {
            args_vec.push(arguments[i].clone());
            i += 1;
        }
        let mut set_fn = |fs: &mut FlagSet, name: string, value: string| -> error {
            fs.Set(name, value)
        };
        let err = self.parse_args(args_vec, &mut set_fn);
        if err != nil {
            match self.error_handling {
                0 /* ContinueOnError */ => return err,
                1 /* ExitOnError */ => {
                    if errors::Is(err.clone(), ErrHelp) {
                        goish::syscall::Exit(0);
                    }
                    self.output_write(fmt::Sprintf!("%v\n", err));
                    goish::syscall::Exit(2);
                }
                _ /* PanicOnError */ => panic!("flag parse error"),
            }
        }
        nil.into()
    }

    // ── Type-specific BoolVar/StringVar etc. ──────────────────────────────

    pub fn BoolVar(&mut self, p: *mut bool, name: string, value: bool, usage: string) {
        self.BoolVarP(p, name, string(""), value, usage);
    }

    pub fn BoolVarP(&mut self, p: *mut bool, name: string, shorthand: string, value: bool, usage: string) {
        let v = alloc::boxed::Box::new(boolValue::new(p, value));
        let idx = self.VarPF(v, name, shorthand, usage);
        self.flags[idx].NoOptDefVal = string("true");
    }

    pub fn BoolP(&mut self, name: string, shorthand: string, value: bool, usage: string) -> *mut bool {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value));
        self.BoolVarP(p, name, shorthand, value, usage);
        p
    }

    pub fn Bool(&mut self, name: string, value: bool, usage: string) -> *mut bool {
        self.BoolP(name, string(""), value, usage)
    }

    pub fn GetBool<S: Into<string>>(&self, name: S) -> (bool, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                false,
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "bool" {
                    return (false, fmt::Errorf!("trying to get bool value of flag of type %s", flag.Value.Type()));
                }
                let s = flag.Value.String();
                let (v, err) = strconv::ParseBool(s);
                if err != nil {
                    return (false, err);
                }
                (v, nil.into())
            }
        }
    }

    pub fn StringVar(&mut self, p: *mut string, name: string, value: string, usage: string) {
        self.StringVarP(p, name, string(""), value, usage);
    }

    pub fn StringVarP(&mut self, p: *mut string, name: string, shorthand: string, value: string, usage: string) {
        let v = alloc::boxed::Box::new(stringValue::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn StringP(&mut self, name: string, shorthand: string, value: string, usage: string) -> *mut string {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value.clone()));
        self.StringVarP(p, name, shorthand, value, usage);
        p
    }

    pub fn String(&mut self, name: string, value: string, usage: string) -> *mut string {
        self.StringP(name, string(""), value, usage)
    }

    pub fn GetString<S: Into<string>>(&self, name: S) -> (string, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                string(""),
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "string" {
                    return (
                        string(""),
                        fmt::Errorf!("trying to get string value of flag of type %s", flag.Value.Type()),
                    );
                }
                (flag.Value.String(), nil.into())
            }
        }
    }

    pub fn IntVar(&mut self, p: *mut int, name: string, value: int, usage: string) {
        self.IntVarP(p, name, string(""), value, usage);
    }

    pub fn IntVarP(&mut self, p: *mut int, name: string, shorthand: string, value: int, usage: string) {
        let v = alloc::boxed::Box::new(intValue::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn IntP(&mut self, name: string, shorthand: string, value: int, usage: string) -> *mut int {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value));
        self.IntVarP(p, name, shorthand, value, usage);
        p
    }

    pub fn Int(&mut self, name: string, value: int, usage: string) -> *mut int {
        self.IntP(name, string(""), value, usage)
    }

    pub fn GetInt<S: Into<string>>(&self, name: S) -> (int, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                0,
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "int" {
                    return (0, fmt::Errorf!("trying to get int value of flag of type %s", flag.Value.Type()));
                }
                let (v, err) = strconv::Atoi(flag.Value.String());
                if err != nil {
                    return (0, err);
                }
                (v, nil.into())
            }
        }
    }

    pub fn Int64Var(&mut self, p: *mut i64, name: string, value: i64, usage: string) {
        self.Int64VarP(p, name, string(""), value, usage);
    }

    pub fn Int64VarP(&mut self, p: *mut i64, name: string, shorthand: string, value: i64, usage: string) {
        let v = alloc::boxed::Box::new(int64Value::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn Int64P(&mut self, name: string, shorthand: string, value: i64, usage: string) -> *mut i64 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value));
        self.Int64VarP(p, name, shorthand, value, usage);
        p
    }

    pub fn Int64(&mut self, name: string, value: i64, usage: string) -> *mut i64 {
        self.Int64P(name, string(""), value, usage)
    }

    pub fn GetInt64<S: Into<string>>(&self, name: S) -> (i64, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                0i64,
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "int64" {
                    return (0i64, fmt::Errorf!("trying to get int64 value of flag of type %s", flag.Value.Type()));
                }
                let (v, err) = strconv::ParseInt(flag.Value.String(), 10, 64);
                if err != nil {
                    return (0i64, err);
                }
                (int64(v), nil.into())
            }
        }
    }

    pub fn Int32Var(&mut self, p: *mut i32, name: string, value: i32, usage: string) {
        self.Int32VarP(p, name, string(""), value, usage);
    }

    pub fn Int32VarP(&mut self, p: *mut i32, name: string, shorthand: string, value: i32, usage: string) {
        let v = alloc::boxed::Box::new(int32Value::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn Int32P(&mut self, name: string, shorthand: string, value: i32, usage: string) -> *mut i32 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value));
        self.Int32VarP(p, name, shorthand, value, usage);
        p
    }

    pub fn Int32(&mut self, name: string, value: i32, usage: string) -> *mut i32 {
        self.Int32P(name, string(""), value, usage)
    }

    pub fn GetInt32<S: Into<string>>(&self, name: S) -> (i32, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                0i32,
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "int32" {
                    return (0i32, fmt::Errorf!("trying to get int32 value of flag of type %s", flag.Value.Type()));
                }
                let (v, err) = strconv::ParseInt(flag.Value.String(), 10, 32);
                if err != nil {
                    return (0i32, err);
                }
                (int32(v), nil.into())
            }
        }
    }

    pub fn UintVar(&mut self, p: *mut uint, name: string, value: uint, usage: string) {
        self.UintVarP(p, name, string(""), value, usage);
    }

    pub fn UintVarP(&mut self, p: *mut uint, name: string, shorthand: string, value: uint, usage: string) {
        let v = alloc::boxed::Box::new(uintValue::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn UintP(&mut self, name: string, shorthand: string, value: uint, usage: string) -> *mut uint {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value));
        self.UintVarP(p, name, shorthand, value, usage);
        p
    }

    pub fn Uint(&mut self, name: string, value: uint, usage: string) -> *mut uint {
        self.UintP(name, string(""), value, usage)
    }

    pub fn GetUint<S: Into<string>>(&self, name: S) -> (uint, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                0u64,
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "uint" {
                    return (0u64, fmt::Errorf!("trying to get uint value of flag of type %s", flag.Value.Type()));
                }
                let (v, err) = strconv::ParseUint(flag.Value.String(), 10, 64);
                if err != nil {
                    return (0u64, err);
                }
                (uint(v), nil.into())
            }
        }
    }

    pub fn Float64Var(&mut self, p: *mut float64, name: string, value: float64, usage: string) {
        self.Float64VarP(p, name, string(""), value, usage);
    }

    pub fn Float64VarP(&mut self, p: *mut float64, name: string, shorthand: string, value: float64, usage: string) {
        let v = alloc::boxed::Box::new(float64Value::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn Float64P(&mut self, name: string, shorthand: string, value: float64, usage: string) -> *mut float64 {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value));
        self.Float64VarP(p, name, shorthand, value, usage);
        p
    }

    pub fn Float64(&mut self, name: string, value: float64, usage: string) -> *mut float64 {
        self.Float64P(name, string(""), value, usage)
    }

    pub fn GetFloat64<S: Into<string>>(&self, name: S) -> (float64, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                0f64,
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "float64" {
                    return (0f64, fmt::Errorf!("trying to get float64 value of flag of type %s", flag.Value.Type()));
                }
                let (v, err) = strconv::ParseFloat(flag.Value.String(), 64);
                if err != nil {
                    return (0f64, err);
                }
                (v, nil.into())
            }
        }
    }

    pub fn DurationVar(&mut self, p: *mut time::Duration, name: string, value: time::Duration, usage: string) {
        self.DurationVarP(p, name, string(""), value, usage);
    }

    pub fn DurationVarP(
        &mut self,
        p: *mut time::Duration,
        name: string,
        shorthand: string,
        value: time::Duration,
        usage: string,
    ) {
        let v = alloc::boxed::Box::new(durationValue::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn DurationP(
        &mut self,
        name: string,
        shorthand: string,
        value: time::Duration,
        usage: string,
    ) -> *mut time::Duration {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value));
        self.DurationVarP(p, name, shorthand, value, usage);
        p
    }

    pub fn Duration(&mut self, name: string, value: time::Duration, usage: string) -> *mut time::Duration {
        self.DurationP(name, string(""), value, usage)
    }

    pub fn GetDuration<S: Into<string>>(&self, name: S) -> (time::Duration, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                time::Duration(0),
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "duration" {
                    return (
                        time::Duration(0),
                        fmt::Errorf!("trying to get duration value of flag of type %s", flag.Value.Type()),
                    );
                }
                let (v, err) = time::ParseDuration(flag.Value.String());
                if err != nil {
                    return (time::Duration(0), err);
                }
                (v, nil.into())
            }
        }
    }

    pub fn CountVar(&mut self, p: *mut int, name: string, usage: string) {
        self.CountVarP(p, name, string(""), usage);
    }

    pub fn CountVarP(&mut self, p: *mut int, name: string, shorthand: string, usage: string) {
        let v = alloc::boxed::Box::new(countValue::new(p));
        let idx = self.VarPF(v, name, shorthand, usage);
        self.flags[idx].NoOptDefVal = string("+1");
    }

    pub fn CountP(&mut self, name: string, shorthand: string, usage: string) -> *mut int {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(0i64));
        self.CountVarP(p, name, shorthand, usage);
        p
    }

    pub fn Count(&mut self, name: string, usage: string) -> *mut int {
        self.CountP(name, string(""), usage)
    }

    pub fn GetCount<S: Into<string>>(&self, name: S) -> (int, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                0,
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "count" {
                    return (0, fmt::Errorf!("trying to get count value of flag of type %s", flag.Value.Type()));
                }
                let (v, err) = strconv::Atoi(flag.Value.String());
                if err != nil {
                    return (0, err);
                }
                (v, nil.into())
            }
        }
    }

    pub fn StringSliceVar(&mut self, p: *mut slice<string>, name: string, value: slice<string>, usage: string) {
        self.StringSliceVarP(p, name, string(""), value, usage);
    }

    pub fn StringSliceVarP(
        &mut self,
        p: *mut slice<string>,
        name: string,
        shorthand: string,
        value: slice<string>,
        usage: string,
    ) {
        let v = alloc::boxed::Box::new(stringSliceValue::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn StringSliceP(
        &mut self,
        name: string,
        shorthand: string,
        value: slice<string>,
        usage: string,
    ) -> *mut slice<string> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(slice::<string>::from(nil)));
        self.StringSliceVarP(p, name, shorthand, value, usage);
        p
    }

    pub fn StringSlice(&mut self, name: string, value: slice<string>, usage: string) -> *mut slice<string> {
        self.StringSliceP(name, string(""), value, usage)
    }

    pub fn GetStringSlice<S: Into<string>>(&self, name: S) -> (slice<string>, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                slice::<string>::from(nil),
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "stringSlice" {
                    return (
                        slice::<string>::from(nil),
                        fmt::Errorf!("trying to get stringSlice value of flag of type %s", flag.Value.Type()),
                    );
                }
                let s = flag.Value.String();
                if s.Len() < 2 {
                    return (slice::<string>::from(nil), nil.into());
                }
                let inner = s.slice(1, s.Len() - 1);
                if inner.Len() == 0 {
                    return (slice::<string>::from(nil), nil.into());
                }
                read_as_csv(inner)
            }
        }
    }

    pub fn StringArrayVar(&mut self, p: *mut slice<string>, name: string, value: slice<string>, usage: string) {
        self.StringArrayVarP(p, name, string(""), value, usage);
    }

    pub fn StringArrayVarP(
        &mut self,
        p: *mut slice<string>,
        name: string,
        shorthand: string,
        value: slice<string>,
        usage: string,
    ) {
        let v = alloc::boxed::Box::new(stringArrayValue::new(p, value));
        self.VarPF(v, name, shorthand, usage);
    }

    pub fn StringArrayP(
        &mut self,
        name: string,
        shorthand: string,
        value: slice<string>,
        usage: string,
    ) -> *mut slice<string> {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(slice::<string>::from(nil)));
        self.StringArrayVarP(p, name, shorthand, value, usage);
        p
    }

    pub fn StringArray(&mut self, name: string, value: slice<string>, usage: string) -> *mut slice<string> {
        self.StringArrayP(name, string(""), value, usage)
    }

    pub fn GetStringArray<S: Into<string>>(&self, name: S) -> (slice<string>, error) {
        let name = name.into();
        match self.Lookup(name.clone()) {
            None => (
                slice::<string>::from(nil),
                errors::Wrap(NotExistError {
                    name,
                    message_type: flagNotDefinedMessage,
                    ..Default::default()
                }),
            ),
            Some(flag) => {
                if flag.Value.Type() != "stringArray" {
                    return (
                        slice::<string>::from(nil),
                        fmt::Errorf!("trying to get stringArray value of flag of type %s", flag.Value.Type()),
                    );
                }
                let s = flag.Value.String();
                if s.Len() < 2 {
                    return (slice::<string>::from(nil), nil.into());
                }
                let inner = s.slice(1, s.Len() - 1);
                if inner.Len() == 0 {
                    return (slice::<string>::from(nil), nil.into());
                }
                read_as_csv(inner)
            }
        }
    }

    /// AddGoFlagSet is a stub — Go's flag package is not portable to Goish.
    pub fn AddGoFlagSet(&mut self) {
        // no-op stub
    }
}

// ── FlagSet constructor ────────────────────────────────────────────────────

pub fn NewFlagSet<S: Into<string>>(name: S, error_handling: ErrorHandling) -> FlagSet {
    FlagSet {
        Usage: None,
        SortFlags: true,
        ParseErrorsAllowlist: ParseErrorsAllowlist::default(),
        ParseErrorsWhitelist: ParseErrorsAllowlist::default(),
        name: name.into(),
        parsed: false,
        flags: alloc::vec::Vec::new(),
        formal: make!(map[string]usize),
        ordered_formal: slice::<usize>::from(nil),
        shorthands: make!(map[byte]usize),
        actual: make!(map[string]usize),
        ordered_actual: slice::<usize>::from(nil),
        args: slice::<string>::from(nil),
        args_len_at_dash: -1,
        error_handling,
        interspersed: true,
        normalize_name_fn: None,
    }
}

// ── CommandLine global ─────────────────────────────────────────────────────

pub static COMMAND_LINE: Lazy<sync::Mutex<FlagSet>> =
    Lazy::new(|| sync::Mutex::new(NewFlagSet("", ExitOnError)));

// ── Helper functions ───────────────────────────────────────────────────────

fn strip_unknown_flag_value(args: &mut alloc::vec::Vec<string>) {
    if args.is_empty() {
        return;
    }
    let first = &args[0];
    if first.Len() > 0 && first[0usize] == b'-' {
        return;
    }
    if args.len() > 1 {
        args.remove(0);
    } else {
        args.clear();
    }
}

fn default_is_zero_value(flag: &Flag) -> bool {
    let __type = flag.Value.Type(); let __type_str: &str = __type.as_ref(); match __type_str {
        "bool" => flag.DefValue == "false" || flag.DefValue.Len() == 0,
        "duration" => flag.DefValue == "0" || flag.DefValue == "0s",
        "int" | "int32" | "int64" | "uint" | "uint32" | "uint64" | "count" | "float64" => {
            flag.DefValue == "0"
        }
        "string" => flag.DefValue.Len() == 0,
        "stringSlice" | "stringArray" => flag.DefValue == "[]",
        _ => {
            flag.DefValue == "false"
                || flag.DefValue == ""
                || flag.DefValue == "0"
        }
    }
}

fn unquote_usage(flag: &Flag) -> (string, string) {
    let usage = flag.Usage.clone();
    let bytes = usage.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let mut j = i + 1;
            while j < bytes.len() {
                if bytes[j] == b'`' {
                    let name = usage.slice(int(i) + 1, int(j));
                    let usage2 = (usage.slice(0, int(i)))
                        + (name.clone())
                        + (usage.slice(int(j) + 1, usage.Len()));
                    return (name, usage2);
                }
                j += 1;
            }
            break;
        }
        i += 1;
    }
    let __ft = flag.Value.Type();
    let __fts: &str = __ft.as_ref();
    let name = match __fts {
        "bool" => string(""),
        "float64" => string("float"),
        "int64" => string("int"),
        "uint64" => string("uint"),
        "stringSlice" => string("strings"),
        _ => flag.Value.Type(),
    };
    (name, usage)
}

// ── CSV helpers ────────────────────────────────────────────────────────────

fn read_as_csv(val: string) -> (slice<string>, error) {
    if val.Len() == 0 {
        return (slice::<string>::from(nil), nil.into());
    }
    let reader = strings::NewReader(val);
    csv::NewReader(reader).Read()
}

fn write_as_csv(vals: slice<string>) -> string {
    if vals.Len() == 0 {
        return string("");
    }
    let mut result = string("");
    let mut i = 0usize;
    while i < vals.Len() as usize {
        let v = vals[i].clone();
        if i > 0 {
            result = (result) + (",");
        }
        if strings::Contains(v.clone(), ",") || strings::Contains(v.clone(), "\"") {
            let escaped = strings::Replace(v.clone(), "\"", "\"\"", -1);
            result = (result) + ("\"") + (escaped) + ("\"");
        } else {
            result = (result) + (v);
        }
        i += 1;
    }
    result
}

// ── Value implementations ──────────────────────────────────────────────────

// boolValue
struct boolValue {
    ptr: *mut bool,
}
unsafe impl Send for boolValue {}
unsafe impl Sync for boolValue {}

impl boolValue {
    fn new(ptr: *mut bool, val: bool) -> Self {
        unsafe { *ptr = val; }
        Self { ptr }
    }
}

impl Value for boolValue {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(boolValue { ptr: self.ptr })
    }
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        if v { string("true") } else { string("false") }
    }
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseBool(s);
        if err != nil {
            return err;
        }
        unsafe { *self.ptr = v; }
        nil.into()
    }
    fn Type(&self) -> string {
        string("bool")
    }
}

// stringValue
struct stringValue {
    ptr: *mut string,
}
unsafe impl Send for stringValue {}
unsafe impl Sync for stringValue {}

impl stringValue {
    fn new(ptr: *mut string, val: string) -> Self {
        unsafe { *ptr = val; }
        Self { ptr }
    }
}

impl Value for stringValue {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(stringValue { ptr: self.ptr })
    }
    fn String(&self) -> string {
        unsafe { (*self.ptr).clone() }
    }
    fn Set_str(&mut self, s: string) -> error {
        unsafe { *self.ptr = s; }
        nil.into()
    }
    fn Type(&self) -> string {
        string("string")
    }
}

// intValue
struct intValue {
    ptr: *mut int,
}
unsafe impl Send for intValue {}
unsafe impl Sync for intValue {}

impl intValue {
    fn new(ptr: *mut int, val: int) -> Self {
        unsafe { *ptr = val; }
        Self { ptr }
    }
}

impl Value for intValue {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(intValue { ptr: self.ptr })
    }
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        strconv::Itoa(v)
    }
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseInt(s, 0, 64);
        if err != nil {
            return err;
        }
        unsafe { *self.ptr = v; }
        nil.into()
    }
    fn Type(&self) -> string {
        string("int")
    }
}

// int64Value
struct int64Value {
    ptr: *mut i64,
}
unsafe impl Send for int64Value {}
unsafe impl Sync for int64Value {}

impl int64Value {
    fn new(ptr: *mut i64, val: i64) -> Self {
        unsafe { *ptr = val; }
        Self { ptr }
    }
}

impl Value for int64Value {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(int64Value { ptr: self.ptr })
    }
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        strconv::FormatInt(int(v), 10)
    }
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseInt(s, 0, 64);
        if err != nil {
            return err;
        }
        unsafe { *self.ptr = int64(v); }
        nil.into()
    }
    fn Type(&self) -> string {
        string("int64")
    }
}

// int32Value
struct int32Value {
    ptr: *mut i32,
}
unsafe impl Send for int32Value {}
unsafe impl Sync for int32Value {}

impl int32Value {
    fn new(ptr: *mut i32, val: i32) -> Self {
        unsafe { *ptr = val; }
        Self { ptr }
    }
}

impl Value for int32Value {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(int32Value { ptr: self.ptr })
    }
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        strconv::FormatInt(int(v), 10)
    }
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseInt(s, 0, 32);
        if err != nil {
            return err;
        }
        unsafe { *self.ptr = int32(v); }
        nil.into()
    }
    fn Type(&self) -> string {
        string("int32")
    }
}

// uintValue
struct uintValue {
    ptr: *mut uint,
}
unsafe impl Send for uintValue {}
unsafe impl Sync for uintValue {}

impl uintValue {
    fn new(ptr: *mut uint, val: uint) -> Self {
        unsafe { *ptr = val; }
        Self { ptr }
    }
}

impl Value for uintValue {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(uintValue { ptr: self.ptr })
    }
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        strconv::FormatUint(v, 10)
    }
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseUint(s, 0, 64);
        if err != nil {
            return err;
        }
        unsafe { *self.ptr = uint(v); }
        nil.into()
    }
    fn Type(&self) -> string {
        string("uint")
    }
}

// float64Value
struct float64Value {
    ptr: *mut float64,
}
unsafe impl Send for float64Value {}
unsafe impl Sync for float64Value {}

impl float64Value {
    fn new(ptr: *mut float64, val: float64) -> Self {
        unsafe { *ptr = val; }
        Self { ptr }
    }
}

impl Value for float64Value {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(float64Value { ptr: self.ptr })
    }
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        strconv::FormatFloat(v, b'g', -1, 64)
    }
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = strconv::ParseFloat(s, 64);
        if err != nil {
            return err;
        }
        unsafe { *self.ptr = v; }
        nil.into()
    }
    fn Type(&self) -> string {
        string("float64")
    }
}

// durationValue
struct durationValue {
    ptr: *mut time::Duration,
}
unsafe impl Send for durationValue {}
unsafe impl Sync for durationValue {}

impl durationValue {
    fn new(ptr: *mut time::Duration, val: time::Duration) -> Self {
        unsafe { *ptr = val; }
        Self { ptr }
    }
}

impl Value for durationValue {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(durationValue { ptr: self.ptr })
    }
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        v.String()
    }
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = time::ParseDuration(s);
        if err != nil {
            return err;
        }
        unsafe { *self.ptr = v; }
        nil.into()
    }
    fn Type(&self) -> string {
        string("duration")
    }
}

// countValue
struct countValue {
    ptr: *mut int,
}
unsafe impl Send for countValue {}
unsafe impl Sync for countValue {}

impl countValue {
    fn new(ptr: *mut int) -> Self {
        unsafe { *ptr = 0; }
        Self { ptr }
    }
}

impl Value for countValue {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(countValue { ptr: self.ptr })
    }
    fn String(&self) -> string {
        let v = unsafe { *self.ptr };
        strconv::Itoa(v)
    }
    fn Set_str(&mut self, s: string) -> error {
        if s == "+1" {
            unsafe { *self.ptr += 1; }
            return nil.into();
        }
        let (v, err) = strconv::ParseInt(s, 0, 0);
        if err != nil {
            return err;
        }
        unsafe { *self.ptr = v; }
        nil.into()
    }
    fn Type(&self) -> string {
        string("count")
    }
}

// stringSliceValue
struct stringSliceValue {
    ptr: *mut slice<string>,
    changed: bool,
}
unsafe impl Send for stringSliceValue {}
unsafe impl Sync for stringSliceValue {}

impl stringSliceValue {
    fn new(ptr: *mut slice<string>, val: slice<string>) -> Self {
        unsafe { *ptr = val; }
        Self { ptr, changed: false }
    }
}

impl Value for stringSliceValue {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(stringSliceValue { ptr: self.ptr, changed: self.changed })
    }
    fn String(&self) -> string {
        let vals = unsafe { (*self.ptr).clone() };
        let csv_str = write_as_csv(vals);
        ("[") + (csv_str) + ("]")
    }
    fn Set_str(&mut self, s: string) -> error {
        let (v, err) = read_as_csv(s);
        if err != nil {
            return err;
        }
        unsafe {
            if !self.changed {
                *self.ptr = v;
            } else {
                *self.ptr = append!((*self.ptr).clone(), v...);
            }
        }
        self.changed = true;
        nil.into()
    }
    fn Type(&self) -> string {
        string("stringSlice")
    }
}

// stringArrayValue
struct stringArrayValue {
    ptr: *mut slice<string>,
    changed: bool,
}
unsafe impl Send for stringArrayValue {}
unsafe impl Sync for stringArrayValue {}

impl stringArrayValue {
    fn new(ptr: *mut slice<string>, val: slice<string>) -> Self {
        unsafe { *ptr = val; }
        Self { ptr, changed: false }
    }
}

impl Value for stringArrayValue {
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(stringArrayValue { ptr: self.ptr, changed: self.changed })
    }
    fn String(&self) -> string {
        let vals = unsafe { (*self.ptr).clone() };
        let csv_str = write_as_csv(vals);
        ("[") + (csv_str) + ("]")
    }
    fn Set_str(&mut self, s: string) -> error {
        unsafe {
            if !self.changed {
                *self.ptr = slice!([]string { s });
                self.changed = true;
            } else {
                *self.ptr = append!((*self.ptr).clone(), s);
            }
        }
        nil.into()
    }
    fn Type(&self) -> string {
        string("stringArray")
    }
}
