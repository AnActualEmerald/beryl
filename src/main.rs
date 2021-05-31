mod beryl;

use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Command, Stdio};
// use clap::{App, Arg, SubCommand};

#[macro_use]
extern crate clap;

///Starts the REPL by default, also has `run` and `examples` subcommands
fn main() {
    let matches = clap_app!(app =>
    (name: "Beryl")
    (version: env!("CARGO_PKG_VERSION"))
    (author: "Emerald <@Emerald#6666>")
    (about: "Runs emerald script programs and other helpful stuff")
    (@arg debug: -d --debug "Display debugging information")

    (@subcommand examples =>
        (about: "Generates some example files")
        (@arg PATH: "Where to generate the files, defaults to current directory"))

    (@arg bin_path: --gem-path "Path to the gem you want to use")
    (@arg PATH: "Path of file to run")
    (@arg ARGS: ... +use_delimiter "Arguments to pass to the script"))
    .get_matches();

    let debug = matches.is_present("debug");

    if let Some(path) = matches.value_of("PATH") {
        let data = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("Couldn't read file {}: {}", path, e);
        });
        let args = if let Some(tmp) = matches.values_of("ARGS") {
            tmp.map(|e| e).collect::<Vec<&str>>()
        } else {
            vec![]
        };

        //check if the environment variable is set
        let command = if let Some(p) = matches.value_of("bin_path") {
            String::from(p)
        } else if let Ok(g) = env::var("BERYLVM_PATH") {
            g
        } else {
            String::from("berylvm")
        };

        let mut cmd = Command::new(command);
        cmd.stdin(Stdio::inherit());
        if debug {
            cmd.arg("-d").arg(&path);
        } else {
            cmd.arg(&path);
        }

        cmd.arg(&args.join(","));

        if debug {
            println!("Trying command {:?}", cmd);
        }
        if let Ok(mut r) = cmd.spawn() {
            let code = loop {
                //wait until the child process exits
                if let Ok(ex) = r.try_wait() {
                    if let None = ex {
                        continue;
                    } else if let Some(stat) = ex {
                        break stat;
                    }
                }
            };
            //log command exit code here
            if debug {
                println!("Gem exited with code {}", code);
            }
        } else {
            //if the other options fail run with the built in version
            if debug {
                println!(
                    "Can't find gem installed on the system, using built in gem version {}",
                    beryl_lib::version()
                );
            }
            beryl_lib::run(data, &args, debug);
        }
        return;
    } else if let Some(sub) = matches.subcommand_matches("examples") {
        let path = if let Some(tmp) = sub.value_of("PATH") {
            PathBuf::from(tmp)
        } else {
            let tmp = env::current_dir().expect("Couldn't get current directory");
            PathBuf::from(format!("{}{}", tmp.display(), "/examples/"))
        };

        create_examples(&path);
        return;
    } else {
        let mut input: String = String::new();
        {
            let stdin = io::stdin();
            for lines in stdin.lock().lines() {
                let line = lines.expect("Couldn't read from stdin");
                input.push_str(&line);
            }
        }
        if input.len() > 0 {
            beryl_lib::run(input, &vec![""], false);
            return;
        }

        let mut b = beryl::Repl::new(debug);
        b.run();
        // repl(debug).expect("REPL encountered an issue: ");
    }
}

///Generates example files in the target directory or one provided by the user
fn create_examples(path: &PathBuf) {
    //big fan of this macro, makes it easy to include files in the binary
    let examples = [
        include_str!("examples/example1.em"),
        include_str!("examples/example2.em"),
        include_str!("examples/example3.em"),
        include_str!("examples/example4.em"),
        // include_str!("examples/example4.em"),
    ];

    println!("Generating example files at {}", path.display());

    //check if the directory exists first, create it if not
    if fs::read_dir(&path).is_err() {
        fs::create_dir_all(&path).unwrap_or_else(|_| {
            println!("Unable to create target directory {}", path.display());
        });
    }

    let mut count = 1;
    for ex in examples.iter() {
        let expath = path.join(format!("example{}.em", count));
        fs::write(&expath, ex).unwrap_or_else(|_| {
            println!("Error generating example file {}", expath.display());
        });
        count += 1;
    }
}
