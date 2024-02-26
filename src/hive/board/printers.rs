use crate::prelude::*;

impl Board
{
    /// Standard debug.
    pub(super) fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "Board {{ {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, }}",
            self.field, self.history, self.immune, self.options, self.pieces, self.pouch, self.stunned, self.zobrist
        )
    }

    /// Pretty print.
    pub(super) fn pretty(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let stacks = self
            .stacks
            .iter()
            .enumerate()
            .filter(|(_, stack)| !stack.empty())
            .map(|(i, stack)| format!("{}: {}", Axial::from(i as Hex), stack))
            .collect::<Vec<String>>();

        write!(
            f,
            "Board {{ {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?} }}\nwith {:#?}\nwith Stacks {:#?}",
            self.field, self.immune, self.options, self.pieces, self.pouch, self.stunned, self.zobrist, self.history, stacks
        )
    }
}
