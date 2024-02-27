mod common;
use common::*;

#[cfg(test)]
mod base
{
    use super::*;

    #[test]
    fn empty()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;NotStarted;White[1]";
        templates::run_game(raw_string);
    }

    #[test]
    fn first_move_ok_white()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;Black[1];wA1";
        templates::run_game(raw_string);
    }

    #[test]
    fn first_move_ok_black()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;White[2];wA1;bS1 /wA1";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn gate_ground_level()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;White[5];wG1;bG1 wG1\;wQ -wG1;bQ /bG1;wA1 \wG1;bA1 bG1\;wA1 -bQ;bA1 -bG1";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn gate_elevated()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;White[9];wG1;bG1 wG1\;wQ -wG1;bQ /bG1;wA1 \wG1;bA1 bG1\;wA1 -bQ;bB1 bA1/;wB1 \wG1;bB2 bQ\;wB1 wG1;bB2 bQ;wS1 -wA1;bB1 bG1;wG2 -wS1;bB1 wQ\";
        templates::run_game(raw_string);
    }

    #[test]
    fn disjoint_perimeters_ok()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;Black[7];wG1;bG1 wG1\;wQ -wG1;bQ /bG1;wA1 \wG1;bA1 /bQ;wG2 -wA1;bA2 -bA1;wG3 /wG2;bA3 -bA2;wA2 /wG3;bS1 /bA3;wQ wG3\";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn disjoint_perimeters_unreachable()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;White[8];wG1;bG1 wG1\;wQ -wG1;bQ /bG1;wA1 \wG1;bA1 /bQ;wG2 -wA1;bA2 -bA1;wG3 /wG2;bA3 -bA2;wA2 /wG3;bS1 /bA3;wQ wG3\;bS1 bA3/";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn no_expansion_bugs()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;Black[1];wL";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn placed_on_top()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;White[2];wA1;bS1 wA1";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn queen_on_first_move_white()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;Black[1];wQ";
        templates::run_game(raw_string);
    }

    #[test]
    #[should_panic]
    fn queen_on_first_move_black()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;White[2];wA1;bQ wA1-";
        templates::run_game(raw_string);
    }

    #[test]
    fn multi_height_hop()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;InProgress;Black[6];wG1;bG1 wG1\;wQ \wG1;bQ /bG1;wA1 -wQ;bB1 bG1\;wB1 wA1\;bB1 bG1;wB1 /wG1;bB1 wG1;wB1 bB1";
        templates::run_game(raw_string);
    }

    #[test]
    fn draw()
    {
        let _setup = setup::setup();
        let raw_string = r"Base;Draw;Black[8];wS1;bS1 wS1\;wQ -wS1;bQ /bS1;wG1 \wS1;bG1 bS1\;wB1 -wG1;bB1 bQ\;wA1 /wQ;bA1 /bQ;wS2 /wB1;bA1 wA1\;wG2 \wB1;bG2 bA1\;wG2 wQ\";
        templates::run_game(raw_string);
    }
}
