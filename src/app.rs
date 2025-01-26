//! Application logic for the CS1675 network APIs project.

use serde::{Deserialize, Serialize};
use std::{
    num::{NonZeroU64, ParseIntError},
    time::Duration,
};

/// Describes a type and amount of busy-work to do.
///
/// Implements [`FromStr`]. String format is `type:amount` where amount is a u64. Options are:
/// - `immediate|imm`
/// - `poisson:[amount]` (amount must be nonzero)
/// - `[const|busytime|bt|busywork|bw]:[amount]`
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Work {
    Immediate,
    Const(u64),
    Payload,
    Poisson(NonZeroU64),

    // what is this?
    BusyTimeConst(u64),
    BusyWorkConst(u64),
}

fn gen_poisson_duration(amt: NonZeroU64) -> Duration {
    use rand_distr::Distribution;

    let mut rng = rand::thread_rng();
    let pois = rand_distr::Poisson::new(amt.get() as f64).unwrap();
    Duration::from_micros(pois.sample(&mut rng) as u64)
}

impl Work {
    /// Perform the busy work.
    ///
    /// Uses blocking calls for non-busy variants ([`Self::Const`] and [`Self::Poisson`]).
    pub fn perform(self) -> Option<Vec<u8>> {
        match self {
            Self::Immediate => None,
            Self::Const(amt) => {
                let now = minstant::Instant::now();
                let amt = Duration::from_micros(amt);
                while now.elapsed() < amt {}
                None
            }
            Self::Poisson(amt) => {
                let amt = gen_poisson_duration(amt);
                let now = minstant::Instant::now();
                while now.elapsed() < amt {}
                None
            }
            Self::Payload => {
                use rand::seq::SliceRandom;
                let x = [64usize, 256, 512, 1024];
                let mut rng = rand::thread_rng();
                Some(vec![0u8; *x.choose(&mut rng).unwrap()])
            }

            Self::BusyTimeConst(amt) => {
                let completion_time = minstant::Instant::now() + Duration::from_micros(amt);
                while minstant::Instant::now() < completion_time {
                    // spin
                }
                None
            }
            Self::BusyWorkConst(amt) => {
                // from shenango:
                // https://github.com/shenango/shenango/blob/master/apps/synthetic/src/fakework.rs#L54
                let k = 2350845.545;
                for i in 0..amt {
                    std::hint::black_box(f64::sqrt(k * i as f64));
                }
                None
            }
        }
    }
}

impl std::str::FromStr for Work {
    type Err = WorkParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sp: Vec<_> = s.split(':').collect();
        match &sp[..] {
            [variant] if *variant == "immediate" || *variant == "imm" => Ok(Work::Immediate),
            [variant] if *variant == "payload" => Ok(Work::Payload),
            [variant, amt] if *variant == "const" => Ok(Work::Const(amt.parse()?)),
            [variant, amt] if *variant == "poisson" => match amt.parse().map(NonZeroU64::new) {
                Ok(Some(x)) => Ok(Work::Poisson(x)),
                Ok(None) => Err(WorkParseErr::ZeroPoissonValue),
                Err(e) => Err(WorkParseErr::U64Parse(e)),
            },
            [variant, amt] if *variant == "busytime" || *variant == "bt" => {
                Ok(Work::BusyTimeConst(amt.parse()?))
            }
            [variant, amt] if *variant == "busywork" || *variant == "bw" => {
                Ok(Work::BusyWorkConst(amt.parse()?))
            }
            _ => Err(WorkParseErr::UnknownFmt(s.to_owned())),
        }
    }
}

impl std::fmt::Display for Work {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Work::Immediate => write!(f, "imm"),
            Work::Const(amt) => write!(f, "const:{}", amt),
            Work::Poisson(amt) => write!(f, "poisson:{}", amt),
            Work::Payload => write!(f, "payload"),
            Work::BusyTimeConst(amt) => write!(f, "busytime:{}", amt),
            Work::BusyWorkConst(amt) => write!(f, "busywork:{}", amt),
        }
    }
}

/// Things that can go wrong when parsing a [`Work`].
#[derive(Debug)]
pub enum WorkParseErr {
    /// The `type:amount` format wasn't followed.
    UnknownFmt(String),
    /// Specified a Poisson distribution with lambda = 0.
    ZeroPoissonValue,
    /// Followed `type:amount`, but `amount` wasn't a `u64`.
    U64Parse(ParseIntError),
}

impl From<ParseIntError> for WorkParseErr {
    fn from(value: ParseIntError) -> Self {
        Self::U64Parse(value)
    }
}

impl std::fmt::Display for WorkParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFmt(s) => {
                write!(f, "Unknown work format specification {}. Format is [immedate|const|poisson|busytime|busywork]:[amount].", s)
            }
            Self::ZeroPoissonValue => {
                write!(f, "Poisson-distributed work amount must be nonzero.")
            }
            Self::U64Parse(n) => {
                write!(f, "Could not parse work amount {} as u64.", n)
            }
        }
    }
}

impl std::error::Error for WorkParseErr {}

#[cfg(test)]
mod t {
    use super::{Work, WorkParseErr};

    #[test]
    fn parse_work_immediate() {
        assert!(matches!(
            "immediate".parse().expect("parse immediate"),
            Work::Immediate
        ));
        assert!(matches!(
            "imm".parse().expect("parse immediate"),
            Work::Immediate
        ));

        assert!(matches!(
            "imm:2".parse::<Work>(),
            Err(WorkParseErr::UnknownFmt(_))
        ));

        assert!(matches!(
            "imm:foo".parse::<Work>(),
            Err(WorkParseErr::UnknownFmt(_))
        ));

        assert!(matches!(
            "foo".parse::<Work>(),
            Err(WorkParseErr::UnknownFmt(_))
        ));
    }

    #[test]
    fn parse_work_const() {
        assert!(matches!(
            "const:2".parse().expect("parse const"),
            Work::Const(2)
        ));

        assert!(matches!(
            "const".parse::<Work>(),
            Err(WorkParseErr::UnknownFmt(_))
        ));

        assert!(matches!(
            "const:foo".parse::<Work>(),
            Err(WorkParseErr::U64Parse(_))
        ));
    }

    #[test]
    fn parse_work_poisson() {
        assert!(matches!(
            "poisson:2".parse().expect("parse poisson"),
            Work::Poisson(x) if x.get() == 2
        ));

        assert!(matches!(
            "poisson:0".parse::<Work>(),
            Err(WorkParseErr::ZeroPoissonValue)
        ));

        assert!(matches!(
            "poisson:foo".parse::<Work>(),
            Err(WorkParseErr::U64Parse(_))
        ));
    }

    #[test]
    fn parse_work_busytime() {
        assert!(matches!(
            "bt:2".parse().expect("parse BusyTimeConst"),
            Work::BusyTimeConst(2)
        ));

        assert!(matches!(
            "busytime:2".parse().expect("parse BusyTimeConst"),
            Work::BusyTimeConst(2)
        ));

        assert!(matches!(
            "busytime:foo".parse::<Work>(),
            Err(WorkParseErr::U64Parse(_))
        ));
    }

    #[test]
    fn parse_work_busywork() {
        assert!(matches!(
            "bw:2".parse().expect("parse BusyWorkConst"),
            Work::BusyWorkConst(2)
        ));

        assert!(matches!(
            "busywork:2".parse().expect("parse BusyWorkConst"),
            Work::BusyWorkConst(2)
        ));

        assert!(matches!(
            "busywork:foo".parse::<Work>(),
            Err(WorkParseErr::U64Parse(_))
        ));
    }
}
