mod common;
use common::*;

#[cfg(test)]
mod base_lmp
{
    use super::*;

    #[test]
    fn empty()
    {
        let _setup = setup::setup();
        let raw_string = "Base+LMP;NotStarted;White[1]";
        templates::run_game(raw_string);
    }
}
