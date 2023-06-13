// This is the entry point of your Rust library.
// When adding new code to your project, note that only items used
// here will be transformed to their Dart equivalents.

// A plain enum without any fields. This is similar to Dart- or C-style enums.
// flutter_rust_bridge is capable of generating code for enums with fields
// (@freezed classes in Dart and tagged unions in C).
pub enum Platform {
    Unknown,
    Android,
    Ios,
    Windows,
    Unix,
    MacIntel,
    MacApple,
    Wasm,
}

// A function definition in Rust. Similar to Dart, the return type must always be named
// and is never inferred.
pub fn platform() -> Platform {
    // This is a macro, a special expression that expands into code. In Rust, all macros
    // end with an exclamation mark and can be invoked with all kinds of brackets (parentheses,
    // brackets and curly braces). However, certain conventions exist, for example the
    // vector macro is almost always invoked as vec![..].
    //
    // The cfg!() macro returns a boolean value based on the current compiler configuration.
    // When attached to expressions (#[cfg(..)] form), they show or hide the expression at compile time.
    // Here, however, they evaluate to runtime values, which may or may not be optimized out
    // by the compiler. A variety of configurations are demonstrated here which cover most of
    // the modern oeprating systems. Try running the Flutter application on different machines
    // and see if it matches your expected OS.
    //
    // Furthermore, in Rust, the last expression in a function is the return value and does
    // not have the trailing semicolon. This entire if-else chain forms a single expression.
    if cfg!(windows) {
        Platform::Windows
    } else if cfg!(target_os = "android") {
        Platform::Android
    } else if cfg!(target_os = "ios") {
        Platform::Ios
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Platform::MacApple
    } else if cfg!(target_os = "macos") {
        Platform::MacIntel
    } else if cfg!(target_family = "wasm") {
        Platform::Wasm
    } else if cfg!(unix) {
        Platform::Unix
    } else {
        Platform::Unknown
    }
}

// The convention for Rust identifiers is the snake_case,
// and they are automatically converted to camelCase on the Dart side.
pub fn rust_release_mode() -> bool {
    cfg!(not(debug_assertions))
}

use anyhow::{anyhow, Result};
use std::process::Command;
use std::fs;

trait LsRootMethod {
    fn execute(&self) -> Result<Vec<String>>;
}

struct PkexecLsMethod;

struct SudoLsMethod {
    password: String,
}

impl LsRootMethod for PkexecLsMethod {
    fn execute(&self) -> Result<Vec<String>> {
        // run pkexec ls -la /root
        match Command::new("pkexec")
            .arg("ls")
            .arg("-la")
            .arg("/root")
            .output()
        {
            Ok(output) => {
                // check if the command was successful
                if output.status.success() {
                    // convert the output to a string and return it
                    let output_str = String::from_utf8(output.stdout)?;
                    return Ok(output_str.lines().map(String::from).collect());
                }
                Err(anyhow!("Permission Denied"))
            }
            Err(_) => Err(anyhow!("Failed to elevate privileges with pkexec method.")),
        }
    }
}

impl LsRootMethod for SudoLsMethod {
    fn execute(&self) -> Result<Vec<String>> {
        // run echo $password | sudo -S ls -la /root and save the output to a file
        let password = &self.password;
        let echo_cmd = format!("echo {}", password);
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "{} | sudo -S ls -la /root > /tmp/result.txt",
                echo_cmd
            ))
            .output()
            .expect("Failed to elevate privileges with sudo.");

        // check if the command was successful and return the output
        if output.status.success() {
            let output = fs::read_to_string("/tmp/result.txt").expect("Failed to read result file");

            return Ok(output.lines().map(String::from).collect());
        }
        Err(anyhow!("Password required"))
    }
}

// with pollkit method
pub fn ls_with_polkit() -> Result<Vec<String>> {
    // create a vector of methods
    let methods: Vec<Box<dyn LsRootMethod>> = vec![Box::new(PkexecLsMethod)];

    // try to execute each method and return the result if successful
    for method in methods {
        match method.execute() {
            Ok(result) => return Ok(result),
            Err(_) => continue,
        };
    }

    Err(anyhow!("Failed to elevate privileges with polkit."))
}

// with sudo and password method
pub fn ls_with_sudo(password: String) -> Result<Vec<String>> {
    // create a vector of methods and add the sudo method
    let method = SudoLsMethod { password };
    method
        .execute()
        .map_err(|_| anyhow!("Failed to elevate privileges with sudo."))
}
