#![allow(dead_code)]

use crate::repl::ReplContext;

// https://github.com/shift-eleven/Eolina/blob/main/Eolina/EolinaParser.cs
mod exec;
mod helper;
mod parse;
mod repl;

fn main() {
    // in, copy, split, filter upper, out, split, filter lower, out
    // <*[^][_]~>
    // orders a string by case upper first

    let mut inter = ReplContext::new();
    inter.run();
}
