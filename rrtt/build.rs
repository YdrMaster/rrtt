//! See [cargo reference](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-cfgkeyvalue) for `rustc-cfg`.

fn main() {
    use std::env;

    const PRIORITY_MAX: &str = "PRIORITY_MAX";
    const PRIORITY_MAX_DEFAULT: usize = 8;
    let val = match env::var(PRIORITY_MAX) {
        Ok(s) => match s.parse::<usize>() {
            Ok(val) => val,
            Err(_) => panic!("failed to parse env {PRIORITY_MAX}={s:?} for not a number."),
        },
        Err(env::VarError::NotPresent) => {
            println!("cargo:rustc-env={PRIORITY_MAX}={PRIORITY_MAX_DEFAULT}");
            PRIORITY_MAX_DEFAULT
        }
        Err(env::VarError::NotUnicode(_)) => {
            panic!("failed to parse env {PRIORITY_MAX} for not unicode.");
        }
    };
    println!("cargo:rerun-if-env-changed={PRIORITY_MAX}");
    println!(
        "cargo:rustc-cfg={}",
        if val > 32 {
            "large_priority"
        } else {
            "small_priority"
        }
    );
}
