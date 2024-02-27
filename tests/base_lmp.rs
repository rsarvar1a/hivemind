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

    #[test]
    #[should_panic]
    fn ladybug_gate()
    {
        let _setup = setup::setup();
        let raw_string = r"Base+LMP;InProgress;Black[14];wL;bG1 wL/;wQ -wL;bQ bG1/;wQ -bG1;bG2 bQ-;wB1 \wQ;bB1 bG2-;wS1 \wB1;bB1 bG2;wS2 \wS1;bG3 \bQ;wG1 wS2/;bB2 bG3/;wB2 wG1/;bB2 bG3;wA1 wB2-;bA1 bB1-;wA2 wA1-;bA1 bB1\;wG2 wA2-;bA1 bB1-;wG3 wG2\;bA1 bB1\;wA3 wG3\;bA1 wA3\;wL bQ/";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn pillbug_threw_last_moved()
    {
        let _setup = setup::setup();
        let raw_string = r"Base+LMP;InProgress;White[15];wP;bS1 wP-;wQ /wP;bQ bS1-;wB1 -wQ;bB1 bS1\;wG1 wB1\;bB1 wP\;wS1 wG1\;bQ bS1/;wB1 -wP;bB1 wQ;wG2 wS1\;bB1 wB1;wG3 wG2\;bA1 bQ\;wS2 wG3-;bA1 bS1\;wA1 wS2/;bA1 bQ\;wA2 wA1/;bA1 bS1\;wA3 wA2/;bA1 bQ\;wB2 wA3/;bA1 wB2/;pass;bQ \bS1;bS1 -bQ;bS1 wP\";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn pillbug_gate_onto()
    {
        let _setup = setup::setup();
        let raw_string = r"Base+LMP;InProgress;White[12];wP;bB1 wP-;wQ /wP;bQ bB1/;wQ wP\;bQ \bB1;wQ /wP;bA1 bQ/;wQ wP\;bA1 -bQ;wQ /wP;bB2 \bQ;wQ wP\;bB2 bQ;wA1 wQ\;bA1 -wP;wA1 /wQ;bB1 wQ;wA1 bB1\;bA1 bB2\;wB1 wA1\;bM bB2/;bA1 -wP";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn pillbug_gate_after()
    {
        let _setup = setup::setup();
        let raw_string = r"Base+LMP;InProgress;Black[9];wP;bB1 wP-;wQ /wP;bQ bB1/;wQ wP\;bQ \bB1;wQ /wP;bA1 bQ/;wQ wP\;bA1 -bQ;wQ /wP;bB2 \bQ;wQ wP\;bB2 bQ;bA1 -wP;bB1 wQ;bA1 wP-";
        templates::run_game(raw_string);
    }
}
