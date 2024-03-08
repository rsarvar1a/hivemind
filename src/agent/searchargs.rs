use std::time::Duration;

use regex::Regex;

use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
/// Options with which the user can control an Evaluator's search.
pub enum SearchArgs
{
    Time(Duration),
    Depth(Depth),
}

impl SearchArgs
{
    /// Determines the hard depth limit.
    pub fn depth(&self) -> Depth
    {
        match self
        {
            | Self::Depth(d) => *d,
            | Self::Time(_) => Depth::MAX,
        }
    }

    /// Tries to parse args into a set of search options.
    pub fn parse(args: &[&str]) -> Result<SearchArgs>
    {
        let base = Error::for_parse::<Self>(args.join(" "));

        if args.len() < 2
        {
            let err = Error::new(
                Kind::ParseError,
                "Search options require a mode (time or depth) and a corresponding value.".into(),
            );
            return Err(err.chain(base));
        }

        match args[0]
        {
            | "time" =>
            {
                let parse = (|| {
                    let re = Regex::new(r"^(?<h>[0-9]{2,3}):(?<m>[0-9]{2}):(?<s>[0-9]{2})$").unwrap();
                    let Some(caps) = re.captures(args[1])
                    else
                    {
                        return Err(Error::new(Kind::InvalidTime, "Expected duration in the form of hh:mm:ss".into()));
                    };

                    let hrs_str = caps.name("h").map(|m| m.as_str()).unwrap();
                    let Ok(hrs) = hrs_str.parse::<u64>()
                    else
                    {
                        return Err(Error::new(Kind::InvalidTime, format!("Invalid number of hours '{}'.", hrs_str)));
                    };

                    let mins_str = caps.name("m").map(|m| m.as_str()).unwrap();
                    let Ok(mins) = mins_str.parse::<u64>()
                    else
                    {
                        return Err(Error::new(Kind::InvalidTime, format!("Invalid number of minutes '{}'.", mins_str)));
                    };

                    let secs_str = caps.name("s").map(|m| m.as_str()).unwrap();
                    let Ok(secs) = secs_str.parse::<u64>()
                    else
                    {
                        return Err(Error::new(Kind::InvalidTime, format!("Invalid number of seconds '{}'.", secs_str)));
                    };

                    Ok((hrs, mins, secs))
                })();

                let Ok((hrs, mins, secs)) = parse
                else
                {
                    let parse_err = parse.err().unwrap();
                    let err = Error::for_parse::<Duration>(args[1].to_owned());
                    return Err(parse_err.chain(err).chain(base));
                };

                let seconds: u64 = secs + 60 * mins + 3600 * hrs;
                let time = Duration::from_secs(seconds);
                Ok(SearchArgs::Time(time))
            }
            | "depth" =>
            {
                let Ok(depth) = args[1].parse::<u8>().map(Depth::from)
                else
                {
                    let err = Error::for_parse::<Depth>(args[1].to_owned());
                    return Err(err.chain(base));
                };
                Ok(SearchArgs::Depth(depth))
            }
            | _ => Err(base),
        }
    }
}
