#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// The options applied to a game of hive.
pub struct Options
{
    /// At the moment, this doesn't actually do *anything*, because the UHP doesn't support it.
    pub tournament: bool,

    /// The expansions enabled on this game.
    pub expansions: ExpansionOptions,
}

impl Options
{
    /// Returns a fully-featured set of Options, including all bugs and tournament settings.
    pub fn all() -> Self
    {
        Options {
            tournament: true,
            expansions: ExpansionOptions::all(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// The expansion bugs enabled on this game.
pub struct ExpansionOptions
{
    pub ladybug:  bool,
    pub mosquito: bool,
    pub pillbug:  bool,
}

impl ExpansionOptions
{
    /// Sets each expansion bug to be in-play.
    pub fn all() -> Self
    {
        ExpansionOptions {
            ladybug:  true,
            mosquito: true,
            pillbug:  true,
        }
    }
}
