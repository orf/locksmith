// Examples:
// UnGrantLock: updated: lock(0xffff9afecbf8) id(13481,16411,0,0,1,1) grantMask(0) req(0,0,0,0,0,0,0)=0 grant(0,0,0,0,0,0,0)=0 wait(0) type(ExclusiveLock)
// LockAcquire: new: lock(0xffff9afc47f8) id(13481,16411,0,0,0,1) grantMask(0) req(0,0,0,0,0,0,0)=0 grant(0,0,0,0,0,0,0)=0 wait(0) type(AccessExclusiveLock)

use regex::{Regex, RegexBuilder};

const PATTERN: &str = r#"^(?<name>[a-zA-Z: ]+): lock\((?<lockid>0[xX][0-9a-fA-F]+)\) id\((?<oid1>\d+),(?<oid2>\d+),(?<oid3>\d+),(?<oid4>\d+),\d+,\d+\) grantMask\(\d+\) req\((?:\d+|,)+\)=\d+ grant\((?:\d+|,)+\)=\d+ wait\(\d+\) type\((?<locktype>[a-zA-Z]+)\)$"#;

pub struct LockParser {
    pattern: Regex,
}

impl LockParser {
    pub fn new() -> Self {
        let pattern = RegexBuilder::new(PATTERN).multi_line(false).dot_matches_new_line(false).build().expect("Failed to compile regex");
        LockParser {
            pattern
        }
    }

    pub fn extract(&self, message: &str) -> Option<Lock> {
        let matches = self.pattern.captures(message);
        if let Some(matches) = matches {
            let name = matches.name("name").unwrap().as_str().to_string();
            let lockid = matches.name("lockid").unwrap().as_str().to_string();
            let oid1 = Oid(matches.name("oid1").unwrap().as_str().parse().unwrap());
            let oid2 = Oid(matches.name("oid2").unwrap().as_str().parse().unwrap());
            let oid3 = Oid(matches.name("oid3").unwrap().as_str().parse().unwrap());
            let oid4 = Oid(matches.name("oid4").unwrap().as_str().parse().unwrap());
            let locktype = matches.name("locktype").unwrap().as_str().parse().unwrap();
            let lockid = u64::from_str_radix(&lockid.trim_start_matches("0x"), 16).unwrap();
            Some(Lock {
                name,
                lockid,
                target: LockTarget {
                    oid1,
                    oid2,
                    oid3,
                    oid4,
                },
                locktype,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, derive_more::Display)]
#[display("{_0}")]
pub struct Oid(u32);

#[derive(Debug, derive_more::Display)]
#[display("({}, {}, {}, {})", oid1, oid2, oid3, oid4)]
pub struct LockTarget {
    oid1: Oid,
    oid2: Oid,
    oid3: Oid,
    oid4: Oid,
}

#[derive(Debug, derive_more::Display)]
#[display("{name} lock={lockid:#01x} target={target} type={locktype}")]
pub struct Lock {
    name: String,
    lockid: u64,
    target: LockTarget,
    locktype: LockType,
}

impl Lock {
    pub fn is_invalid(&self) -> bool {
        self.locktype == LockType::INVALID
    }
}

#[derive(Debug, Eq, PartialEq)]
#[derive(derive_more::FromStr, derive_more::Display)]
enum LockType {
    INVALID,
    AccessShareLock,
    RowShareLock,
    RowExclusiveLock,
    ShareUpdateExclusiveLock,
    ShareLock,
    ShareRowExclusiveLock,
    ExclusiveLock,
    AccessExclusiveLock,
}