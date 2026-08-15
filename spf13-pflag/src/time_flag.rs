// go: file time.go decls: timeValue, newTimeValue, timeValue.Set, timeValue.Type, timeValue.String, FlagSet.GetTime, FlagSet.TimeVar, FlagSet.TimeVarP, TimeVar, TimeVarP, FlagSet.Time, FlagSet.TimeP, Time, TimeP
//
// time.go — a time.Time flag parsed against a caller-supplied list of
// layouts.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 time.go:10-13 timeValue
pub struct timeValue {
    Time: *mut time::Time,
    formats: slice<string>,
}
unsafe impl Send for timeValue {}
unsafe impl Sync for timeValue {}

// go: github.com/spf13/pflag@v1.0.10 time.go:15-21 newTimeValue
pub fn newTimeValue(val: time::Time, p: *mut time::Time, formats: slice<string>) -> timeValue {
    unsafe {
        *p = val;
    }
    return timeValue { Time: p, formats };
}

impl Value for timeValue {
    // go: none — Goish glue; see the Value trait's CloneBox note.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        alloc::boxed::Box::new(timeValue { Time: self.Time, formats: self.formats.clone() })
    }

    // go: none — Goish glue; see the Value trait's __as_time note.
    fn __as_time(&self) -> Option<time::Time> {
        return Some(unsafe { *self.Time });
    }

    // go: github.com/spf13/pflag@v1.0.10 time.go:51-57 timeValue.String
    /// DIVERGENCE, inherited from goish: time::Time's zero is the Unix
    /// epoch, where Go's is year 1. So an unset flag renders "" here
    /// only if it is 1970-01-01, not 0001-01-01.
    fn String(&self) -> string {
        let t = unsafe { *self.Time };
        if t.IsZero() {
            return string("");
        }
        return t.Format(time::RFC3339Nano);
    }

    // go: github.com/spf13/pflag@v1.0.10 time.go:24-44 timeValue.Set
    /// Tries each layout in order, keeping the first that parses; the
    /// error names every accepted layout, as Go's does.
    fn Set_str(&mut self, s: string) -> error {
        let s = strings::TrimSpace(s);
        for i in 0..self.formats.Len() {
            let f = self.formats[i].clone();
            let (v, err) = time::Parse(f, s.clone());
            if err != nil {
                continue;
            }
            unsafe {
                *self.Time = v;
            }
            return nil.into();
        }
        let mut formats_string = string("");
        for i in 0..self.formats.Len() {
            if i > 0 {
                formats_string = formats_string + string(", ");
            }
            formats_string = formats_string + fmt::Sprintf!("`%s`", self.formats[i].clone());
        }
        return fmt::Errorf!("invalid time format `%s` must be one of: %s", s, formats_string);
    }

    // go: github.com/spf13/pflag@v1.0.10 time.go:47-49 timeValue.Type
    fn Type(&self) -> string {
        return string("time");
    }
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 time.go:60-78 FlagSet.GetTime
    /// Go type-asserts `flag.Value.(*timeValue)`; the port asks the
    /// Value for its time through __as_time, which only timeValue
    /// answers. Note this does NOT route through getFlagType — Go's
    /// GetTime does its own lookup and type check.
    pub fn GetTime<S: Into<string>>(&self, name: S) -> (time::Time, error) {
        let name = name.into();
        let flag = match self.Lookup(name.clone()) {
            None => {
                return (
                    time::Time::default(),
                    fmt::Errorf!("flag accessed but not defined: %s", name),
                );
            }
            Some(f) => f,
        };
        if flag.Value.Type() != "time" {
            return (
                time::Time::default(),
                fmt::Errorf!("trying to get %s value of flag of type %s",
                             string("time"), flag.Value.Type()),
            );
        }
        match flag.Value.__as_time() {
            None => (
                time::Time::default(),
                fmt::Errorf!("value %s is not a time", flag.Value.String()),
            ),
            Some(t) => (t, nil.into()),
        }
    }

    // go: github.com/spf13/pflag@v1.0.10 time.go:82-84 FlagSet.TimeVar
    pub fn TimeVar(&mut self, p: *mut time::Time, name: string, value: time::Time, formats: slice<string>, usage: string) {
        self.TimeVarP(p, name, string(""), value, formats, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 time.go:87-89 FlagSet.TimeVarP
    pub fn TimeVarP(&mut self, p: *mut time::Time, name: string, shorthand: string, value: time::Time, formats: slice<string>, usage: string) {
        self.VarP(alloc::boxed::Box::new(newTimeValue(value, p, formats)), name, shorthand, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 time.go:104-106 FlagSet.Time
    pub fn Time(&mut self, name: string, value: time::Time, formats: slice<string>, usage: string) -> *mut time::Time {
        return self.TimeP(name, string(""), value, formats, usage);
    }

    // go: github.com/spf13/pflag@v1.0.10 time.go:109-113 FlagSet.TimeP
    pub fn TimeP(&mut self, name: string, shorthand: string, value: time::Time, formats: slice<string>, usage: string) -> *mut time::Time {
        let p = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(time::Time::default()));
        self.TimeVarP(p, name, shorthand, value, formats, usage);
        return p;
    }
}

// go: github.com/spf13/pflag@v1.0.10 time.go:93-95 TimeVar
pub fn TimeVar(p: *mut time::Time, name: string, value: time::Time, formats: slice<string>, usage: string) {
    COMMAND_LINE.Lock().TimeVarP(p, name, string(""), value, formats, usage);
}

// go: github.com/spf13/pflag@v1.0.10 time.go:98-100 TimeVarP
pub fn TimeVarP(p: *mut time::Time, name: string, shorthand: string, value: time::Time, formats: slice<string>, usage: string) {
    COMMAND_LINE.Lock().TimeVarP(p, name, shorthand, value, formats, usage);
}

// go: github.com/spf13/pflag@v1.0.10 time.go:117-119 Time
pub fn Time(name: string, value: time::Time, formats: slice<string>, usage: string) -> *mut time::Time {
    return COMMAND_LINE.Lock().TimeP(name, string(""), value, formats, usage);
}

// go: github.com/spf13/pflag@v1.0.10 time.go:122-124 TimeP
pub fn TimeP(name: string, shorthand: string, value: time::Time, formats: slice<string>, usage: string) -> *mut time::Time {
    return COMMAND_LINE.Lock().TimeP(name, shorthand, value, formats, usage);
}
