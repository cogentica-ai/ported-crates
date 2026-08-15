// Test binary for spf13-pflag port
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate alloc;

use goish::{string};
use goish::syscall;
use goish::testing;
use goish::fmt;
use goish::{nil, slice, int32, int};
use goish::time;
use goish::types::{uint, float64, int8, uint16, float32};
use spf13_pflag::{NewFlagSet, ContinueOnError};

#[goish::main]
fn main() {
    let tests: &[(&str, testing::TestFn)] = &[
        ("TestNewFlagSet", test_new_flag_set),
        ("TestBoolFlag", test_bool_flag),
        ("TestStringFlag", test_string_flag),
        ("TestIntFlag", test_int_flag),
        ("TestInt64Flag", test_int64_flag),
        ("TestUintFlag", test_uint_flag),
        ("TestFloat64Flag", test_float64_flag),
        ("TestDurationFlag", test_duration_flag),
        ("TestCountFlag", test_count_flag),
        ("TestChangedHelper", test_changed_helper),
        ("TestParseLongFlag", test_parse_long_flag),
        ("TestParseShortFlag", test_parse_short_flag),
        ("TestParseFlagEquals", test_parse_flag_equals),
        ("TestParseDoubleDash", test_parse_double_dash),
        ("TestVisitAll", test_visit_all),
        ("TestVisitChanged", test_visit_changed),
        ("TestNFlag", test_nflag),
        ("TestArgs", test_args),
        ("TestLookup", test_lookup),
        ("TestInt8Flag", test_int8_flag),
        ("TestUint16Flag", test_uint16_flag),
        ("TestFloat32Flag", test_float32_flag),
        ("TestGetFlagTypeMismatch", test_getflagtype_mismatch),
        ("TestInt64SliceFlag", test_int64_slice_flag),
        ("TestSliceReplaceThenAppend", test_slice_replace_then_append),
        ("TestSliceValueIface", test_slicevalue_iface),
    ];
    let code = testing::Main(tests);
    syscall::Exit(int32(code));
}

fn test_new_flag_set(t: &mut testing::T) {
    let fs = NewFlagSet("test", ContinueOnError);
    if fs.Name_str() != "test" {
        t.Fatal(fmt::Sprintf!("expected name %q, got %q", "test", fs.Name_str()));
    }
    if fs.HasFlags() {
        t.Fatal(fmt::Sprintf!("new FlagSet should have no flags"));
    }
}

fn test_bool_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: bool = false;
    fs.BoolVar(&mut val as *mut bool, string("verbose"), false, string("enable verbose"));

    // default value
    if val {
        t.Fatal(fmt::Sprintf!("expected default false, got true"));
    }

    // parse --verbose
    let args = slice!([]string { string("--verbose") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if !val {
        t.Fatal(fmt::Sprintf!("expected val=true after --verbose, got false"));
    }

    // test GetBool
    let (b, err2) = fs.GetBool("verbose");
    if err2 != nil {
        t.Fatal(fmt::Sprintf!("GetBool error: %v", err2));
    }
    if !b {
        t.Fatal(fmt::Sprintf!("GetBool returned false, want true"));
    }
}

fn test_string_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val = string("default");
    fs.StringVar(&mut val as *mut string, string("name"), string("default"), string("a name"));

    // default
    if val != "default" {
        t.Fatal(fmt::Sprintf!("expected default %q, got %q", "default", val));
    }

    // parse --name=hello
    let args = slice!([]string { string("--name=hello") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if val != "hello" {
        t.Fatal(fmt::Sprintf!("expected %q, got %q", "hello", val));
    }

    // GetString
    let (s, err2) = fs.GetString("name");
    if err2 != nil {
        t.Fatal(fmt::Sprintf!("GetString error: %v", err2));
    }
    if s != "hello" {
        t.Fatal(fmt::Sprintf!("GetString returned %q, want %q", s, "hello"));
    }
}

fn test_int_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: int = 0;
    fs.IntVar(&mut val as *mut int, string("count"), 0, string("a count"));

    let args = slice!([]string { string("--count"), string("42") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if val != 42 {
        t.Fatal(fmt::Sprintf!("expected 42, got %d", val));
    }

    let (n, err2) = fs.GetInt("count");
    if err2 != nil {
        t.Fatal(fmt::Sprintf!("GetInt error: %v", err2));
    }
    if n != 42 {
        t.Fatal(fmt::Sprintf!("GetInt returned %d, want 42", n));
    }
}

fn test_int64_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: i64 = 0;
    fs.Int64Var(&mut val as *mut i64, string("big"), 0i64, string("a big int"));

    let args = slice!([]string { string("--big=1000000000") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if val != 1000000000i64 {
        t.Fatal(fmt::Sprintf!("expected 1000000000, got %d", val));
    }

    let (n, err2) = fs.GetInt64("big");
    if err2 != nil {
        t.Fatal(fmt::Sprintf!("GetInt64 error: %v", err2));
    }
    if n != 1000000000i64 {
        t.Fatal(fmt::Sprintf!("GetInt64 returned %d, want 1000000000", n));
    }
}

fn test_uint_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: uint = 0;
    fs.UintVar(&mut val as *mut uint, string("uval"), 0u64, string("a uint"));

    let args = slice!([]string { string("--uval=99") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if val != 99 {
        t.Fatal(fmt::Sprintf!("expected 99, got %d", val));
    }

    let (u, err2) = fs.GetUint("uval");
    if err2 != nil {
        t.Fatal(fmt::Sprintf!("GetUint error: %v", err2));
    }
    if u != 99 {
        t.Fatal(fmt::Sprintf!("GetUint returned %d, want 99", u));
    }
}

fn test_float64_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: float64 = 0.0;
    fs.Float64Var(&mut val as *mut float64, string("ratio"), 0.0, string("a ratio"));

    let args = slice!([]string { string("--ratio=3.14") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if val < 3.13 || val > 3.15 {
        t.Fatal(fmt::Sprintf!("expected ~3.14, got %v", val));
    }

    let (f, err2) = fs.GetFloat64("ratio");
    if err2 != nil {
        t.Fatal(fmt::Sprintf!("GetFloat64 error: %v", err2));
    }
    if f < 3.13 || f > 3.15 {
        t.Fatal(fmt::Sprintf!("GetFloat64 returned %v, want ~3.14", f));
    }
}

fn test_duration_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val = time::Duration(0);
    fs.DurationVar(&mut val as *mut time::Duration, string("timeout"), time::Duration(0), string("a timeout"));

    let args = slice!([]string { string("--timeout=5s") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    // 5s = 5,000,000,000 ns
    let five_sec = time::Duration(5 * 1_000_000_000);
    if val != five_sec {
        t.Fatal(fmt::Sprintf!("expected 5s duration, got %v", val.String()));
    }

    let (d, err2) = fs.GetDuration("timeout");
    if err2 != nil {
        t.Fatal(fmt::Sprintf!("GetDuration error: %v", err2));
    }
    if d != five_sec {
        t.Fatal(fmt::Sprintf!("GetDuration returned %v, want 5s", d.String()));
    }
}

fn test_count_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let p = fs.CountP(string("verbose"), string("v"), string("verbosity"));

    // parse -v -v --verbose
    let args = slice!([]string {
        string("-v"),
        string("-v"),
        string("--verbose")
    });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    let got = unsafe { *p };
    if got != 3 {
        t.Fatal(fmt::Sprintf!("expected count=3, got %d", got));
    }
}

fn test_changed_helper(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut b: bool = false;
    let mut s = string("");
    fs.BoolVar(&mut b, string("flag1"), false, string(""));
    fs.StringVar(&mut s, string("flag2"), string(""), string(""));

    // nothing set yet
    if fs.Changed("flag1") {
        t.Fatal(fmt::Sprintf!("flag1 should not be Changed before parse"));
    }
    if fs.Changed("flag2") {
        t.Fatal(fmt::Sprintf!("flag2 should not be Changed before parse"));
    }

    let args = slice!([]string { string("--flag1") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }

    if !fs.Changed("flag1") {
        t.Fatal(fmt::Sprintf!("flag1 should be Changed after parse"));
    }
    if fs.Changed("flag2") {
        t.Fatal(fmt::Sprintf!("flag2 should not be Changed (not in args)"));
    }
    // non-existent flag
    if fs.Changed("nonexistent") {
        t.Fatal(fmt::Sprintf!("nonexistent flag should return false for Changed"));
    }
}

fn test_parse_long_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut s = string("");
    fs.StringVar(&mut s, string("output"), string(""), string("output path"));

    // test --output value (space-separated)
    let args = slice!([]string {
        string("--output"),
        string("/tmp/out")
    });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if s != "/tmp/out" {
        t.Fatal(fmt::Sprintf!("expected %q, got %q", "/tmp/out", s));
    }
}

fn test_parse_short_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut s = string("");
    fs.StringVarP(&mut s, string("output"), string("o"), string(""), string("output path"));

    // test -o value
    let args = slice!([]string {
        string("-o"),
        string("result.txt")
    });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if s != "result.txt" {
        t.Fatal(fmt::Sprintf!("expected %q, got %q", "result.txt", s));
    }
}

fn test_parse_flag_equals(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut n: int = 0;
    fs.IntVar(&mut n, string("port"), 0, string("port number"));

    // test --port=8080
    let args = slice!([]string { string("--port=8080") });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if n != 8080 {
        t.Fatal(fmt::Sprintf!("expected 8080, got %d", n));
    }
}

fn test_parse_double_dash(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut s = string("");
    fs.StringVar(&mut s, string("name"), string(""), string(""));

    // -- separates flags from args
    let args = slice!([]string {
        string("--name=before"),
        string("--"),
        string("after1"),
        string("after2")
    });
    let err = fs.Parse(args);
    if err != nil {
        t.Fatal(fmt::Sprintf!("parse error: %v", err));
    }
    if s != "before" {
        t.Fatal(fmt::Sprintf!("expected name=before, got %q", s));
    }
    let remaining = fs.Args();
    if remaining.Len() != 2 {
        t.Fatal(fmt::Sprintf!("expected 2 remaining args, got %d", remaining.Len()));
    }
    if remaining[0usize] != "after1" {
        t.Fatal(fmt::Sprintf!("expected after1, got %q", remaining[0usize]));
    }
}

fn test_visit_all(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut b: bool = false;
    let mut s = string("");
    let mut n: int = 0;
    fs.BoolVar(&mut b, string("alpha"), false, string(""));
    fs.StringVar(&mut s, string("beta"), string(""), string(""));
    fs.IntVar(&mut n, string("gamma"), 0, string(""));

    let mut count = 0i64;
    fs.VisitAll(|_flag| {
        count += 1;
    });
    if count != 3 {
        t.Fatal(fmt::Sprintf!("expected 3 flags in VisitAll, got %d", count));
    }
}

fn test_visit_changed(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut b: bool = false;
    let mut s = string("");
    fs.BoolVar(&mut b, string("flag1"), false, string(""));
    fs.StringVar(&mut s, string("flag2"), string(""), string(""));

    // only flag1 is set
    let args = slice!([]string { string("--flag1") });
    let _ = fs.Parse(args);

    let mut changed_names: alloc::vec::Vec<string> = alloc::vec::Vec::new();
    fs.Visit(|flag| {
        changed_names.push(flag.Name.clone());
    });
    if changed_names.len() != 1 {
        t.Fatal(fmt::Sprintf!("expected 1 changed flag, got %d", int(changed_names.len())));
    }
    if changed_names[0] != "flag1" {
        t.Fatal(fmt::Sprintf!("expected flag1 to be changed, got %q", changed_names[0]));
    }
}

fn test_nflag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut a: bool = false;
    let mut b: bool = false;
    fs.BoolVar(&mut a, string("a"), false, string(""));
    fs.BoolVar(&mut b, string("b"), false, string(""));

    if fs.NFlag() != 0 {
        t.Fatal(fmt::Sprintf!("expected NFlag=0 before parse, got %d", fs.NFlag()));
    }

    let args = slice!([]string { string("--a") });
    let _ = fs.Parse(args);
    if fs.NFlag() != 1 {
        t.Fatal(fmt::Sprintf!("expected NFlag=1, got %d", fs.NFlag()));
    }
}

fn test_args(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut s = string("");
    fs.StringVar(&mut s, string("name"), string(""), string(""));

    let args = slice!([]string {
        string("--name=foo"),
        string("arg1"),
        string("arg2")
    });
    let _ = fs.Parse(args);
    let remaining = fs.Args();
    if remaining.Len() != 2 {
        t.Fatal(fmt::Sprintf!("expected 2 remaining args, got %d", remaining.Len()));
    }
    if remaining[0usize] != "arg1" {
        t.Fatal(fmt::Sprintf!("expected arg1, got %q", remaining[0usize]));
    }
    if remaining[1usize] != "arg2" {
        t.Fatal(fmt::Sprintf!("expected arg2, got %q", remaining[1usize]));
    }
    if fs.NArg() != 2 {
        t.Fatal(fmt::Sprintf!("NArg() returned %d, want 2", fs.NArg()));
    }
    if fs.Arg(0) != "arg1" {
        t.Fatal(fmt::Sprintf!("Arg(0) returned %q, want arg1", fs.Arg(0)));
    }
}

fn test_lookup(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: int = 0;
    fs.IntVar(&mut val, string("port"), 8080, string("port number"));

    let flag = fs.Lookup("port");
    if flag.is_none() {
        t.Fatal(fmt::Sprintf!("Lookup returned None for existing flag"));
    }
    let flag = flag.unwrap();
    if flag.Name != "port" {
        t.Fatal(fmt::Sprintf!("flag name %q, want port", flag.Name));
    }
    if flag.DefValue != "8080" {
        t.Fatal(fmt::Sprintf!("flag DefValue %q, want 8080", flag.DefValue));
    }

    let missing = fs.Lookup("nonexistent");
    if missing.is_some() {
        t.Fatal(fmt::Sprintf!("Lookup should return None for nonexistent flag"));
    }
}

// The per-type scalar family (int8.go and its six structural twins) plus
// the getFlagType spine every GetX routes through.

fn test_int8_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: int8 = 0;
    fs.Int8Var(&mut val as *mut int8, string("level"), 3, string("a level"));
    if val != 3 {
        t.Fatal(fmt::Sprintf!("default not stored: got %d, want 3", val as i64));
    }
    let args = slice!([]string { string("--level"), string("-42") });
    if fs.Parse(args) != nil {
        t.Fatal(fmt::Sprintf!("parse failed"));
    }
    if val != -42 {
        t.Fatal(fmt::Sprintf!("expected -42, got %d", val as i64));
    }
    let (n, err) = fs.GetInt8("level");
    if err != nil {
        t.Fatal(fmt::Sprintf!("GetInt8 error: %v", err));
    }
    if n != -42 {
        t.Fatal(fmt::Sprintf!("GetInt8 = %d, want -42", n as i64));
    }
}

fn test_uint16_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: uint16 = 0;
    fs.Uint16Var(&mut val as *mut uint16, string("port"), 80, string("a port"));
    let args = slice!([]string { string("--port=65535") });
    if fs.Parse(args) != nil {
        t.Fatal(fmt::Sprintf!("parse failed"));
    }
    if val != 65535 {
        t.Fatal(fmt::Sprintf!("expected 65535, got %d", val as i64));
    }
    let (n, err) = fs.GetUint16("port");
    if err != nil || n != 65535 {
        t.Fatal(fmt::Sprintf!("GetUint16 = %d err=%v, want 65535", n as i64, err));
    }
}

fn test_float32_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: float32 = 0.0;
    fs.Float32Var(&mut val as *mut float32, string("ratio"), 1.0, string("a ratio"));
    let args = slice!([]string { string("--ratio"), string("2.5") });
    if fs.Parse(args) != nil {
        t.Fatal(fmt::Sprintf!("parse failed"));
    }
    if val != 2.5 {
        t.Fatal(fmt::Sprintf!("expected 2.5, got %v", val as f64));
    }
    let (n, err) = fs.GetFloat32("ratio");
    if err != nil || n != 2.5 {
        t.Fatal(fmt::Sprintf!("GetFloat32 = %v err=%v, want 2.5", n as f64, err));
    }
}

/// getFlagType's two error arms: an undefined flag, and a real flag whose
/// Value.Type() is not what the caller asked for. Without the second, a
/// GetX could hand back a parse of an unrelated flag's string form.
fn test_getflagtype_mismatch(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut s: goish::gostring::string = string("");
    fs.StringVar(&mut s as *mut goish::gostring::string, string("name"), string("x"), string("a name"));

    let (_, err) = fs.GetInt8("name");
    if err == nil {
        t.Fatal(fmt::Sprintf!("GetInt8 on a string flag must fail"));
    }
    let (_, err2) = fs.GetInt8("nope");
    if err2 == nil {
        t.Fatal(fmt::Sprintf!("GetInt8 on an undefined flag must fail"));
    }
}

// The slice family: CSV parsing, the replace-then-append rule, and the
// public SliceValue interface consumers (cobra/viper) reach through.

fn test_int64_slice_flag(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: goish::goslice::slice<i64> = goish::make!([]i64, 0);
    fs.Int64SliceVar(&mut val as *mut goish::goslice::slice<i64>,
                     string("ids"), goish::make!([]i64, 0), string("ids"));
    let args = slice!([]string { string("--ids"), string("1,2,3") });
    if fs.Parse(args) != nil {
        t.Fatal(fmt::Sprintf!("parse failed"));
    }
    if val.Len() != 3 || val[0] != 1 || val[2] != 3 {
        t.Fatal(fmt::Sprintf!("got len %d, want [1,2,3]", val.Len() as i64));
    }
    let (got, err) = fs.GetInt64Slice("ids");
    if err != nil || got.Len() != 3 || got[1] != 2 {
        t.Fatal(fmt::Sprintf!("GetInt64Slice len=%d err=%v", got.Len() as i64, err));
    }
}

/// Go's rule: the FIRST --flag replaces the default, every later one
/// appends. Without the `changed` latch, `--ids=1 --ids=2` would mean
/// [2] rather than [1,2] — a silent difference, so it gets a tripwire.
fn test_slice_replace_then_append(t: &mut testing::T) {
    let mut fs = NewFlagSet("test", ContinueOnError);
    let mut val: goish::goslice::slice<i64> = goish::make!([]i64, 0);
    fs.Int64SliceVar(&mut val as *mut goish::goslice::slice<i64>,
                     string("ids"), slice!([]i64 { 99i64 }), string("ids"));
    if val.Len() != 1 || val[0] != 99 {
        t.Fatal(fmt::Sprintf!("default not stored"));
    }
    let args = slice!([]string { string("--ids=1"), string("--ids=2") });
    if fs.Parse(args) != nil {
        t.Fatal(fmt::Sprintf!("parse failed"));
    }
    if val.Len() != 2 || val[0] != 1 || val[1] != 2 {
        t.Fatal(fmt::Sprintf!("want [1,2] (default replaced then appended), got len %d", val.Len() as i64));
    }
}

fn test_slicevalue_iface(t: &mut testing::T) {
    use spf13_pflag::SliceValue;
    let mut backing: goish::goslice::slice<i64> = goish::make!([]i64, 0);
    let mut v = spf13_pflag::newInt64SliceValue(
        slice!([]i64 { 7i64 }), &mut backing as *mut goish::goslice::slice<i64>);
    if v.Append(string("8")) != nil {
        t.Fatal(fmt::Sprintf!("Append failed"));
    }
    if backing.Len() != 2 || backing[1] != 8 {
        t.Fatal(fmt::Sprintf!("Append did not extend the list"));
    }
    if v.Replace(slice!([]string { string("5"), string("6") })) != nil {
        t.Fatal(fmt::Sprintf!("Replace failed"));
    }
    if backing.Len() != 2 || backing[0] != 5 {
        t.Fatal(fmt::Sprintf!("Replace did not overwrite"));
    }
    let got = v.GetSlice();
    if got.Len() != 2 || got[0] != "5" || got[1] != "6" {
        t.Fatal(fmt::Sprintf!("GetSlice = %s,%s", got[0].clone(), got[1].clone()));
    }
    if v.Append(string("notanumber")) == nil {
        t.Fatal(fmt::Sprintf!("Append must reject a non-numeric value"));
    }
}
