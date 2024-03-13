use clap::Parser;
use minimax::{IterativeOptions, ParallelOptions, ParallelSearch, Strategy};

use crate::prelude::*;

#[derive(Clone, Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct UhpOptions
{
    #[arg(long, default_value_t = 1.0)]
    /// maximum memory in GB for LFU
    pub cache_memory: f64,

    #[arg(long, default_value_t = 1.0)]
    /// maximum memory in GB for transpositions
    pub table_memory: f64,

    #[arg(short, long, default_value = "info")]
    /// lowest log level to show
    pub log_level: String,

    #[arg(short, long, default_value_t = 0)]
    /// number of search threads
    pub num_threads: usize,

    #[arg(short, long, default_value_t = false)]
    /// whether to print minimax verbose output
    pub verbose: bool,
}

pub struct Server
{
    #[allow(unused)]
    options:  UhpOptions,
    board:    Option<Board>,
    strategy: minimax::ParallelSearch<HiveEval>,
}

impl Server
{
    /// Creates a new server with the given capabilities.
    pub fn new(options: UhpOptions) -> Self
    {
        let bytes = (options.table_memory * 1e+9).floor() as usize;

        let mut iter_opts = IterativeOptions::new()
            .with_table_byte_size(bytes)
            .with_countermoves()
            .with_null_window_search(true)
            .with_singular_extension();

        if options.verbose
        {
            iter_opts = iter_opts.verbose();
        }

        let num_threads = if options.num_threads == 0
        {
            std::thread::available_parallelism().map(|nzu| nzu.into()).unwrap_or(1)
        }
        else
        {
            options.num_threads
        };

        let par_opts = ParallelOptions::new().with_num_threads(num_threads).with_background_pondering();

        Server {
            options:  options.clone(),
            board:    None,
            strategy: ParallelSearch::new(HiveEval {}, iter_opts, par_opts),
        }
    }

    /// Starts the server, which cannot return.
    pub fn run(&mut self) -> Result<!>
    {
        let a_bit = std::time::Duration::from_secs(2);
        std::thread::sleep(a_bit);

        loop
        {
            let mut cmdstr: String = String::new();
            std::io::stdin().read_line(&mut cmdstr)?;

            let args: Vec<&str> = cmdstr.split_whitespace().filter(|s| !s.is_empty()).collect();
            let cmd = *args.first().unwrap_or(&"");

            self.apply(cmd, &args[1..])?;
        }
    }
}

impl Server
{
    /// Matches the command to the server's functionality.
    fn apply(&mut self, cmd: &str, args: &[&str]) -> Result<()>
    {
        let result = match cmd
        {
            | "" => Ok(()),
            | "bestmove" => self.best_move(args),
            | "info" => self.info(),
            | "newgame" => self.new_game(args),
            | "options" => self.options(args),
            | "pass" => self.play_move(&["pass"]),
            | "play" => self.play_move(args),
            | "undo" => self.undo(args),
            | "validmoves" => self.valid_moves(),
            | _ => Err(Error::new(Kind::UnrecognizedCommand, cmd.into())),
        };

        match result
        {
            | Ok(_) =>
            {
                log::debug!("Command completed successfully: {cmd} {}", args.join(" "));
                self.ok()
            }
            | Err(err) => match err.fatal()
            {
                | true =>
                {
                    let _ = self.err(&err);
                    Err(err)
                }
                | false =>
                {
                    log::warn!("encountered recoverable error:\n{err}");
                    self.err(&err)
                }
            },
        }
    }

    /// Returns the best move available in this position (for the player to move).
    fn best_move(&mut self, args: &[&str]) -> Result<()>
    {
        let search_args = SearchArgs::parse(args)?;
        match search_args
        {
            | SearchArgs::Depth(d) => self.strategy.set_max_depth(d),
            | SearchArgs::Time(dur) => self.strategy.set_timeout(dur),
        };

        let board = self.ensure_started()?;
        let mv = self.strategy.choose_move(&board.clone()).unwrap();

        println!("{}", Into::<MoveString>::into(mv));
        Ok(())
    }

    /// Ensures there is a board loaded on this server.
    fn ensure_started(&self) -> Result<&Board>
    {
        match self.board.as_ref()
        {
            | Some(b) => Ok(b),
            | None => Err(Error::empty(Kind::GameNotStarted)),
        }
    }

    /// Ensures there is a board loaded on this server.
    fn ensure_started_mut(&mut self) -> Result<&mut Board>
    {
        match self.board.as_mut()
        {
            | Some(b) => Ok(b),
            | None => Err(Error::empty(Kind::GameNotStarted)),
        }
    }

    /// Prints an error to the UHP stream.
    fn err(&self, err: &Error) -> Result<()>
    {
        println!("err\n{}", err);
        self.ok()
    }

    /// Prints the server's ID.
    fn info(&self) -> Result<()>
    {
        println!("id {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"),);
        println!("{};{};{}", Bug::Ladybug.long(), Bug::Mosquito.long(), Bug::Pillbug.long());
        Ok(())
    }

    /// Creates a new game with the given options string.
    fn new_game(&mut self, args: &[&str]) -> Result<()>
    {
        if !args.is_empty()
        {
            let gamestr: GameString = args.join(" ").parse::<GameString>()?;
            self.board = Some(gamestr.into());
        }
        else
        {
            self.board = Some(Board::default())
        }

        let board = self.ensure_started()?;
        println!("{}", GameString::from(board));
        Ok(())
    }

    /// Prints the ok footer to the UHP stream.
    fn ok(&self) -> Result<()>
    {
        println!("ok");
        Ok(())
    }

    /// Implements the optionsmap interface for this server.
    fn options(&mut self, _args: &[&str]) -> Result<()>
    {
        Ok(())
    }

    /// Plays the given move on the current board, if one exists.
    fn play_move(&mut self, args: &[&str]) -> Result<()>
    {
        if args.is_empty()
        {
            return Err(Error::new(Kind::ParseError, "You must provide a MoveString.".into()));
        }

        let board = self.ensure_started_mut()?;

        let mv = Move::from(&args.join(" ").parse::<MoveString>()?, &*board)?;
        board.play(&mv)?;

        println!("{}", GameString::from(&*board));
        Ok(())
    }

    #[allow(unused)]
    /// Placeholder for unimplemented features.
    fn todo(&self) -> Result<()>
    {
        Err(Error::new(Kind::InternalError, "not implemented".into()))
    }

    /// Undoes the given number of moves on the current board.
    fn undo(&mut self, args: &[&str]) -> Result<()>
    {
        let mut n: u8 = 1;
        match args.len()
        {
            | 0 =>
            {}
            | _ =>
            {
                let try_n = args[0].parse::<u8>();
                if let Ok(num) = try_n
                {
                    n = num;
                }
                else
                {
                    return Err(Error::for_parse::<u8>(args[0].into()));
                }
            }
        };

        let board = self.ensure_started_mut()?;
        board.undo(n)?;

        println!("{}", GameString::from(&*board));
        Ok(())
    }

    /// Gets all of the valid moves in this position.
    fn valid_moves(&self) -> Result<()>
    {
        let board = self.ensure_started()?;
        let mut moves = Vec::new();
        board.generate_moves(&mut moves, false);

        let movelist = moves
            .iter()
            .map(|mv| format!("{}", Into::<MoveString>::into(*mv)))
            .collect::<Vec<_>>()
            .join(";");
        let movelist = if movelist == "" { "pass".into() } else { movelist };

        println!("{}", movelist);
        Ok(())
    }
}
