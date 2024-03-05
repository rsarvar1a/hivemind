use crate::prelude::*;

/// A dense representation of a piece.
///
/// Bits:
///
/// 0 - 1: num
///
/// 2 - 4: kind
///
/// 5 - 5: player
///
/// 6 - 7: empty
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token(u8);

impl From<Piece> for Token
{
    fn from(value: Piece) -> Self
    {
        let p: u8 = (value.player as u8) << Self::OFFSET_PLAYER;
        let k: u8 = (value.kind as u8) << Self::OFFSET_KIND;
        let n: u8 = value.num << Self::OFFSET_NUMBER;

        Token(p | k | n)
    }
}

impl From<Token> for Option<Piece>
{
    fn from(value: Token) -> Option<Piece>
    {
        match value.valid()
        {
            | true =>
            {
                let player = value.player();
                let kind = value.kind();
                let num = value.number();

                Some(Piece { player, kind, num })
            }
            | false => None,
        }
    }
}

impl From<u8> for Token
{
    fn from(value: u8) -> Self
    {
        Token(value)
    }
}

impl From<Token> for u8
{
    fn from(value: Token) -> u8
    {
        value.0
    }
}

impl Token
{
    const OFFSET_PLAYER: u8 = 5;
    const EXTENT_PLAYER: u8 = 0x1;

    const OFFSET_KIND: u8 = 2;
    const EXTENT_KIND: u8 = 0x7;

    const OFFSET_NUMBER: u8 = 0;
    const EXTENT_NUMBER: u8 = 0x3;

    /// Extracts the bug type from this token.
    pub fn kind(&self) -> Bug
    {
        unsafe { std::mem::transmute::<u8, Bug>((self.0 >> Self::OFFSET_KIND) & Self::EXTENT_KIND) }
    }

    /// Extracts the piece number from this token.
    pub fn number(&self) -> u8
    {
        (self.0 >> Self::OFFSET_NUMBER) & Self::EXTENT_NUMBER
    }

    /// Extracts the player from this token.
    pub fn player(&self) -> Player
    {
        unsafe { std::mem::transmute::<u8, Player>((self.0 >> Self::OFFSET_PLAYER) & Self::EXTENT_PLAYER) }
    }

    /// Determines if this token is valid.
    pub fn valid(&self) -> bool
    {
        let n = self.number();
        (1..=3).contains(&n)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A stack of tokens that also notes its own height.
pub struct Stack(u64);

impl std::fmt::Display for Stack
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let height = self.height();

        if height == 0
        {
            write!(f, "empty")?;
            return Ok(());
        }

        write!(f, "stack")?;
        for i in 1..=height
        {
            let token: Token = self._at(i).into();
            let piece = Option::<Piece>::from(token);
            let piece_repr = piece.map(|piece| format!("{}", piece)).unwrap_or("".into());
            write!(f, " {: <4}", piece_repr)?;
        }
        Ok(())
    }
}

#[allow(unused)]
impl Stack
{
    const MASK: u64 = u8::MAX as u64;
    const WIDTH: u8 = 8;

    /// Whether or not the stack contains a given token.
    pub fn contains(&self, token: Token) -> bool
    {
        let h = self.height();
        for shift in 1..=h
        {
            let here: Token = self._at(shift).into();
            if here == token
            {
                return true;
            }
        }
        false
    }

    // Determines if the stack is empty.
    pub fn empty(&self) -> bool
    {
        self.height() == 0
    }

    /// Determines if the stack is full.
    pub fn full(&self) -> bool
    {
        self.height() == 7
    }

    /// Determines the height of this stack.
    pub fn height(&self) -> u8
    {
        (self.0 & Self::MASK) as u8
    }

    /// Returns the octet representation of the stack.
    pub fn octets(&self) -> String
    {
        let perceived_state = if self.empty() { "empty" } else { "stack" };
        let contents = self.0.to_ne_bytes().map(|byte| format!(" 0b{:08b}", byte)).join(" ");
        format!("{} {}", perceived_state, contents)
    }

    /// Pops the top token off the stack and returns it.
    pub fn pop(&mut self) -> Token
    {
        let t = self.top();
        self._pop();
        t
    }

    /// Pushes a new token to the top of the stack; if there is no room, does nothing.
    pub fn push(&mut self, token: Token)
    {
        self._push(token);
    }

    /// Returns the token at the top of the stack, which is empty if there is none.
    pub fn top(&self) -> Token
    {
        let h = self.height();
        let bits = self._at(h);
        bits.into()
    }
}

impl Stack
{
    fn _at(&self, height: u8) -> u8
    {
        ((self.0 >> (Self::WIDTH * height)) & Self::MASK) as u8
    }

    /// Sets a new height on this stack.
    fn _height(&mut self, height: u8)
    {
        self.0 &= !(u8::MAX as u64);
        self.0 |= height.clamp(0, 7) as u64 & Self::MASK;
    }

    /// Zeroes the last element and decrements the height.
    fn _pop(&mut self)
    {
        let h = self.height();
        if h == 0
        {
            return;
        }

        self._height(h - 1);
        self._set(h, Token(0));
    }

    /// Sets the highest empty slot to be the given token, and increments the height.
    fn _push(&mut self, token: Token)
    {
        let h = self.height();
        if h == 7
        {
            return;
        }

        self._height(h + 1);
        self._set(h + 1, token);
    }

    /// Sets the value at the given height.
    fn _set(&mut self, height: u8, token: Token)
    {
        let h = height.clamp(1, 7);
        let w = Self::WIDTH * h;
        self.0 &= (!(u8::MAX as u64)).rotate_left(w.into());
        self.0 |= (token.0 as u64) << w;
    }
}
