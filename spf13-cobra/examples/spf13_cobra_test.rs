// Smoke tests for the spf13-cobra port: command tree construction, Execute
// dispatch, --help routing, and unknown-command errors.
#![no_std]
#![no_main]
#![allow(non_snake_case)]

extern crate alloc;

use goish::bytes;
use goish::fmt;
use goish::strings;
use goish::sync;
use goish::syscall;
use goish::testing;
use goish::string;
use goish::slice;
use goish::{int32, nil};

use spf13_cobra as cobra;

fn newBuf() -> cobra::SharedBuf {
    alloc::sync::Arc::new(sync::Mutex::new(bytes::Buffer::new()))
}

fn newRoot() -> cobra::Command {
    let mut rootCmd = cobra::Command {
        Use: string("kvlm"),
        Short: string("kvlm is a tiny key-value CLI"),
        ..Default::default()
    };
    let helloCmd = cobra::Command {
        Use: string("hello"),
        Short: string("Say hello"),
        Run: Some(alloc::sync::Arc::new(
            |cmd: &mut cobra::Command, _args: slice<string>| {
                cmd.Print(string("Hello, World!\n"));
            },
        )),
        ..Default::default()
    };
    rootCmd.AddCommand(helloCmd);
    rootCmd
}

fn test_hello(t: &mut testing::T) {
    let mut root = newRoot();
    let out = newBuf();
    root.SetOut(Some(out.clone()));
    root.SetErr(Some(out.clone()));
    root.SetArgs(slice!([]string{"hello"}));
    let err = root.Execute();
    if err != nil {
        t.Fatal(fmt::Sprintf!("Execute: %v", err));
    }
    let got = out.Lock().String();
    if got != "Hello, World!\n" {
        t.Fatal(fmt::Sprintf!("got %q, want %q", got, "Hello, World!\n"));
    }
}

fn test_help_flag(t: &mut testing::T) {
    let mut root = newRoot();
    let out = newBuf();
    root.SetOut(Some(out.clone()));
    root.SetErr(Some(out.clone()));
    root.SetArgs(slice!([]string{"--help"}));
    let err = root.Execute();
    if err != nil {
        t.Fatal(fmt::Sprintf!("Execute --help: %v", err));
    }
    let got = out.Lock().String();
    if !strings::Contains(got.clone(), "Usage:") {
        t.Fatal(fmt::Sprintf!("help output missing Usage section: %q", got));
    }
    if !strings::Contains(got.clone(), "hello") {
        t.Fatal(fmt::Sprintf!("help output missing hello subcommand: %q", got));
    }
    if !strings::Contains(got.clone(), "-h, --help") {
        t.Fatal(fmt::Sprintf!("help output missing help flag usage: %q", got));
    }
}

fn test_unknown_command(t: &mut testing::T) {
    let mut root = newRoot();
    let out = newBuf();
    root.SetOut(Some(out.clone()));
    root.SetErr(Some(out.clone()));
    root.SetArgs(slice!([]string{"nope"}));
    let err = root.Execute();
    if err == nil {
        t.Fatal(string("expected error for unknown command"));
    }
    if !strings::Contains(err.Error(), "unknown command") {
        t.Fatal(fmt::Sprintf!("unexpected error: %v", err));
    }
}

fn test_help_command(t: &mut testing::T) {
    let mut root = newRoot();
    let out = newBuf();
    root.SetOut(Some(out.clone()));
    root.SetErr(Some(out.clone()));
    root.SetArgs(slice!([]string{"help", "hello"}));
    let err = root.Execute();
    if err != nil {
        t.Fatal(fmt::Sprintf!("Execute help hello: %v", err));
    }
    let got = out.Lock().String();
    if !strings::Contains(got.clone(), "Say hello") {
        t.Fatal(fmt::Sprintf!("help hello output missing short description: %q", got));
    }
}

#[goish::main]
fn main() {
    let tests: &[(&str, testing::TestFn)] = &[
        ("TestHello", test_hello),
        ("TestHelpFlag", test_help_flag),
        ("TestUnknownCommand", test_unknown_command),
        ("TestHelpCommand", test_help_command),
    ];
    let code = testing::Main(tests);
    syscall::Exit(int32(code));
}
