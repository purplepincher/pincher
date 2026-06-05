//! Hard-coded blocked security patterns (non-negotiable from pinch5 spec)

use once_cell::sync::Lazy;
use regex::Regex;

/// Hard-coded blocked dangerous commands
pub static BLOCKED_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"^rm -rf /$",).unwrap(),
        Regex::new(r"^rm -rf /\*$",).unwrap(),
        Regex::new(r"^rm -r /$",).unwrap(),
        Regex::new(r"^dd if=/dev/zero$",).unwrap(),
        Regex::new(r"^dd if=/dev/random$",).unwrap(),
        Regex::new(r"^:\(\):{ :|:& };:",).unwrap(), // Fork bomb
        Regex::new(r"^mkfs",).unwrap(),
        Regex::new(r"> /dev/sda$",).unwrap(),
        Regex::new(r"^chmod -R 777 /$",).unwrap(),
        Regex::new(r"^chown -R$",).unwrap(),
        Regex::new(r"^shutdown$",).unwrap(),
        Regex::new(r"^reboot$",).unwrap(),
        Regex::new(r"^halt$",).unwrap(),
        Regex::new(r"^init 0$",).unwrap(),
        Regex::new(r"^init 6$",).unwrap(),
    ]
});

/// Whitelisted executable binaries only
pub static EXECUTABLE_WHITELIST: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "mkdir", "cp", "mv", "ls", "cat", "echo", "touch", "rm",
        "find", "grep", "head", "tail", "wc", "sort", "uniq",
    ]
});

/// Blocked sensitive file paths
pub static SENSITIVE_PATHS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"^/etc/shadow$",).unwrap(),
        Regex::new(r"^/etc/ssh$",).unwrap(),
        Regex::new(r"^/root/\.ssh$",).unwrap(),
        Regex::new(r"^/proc/self/environ$",).unwrap(),
        Regex::new(r"^/etc/gshadow$",).unwrap(),
        Regex::new(r"^/etc/passwd-$",).unwrap(),
        Regex::new(r"^/etc/shadow-$",).unwrap(),
    ]
});

/// Whitelisted safe environment variables
pub static SAFE_ENV_VARS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "HOME", "USER", "SHELL", "LANG", "PATH", "TERM", "EDITOR",
        "PWD", "OLDPWD", "HOSTNAME", "LOGNAME", "COLORTERM",
    ]
});
