use std::process::{Command, Stdio, ChildStdin, ChildStdout};
use std::io::{BufReader, Write, BufRead};
use std::thread;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Mutex, Arc};

// use log::{debug, warn};
use vampirc_uci::{ByteVecUciMessage, UciMessage, parse_one, UciFen, UciSearchControl, UciTimeControl, UciInfoAttribute, UciMove};
use vampirc_uci::Duration;
use std::collections::HashMap;
// use itertools::Itertools;

#[derive(Clone, Debug)]
pub enum Analysis {
    PossibleMove(PossibleMove),
    BestMove(UciMove)
}

impl Analysis {
    pub fn as_possible_move(&self) -> &PossibleMove {
        match self {
            Analysis::PossibleMove(pmv) => pmv,
            Analysis::BestMove(_) => panic!("Not a PossibleMove")
        }
    }
}

/// This is a candidate move given the depth
#[derive(Clone, Default, Debug)]
pub struct PossibleMove {
    pub depth: u8,
    pub score: i32,
    pub multi_pv: u16,
    pub moves: Vec<UciMove>
}

#[derive(Debug, Clone)]
pub struct Uci {
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
}

impl Uci {
    /// Starts an engine initializing it by taking a Command with all
    /// appropriate arguments passed for UCI
    pub fn start_engine(engine :&mut Command) -> Self {
        // create a child process
        let child = engine.stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("Error starting engine");

        let mut stdin = child.stdin.unwrap();
        let mut stdout = BufReader::new(child.stdout.unwrap());

        // init with the UCI message
        Self::inner_send_msg(&mut stdin, UciMessage::Uci);

        // we manually read because a lot of engines send non-UCI at first
        let mut msg_buffer = String::new();

        stdout.read_line(&mut msg_buffer).expect("Error reading");

        while msg_buffer.find("id ").is_none() {
            msg_buffer.clear();
            stdout.read_line(&mut msg_buffer).expect("Error reading");
        }

        // found the first id line
        let start = msg_buffer.find("id ").unwrap();
        let mut message = parse_one(&msg_buffer.as_str()[start..]);

        loop {
            // println!("MSG: {:?}", message);

            // go until we get the OK
            if let UciMessage::UciOk = message {
                break
            }

            // keep reading messages
            message = Self::inner_recv_msg(&mut stdout) ;
        }

        // check to see if it's ready
        Self::inner_send_msg(&mut stdin, UciMessage::IsReady);
        message = Self::inner_recv_msg(&mut stdout) ;

        // println!("MSG: {:?}", message);

        if UciMessage::ReadyOk != message {
            panic!("Error setting up engine");
        }

        // let the engine we're staring a new game
        // Self::inner_send_msg(&mut stdin, UciMessage::UciNewGame);

        // bump the number of threads so it works faster :-)
        Self::inner_send_msg(&mut stdin, UciMessage::SetOption {name: "Threads".to_string(), value: Some("4".to_string())});

        // tell it to use analysis mode
        Self::inner_send_msg(&mut stdin, UciMessage::SetOption { name: "UCI_AnalyseMode".to_string(), value: Some("true".to_string()) });

        // check to see if it's ready
        Self::inner_send_msg(&mut stdin, UciMessage::IsReady);
        message = Self::inner_recv_msg(&mut stdout) ;

        if let UciMessage::ReadyOk = message {
            Uci {
                stdin: Arc::new(Mutex::new(stdin)),
                stdout: Arc::new(Mutex::new(stdout))
            }
        } else {
            panic!("Error setting up engine");
        }
    }

    pub fn set_option(&mut self, name :&str, value :&str) {
        let mut stdin = self.stdin.lock().unwrap();
        let mut stdout = self.stdout.lock().unwrap();

        // send the option message
        Self::inner_send_msg(&mut stdin, UciMessage::SetOption { name: name.to_string(), value: Some(value.to_string()) });

        // check to see if it's ready
        Self::inner_send_msg(&mut stdin, UciMessage::IsReady);

        if UciMessage::ReadyOk != Self::inner_recv_msg(&mut stdout) {
            panic!("Error setting option")
        }
    }

    fn inner_send_msg(stdin :&mut ChildStdin, message :UciMessage) {
        // println!("SENDING MSG: {}", message.to_string());
        stdin.write_all(ByteVecUciMessage::from(message).as_ref()).expect("Error writing");
        stdin.flush().expect("Error flushing");
    }

    pub fn send_msg(&mut self, message :UciMessage) {
        let mut stdin = self.stdin.lock().unwrap();

        Self::inner_send_msg(&mut stdin, message);
    }

    fn inner_recv_msg(stdout: &mut BufReader<ChildStdout>) -> UciMessage {
        let mut buff = String::new();

        stdout.read_line(&mut buff).expect("Error reading");
        parse_one(buff.as_str())
    }

    pub fn recv_msg(&mut self) -> UciMessage {
        let mut stdout = self.stdout.lock().unwrap();

        Self::inner_recv_msg(&mut stdout)
    }


    /// Given a game, and additional moves to consider, and a depth; analyze the game
    /// A Receiver of Analysis structs is returned
    /// When the depth is reached (None for infinite), or the Receiver is dropped,
    /// the engine will stop its analysis
    pub fn analyze(&mut self, pos :String, depth :u8) -> Receiver<Analysis> {
        { // scope our lock
            let mut stdin = self.stdin.lock().unwrap();

            // set the position
            Self::inner_send_msg(&mut stdin, UciMessage::Position {
                startpos: false,
                fen: Some(UciFen(pos)),
                moves: vec![]
            });

            // tell the engine to start processing
            Self::inner_send_msg(&mut stdin, UciMessage::Go {
                // time_control: None,
                time_control: None,
                search_control: Some(UciSearchControl {
                    search_moves: vec![],
                    mate: None,
                    depth: Some(depth),
                    nodes: None
                })
            });
        }

        // clone STDIN & STDOUT
        let stdin_clone = self.stdin.clone();
        let stdout_clone = self.stdout.clone();

        // create a channel for sending back the analysis
        let (tx, rx) = channel();

        // spawn a thread to read the messages from the engine
        thread::spawn(move || {
            // read everything it sent back
            loop {
                let message = {
                    let mut stdout = stdout_clone.lock().unwrap();

                    Self::inner_recv_msg(&mut stdout)
                };

                // debug!("MSG: {:?}", message);

                // convert the messages into Analysis
                let analysis = match message {
                    // convert this into a PossibleMove
                    UciMessage::Info(attrs) => {
                        let mut possible_move = PossibleMove::default();

                        // set this to 1 just in case we didn't set the MultiPV option above
                        possible_move.multi_pv = 1;

                        // debug!("ATTRS: {:?}", attrs);

                        for attr in attrs {
                            match attr {
                                UciInfoAttribute::Depth(d) => { possible_move.depth = d; },
                                UciInfoAttribute::Score { cp, mate, .. } => { if let Some(score) = cp { possible_move.score = score; } },
                                UciInfoAttribute::Pv(moves) => { possible_move.moves = moves; }
                                UciInfoAttribute::MultiPv(multi_pv) => { possible_move.multi_pv = multi_pv; }
                                // UciInfoAttribute::CurrMove(chess_move) => { info.push_str(&chess_move.to_string()); },
                                // UciInfoAttribute::String(s) => { eprintln!("STR: {}", s); }
                                _ => ()
                            }
                        }

                        // debug!("POSSIBLE MOVE: {} {} {:?}",
                        //        possible_move.depth,
                        //        possible_move.score,
                        //        possible_move.moves.iter().map(|mv| format!("{}", mv)).collect::<Vec<_>>());

                        Analysis::PossibleMove(possible_move)
                    },
                    UciMessage::BestMove { best_move, ponder } => {
                        Analysis::BestMove(best_move)
                    }
                    _ => {
                        panic!("Unexpected message: {:?}", message)
                    }
                };

                let break_loop = if let Analysis::BestMove(_) = analysis { true } else { false };

                // send the analysis, check for disconnected receiver
                if let Err(send_err) = tx.send(analysis) {
                    // debug!("SEND ERR: {:?}", send_err);

                    // tell the engine to stop
                    let mut stdin = stdin_clone.lock().unwrap();
                    Self::inner_send_msg(&mut stdin, UciMessage::Stop);
                }

                // if we got the best move, then break out of the loop
                if break_loop {
                    break
                }
            }
        });

        // return the receiver side of the channel
        rx
    }

}


#[cfg(test)]
mod uci_tests {
    use std::process::Command;
    use std::convert::TryFrom;
    use std::str::FromStr;

    use chess::{Game, ChessMove, Square};
    use crate::uci::{Uci, Analysis};
    use simple_logger::SimpleLogger;
    use std::time::Duration;

    // #[test]
    // fn start_gnuchess_test() {
    //     let mut cmd = Command::new("/usr/games/gnuchess");
    //
    //     let uci = Uci::start_engine(cmd.arg("-u"));
    // }

    #[test]
    fn start_stockfish_test() {
        let mut cmd = Command::new("/usr/local/bin/stockfish");

        let uci = Uci::start_engine(&mut cmd);
    }

    #[test]
    fn start_ethereal_test() {
        let mut cmd = Command::new("/usr/games/ethereal-chess");

        let uci = Uci::start_engine(&mut cmd);
    }

    #[test]
    fn analyze_test() {
        SimpleLogger::new().init().unwrap();
        let mut cmd = Command::new("/usr/games/ethereal-chess");
        let mut uci = Uci::start_engine(&mut cmd);
        let game = Game::from_str("r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/3P1P2/PPP3PP/RNBQKBNR w KQkq - 0 1").expect("Error creating game");

        let rx = uci.analyze(&game, vec![], Some(7));

        for analysis in rx {
            if let Analysis::BestMove(mv) = analysis {
                println!("{:?}", analysis);
            }
        }

        let rx = uci.analyze(&game, vec![], Some(7));

        for analysis in rx {
            if let Analysis::BestMove(mv) = analysis {
                println!("{:?}", analysis);
            }
        }
    }

    #[test]
    fn check_for_blunder_true_test() {
        SimpleLogger::new().init().unwrap();
        let mut cmd = Command::new("/usr/local/bin/stockfish");
        let mut uci = Uci::start_engine(&mut cmd);

        uci.set_option("UCI_AnalyseMode", "true");
        uci.set_option("MultiPV", "5");

        // let game = Game::from_str("r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/3P1P2/PPP3PP/RNBQKBNR w KQkq - 0 1").expect("Error creating game");
        // let blunder_move = ChessMove::new(Square::B2, Square::B4, None);

        let game = Game::from_str("r1bqkb1r/pppp1ppp/5n2/4p3/2PnP3/3P1P2/PP4PP/RNBQKBNR w KQkq - 1 2").expect("Error creating game");
        let blunder_move = ChessMove::new(Square::D1, Square::B3, None);

        let (mv, score) = uci.check_for_blunder(&game, blunder_move, 5);

        assert!(mv) // this is a blunder
    }

    #[test]
    fn check_for_blunder_false_test() {
        SimpleLogger::new().init().unwrap();
        let mut cmd = Command::new("/usr/local/bin/stockfish");
        let mut uci = Uci::start_engine(&mut cmd);

        uci.set_option("UCI_AnalyseMode", "true");
        uci.set_option("MultiPV", "5");

        let game = Game::from_str("r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/3P1P2/PPP3PP/RNBQKBNR w KQkq - 0 1").expect("Error creating game");
        let blunder_move = ChessMove::new(Square::C1, Square::G5, None);

        let (mv, score) = uci.check_for_blunder(&game, blunder_move, 5);

        assert!(!mv) // this isn't a blunder
    }

}
