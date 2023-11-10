use rand::Rng;
use std::{fmt, str};

pub trait CapitalizationEngine {
    fn should_capitalize(&mut self, index: usize, character: char) -> bool;
}

struct AlternatingCapitalizationEngine {
    pub next_is_capital: bool,
    pub skip_whitespace: bool,
}

impl CapitalizationEngine for AlternatingCapitalizationEngine {
    fn should_capitalize(&mut self, _index: usize, character: char) -> bool {
        let ret = self.next_is_capital;
        if !(self.skip_whitespace && character.is_whitespace()) {
            self.next_is_capital = !self.next_is_capital;
        }
        ret
    }
}

struct RandomCapitalizationEngine {
    rng: rand::rngs::ThreadRng,
}

impl RandomCapitalizationEngine {
    pub fn new() -> RandomCapitalizationEngine {
        RandomCapitalizationEngine {
            rng: rand::thread_rng(),
        }
    }
}

impl CapitalizationEngine for RandomCapitalizationEngine {
    fn should_capitalize(&mut self, _index: usize, _character: char) -> bool {
        self.rng.gen_bool(0.5)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum CapitalizationStrategy {
    AlternatingInitialUppercase,
    AlternatingInitialLowercase,
    AlternatingInitialUppercaseSkipWhitespace,
    AlternatingInitialLowercaseSkipWhitespace,
    Randomly,
}

impl CapitalizationStrategy {
    /// Create a `CapitalizationEngine` based on this strategy description.
    pub fn create_engine(&self) -> Box<dyn CapitalizationEngine> {
        match self {
            Self::AlternatingInitialUppercase => Box::new(AlternatingCapitalizationEngine {
                next_is_capital: true,
                skip_whitespace: false,
            }),
            Self::AlternatingInitialLowercase => Box::new(AlternatingCapitalizationEngine {
                next_is_capital: false,
                skip_whitespace: false,
            }),
            Self::AlternatingInitialUppercaseSkipWhitespace => {
                Box::new(AlternatingCapitalizationEngine {
                    next_is_capital: true,
                    skip_whitespace: true,
                })
            }
            Self::AlternatingInitialLowercaseSkipWhitespace => {
                Box::new(AlternatingCapitalizationEngine {
                    next_is_capital: false,
                    skip_whitespace: true,
                })
            }
            Self::Randomly => Box::new(RandomCapitalizationEngine::new()),
        }
    }
}

impl Default for CapitalizationStrategy {
    fn default() -> Self {
        Self::AlternatingInitialUppercase
    }
}

impl fmt::Display for CapitalizationStrategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CapitalizationStrategy::*;

        let s = match self {
            AlternatingInitialLowercase => "lIkE ThIs",
            AlternatingInitialUppercase => "LiKe tHiS",
            AlternatingInitialLowercaseSkipWhitespace => "lIkE tHiS",
            AlternatingInitialUppercaseSkipWhitespace => "LiKe ThIs",
            Randomly => "RAnDOmlY",
        };

        write!(f, "{}", s)
    }
}

impl str::FromStr for CapitalizationStrategy {
    type Err = String;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        match input {
            "lIkE ThIs" => Ok(Self::AlternatingInitialLowercase),
            "LiKe tHiS" => Ok(Self::AlternatingInitialUppercase),
            "lIkE tHiS" => Ok(Self::AlternatingInitialLowercaseSkipWhitespace),
            "LiKe ThIs" => Ok(Self::AlternatingInitialUppercaseSkipWhitespace),
            x if x.to_lowercase().matches("randomly").count() == 1 => Ok(Self::Randomly),
            _ => Err(format!("Unknown capitalization \"{}\"", input)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalization_strategy_from_str() {
        use CapitalizationStrategy::*;

        assert_eq!(AlternatingInitialLowercase, "lIkE ThIs".parse().unwrap());
        assert_eq!(AlternatingInitialUppercase, "LiKe tHiS".parse().unwrap());
        assert_eq!(
            AlternatingInitialLowercaseSkipWhitespace,
            "lIkE tHiS".parse().unwrap()
        );
        assert_eq!(
            AlternatingInitialUppercaseSkipWhitespace,
            "LiKe ThIs".parse().unwrap()
        );
        assert_eq!(Randomly, "randomly".parse().unwrap());
    }

    fn capitalize_with(style: CapitalizationStrategy, src: &str) -> String {
        let mut engine = style.create_engine();

        let mut out = String::new();

        for (idx, c) in src.chars().enumerate() {
            if engine.should_capitalize(idx, c) {
                out.push(c.to_ascii_uppercase())
            } else {
                out.push(c.to_ascii_lowercase())
            }
        }

        out
    }

    #[test]
    fn alternating_initial_uppercase() {
        let strategy = CapitalizationStrategy::AlternatingInitialUppercase;

        assert_eq!(capitalize_with(strategy, "taco truck"), "TaCo tRuCk");
    }

    #[test]
    fn alternating_initial_lowercase() {
        let strategy = CapitalizationStrategy::AlternatingInitialLowercase;

        assert_eq!(capitalize_with(strategy, "taco truck"), "tAcO TrUcK");
    }

    #[test]
    fn alternating_initial_uppercase_skip_whitespace() {
        let strategy = CapitalizationStrategy::AlternatingInitialUppercaseSkipWhitespace;

        assert_eq!(capitalize_with(strategy, "taco truck"), "TaCo TrUcK");
    }

    #[test]
    fn alternating_initial_lowercase_skip_whitespace() {
        let strategy = CapitalizationStrategy::AlternatingInitialLowercaseSkipWhitespace;

        assert_eq!(capitalize_with(strategy, "taco truck"), "tAcO tRuCk");
    }
}
