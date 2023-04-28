// Copyright 2023 Eason Qin <eason@ezntek.com>.
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
pub mod cli_parser;

#[allow(unused_imports)] // XXX: we're gonna need it for the macros...
use colored::Colorize;
use smbuilder::prelude::MakeoptsType;

#[cfg(test)]
mod tests {}

// nice macros for logging

#[macro_export]
macro_rules! log_info {
    ( $text:expr ) => {
        println!("{}{}", "Info: ".bold().cyan(), $text)
    };
}

#[macro_export]
macro_rules! log_warn {
    ( $text:expr ) => {
        println!("{}{}", "Info: ".bold().yellow(), $text)
    };
}

#[macro_export]
macro_rules! log_err {
    ( $text:expr ) => {
        println!("{}{}", "Info: ".bold.red(), $text)
    };
}

pub fn get_toml_makeopts_from_string<S>(string: S) -> String
where
    S: AsRef<str>
     + ToString
{
    let s = string.to_string();
    let makeopt_expressions = s.split(" ").collect::<Vec<&str>>();
    let mut toml_string = String::from("[");
    
    for expr in makeopt_expressions {
        let makeopt = expr.split("=").collect::<Vec<&str>>();
        toml_string.push_str(format!("{{opt={},arg={}}},", makeopt[0], makeopt[1]).as_str());
    };
    toml_string.push_str("]");
    toml_string
}