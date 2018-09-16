#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate rustty;

extern crate negamax;

use negamax::{GameState, Table};
use rand::{thread_rng, Rng};
use std::time;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    board: [i32; 3 * 8],
    hands: [usize; 2],
}

impl State {
    fn new() -> State {
        State {
            board: [0; 3 * 8],
            hands: [9, 9], // [player +1, player -1]
        }
    }

    fn play(&self, player: i32, from: usize, to: usize) -> Vec<State> {
        let mut state = self.clone();
        if from == to {
            state.hands[((1 - player) / 2) as usize] -= 1;
        }
        state.board[from] = 0;
        state.board[to] = player;
        let state = state;

        let mut r = vec![state.clone()];
        for mill in &MILLSFROM[to] {
            if mill.iter().all(|&i| state.board[i] == player) {
                for (i, &x) in state.board.iter().enumerate() {
                    if x == -player
                        && !MILLSFROM[i]
                            .iter()
                            .any(|mill| mill.iter().all(|&j| state.board[j] == -player))
                    {
                        let mut eat = state.clone();
                        eat.board[i] = 0;
                        r.push(eat);
                    }
                }
            }
        }
        r
    }

    fn d4rotation(&self) -> State {
        let mut state = self.clone();
        for (i, &j) in D4ROTATION.iter().enumerate() {
            state.board[i] = self.board[j];
        }
        state
    }

    fn d4mirror(&self) -> State {
        let mut state = self.clone();
        for (i, &j) in D4MIRROR.iter().enumerate() {
            state.board[i] = self.board[j];
        }
        state
    }
}

impl<'a> GameState<'a> for State {
    type It = Vec<State>;

    // compute the value in player +1 perspective
    fn value(&self) -> i32 {
        let mut value = 0;
        for mill in &MILLS {
            let me = mill.iter().filter(|&&i| self.board[i] == 1).count();
            let op = mill.iter().filter(|&&i| self.board[i] == -1).count();
            value += match (me, op) {
                (3, 0) => 4,
                (0, 3) => -4,
                (2, 0) => 4,
                (0, 2) => -4,
                (1, 0) => 1,
                (0, 1) => -1,
                (_, _) => 0,
            }
        }
        let me = self.board.iter().filter(|&&x| x == 1).count() as i32 + self.hands[0] as i32;
        let op = self.board.iter().filter(|&&x| x == -1).count() as i32 + self.hands[1] as i32;
        if op < 3 {
            value += 100;
        }
        if me < 3 {
            value -= 100;
        }
        value
    }

    fn possibilities(&self, player: i32) -> Vec<State> {
        let mut r = Vec::new();
        if self.hands[((1 - player) / 2) as usize] > 0 {
            for i in 0..24 {
                if self.board[i] == 0 {
                    r.append(&mut self.play(player, i, i));
                }
            }
        } else {
            for (i, con) in CONNECTED.iter().enumerate() {
                if self.board[i] == player {
                    for &j in con {
                        if self.board[j] == 0 {
                            r.append(&mut self.play(player, i, j));
                        }
                    }
                }
            }
        }
        if r.is_empty() {
            r.push(self.clone());
        }
        r
    }

    fn win(&self, player: i32) -> bool {
        self.hands[((1 + player) / 2) as usize] == 0
            && self.board.iter().filter(|&&x| x == -player).count() < 3
    }

    fn swap(&mut self) {
        self.hands.swap(0, 1);
        for x in &mut self.board {
            *x = -*x;
        }
    }

    fn symmetries(&self) -> Vec<State> {
        vec![
            self.clone(),
            self.d4rotation(),
            self.d4rotation().d4rotation(),
            self.d4rotation().d4rotation().d4rotation(),
            self.d4mirror(),
            self.d4mirror().d4rotation(),
            self.d4mirror().d4rotation().d4rotation(),
            self.d4mirror().d4rotation().d4rotation().d4rotation(),
        ]
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut text = String::from(
            r#"       h+◎  h-◉
00─────01─────02
│ 08───09───10 │
│ │ 16 17 18 │ │
07─15─23   19─11─03
│ │ 22 21 20 │ │
│ 14───13───12 │
06─────05─────04"#,
        );
        text = text.replace("h+", &format!("{}", self.hands[0]));
        text = text.replace("h-", &format!("{}", self.hands[1]));

        for (i, &x) in self.board.iter().enumerate() {
            text = text.replace(
                &format!("{:02}", i),
                if x == 1 {
                    "◎"
                } else if x == -1 {
                    "◉"
                } else {
                    " "
                },
            );
        }

        write!(f, "{}", text)
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

use rustty::ui::{Alignable, HorizontalAlign, VerticalAlign, Widget};
use rustty::{CellAccessor, Event, HasSize, Terminal};

fn main() {
    let mut term = Terminal::new().unwrap();
    let mut canvas = Widget::new(term.size().0 - 2, term.size().1 - 2);
    canvas.align(&term, HorizontalAlign::Left, VerticalAlign::Top, 1);

    let pos = [
        (-3, -3),
        (0, -3),
        (3, -3),
        (3, 0),
        (3, 3),
        (0, 3),
        (-3, 3),
        (-3, 0),
        (-2, -2),
        (0, -2),
        (2, -2),
        (2, 0),
        (2, 2),
        (0, 2),
        (-2, 2),
        (-2, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
    ];

    let mut table: Table<State> = Table::new();
    let mut state = State::new();
    let mut moves = state.possibilities(1);
    let mut index: isize = -1;

    'main: loop {
        while let Some(Event::Key(ch)) = term.get_event(time::Duration::new(0, 0)).unwrap() {
            match ch {
                'q' => break 'main,
                'C' => {
                    if index < moves.len() as isize - 1 {
                        index += 1;
                    }
                }
                'D' => {
                    if index > -1 {
                        index -= 1;
                    }
                }
                '\r' => {
                    if index >= 0 {
                        state = moves[index as usize].clone();
                        if state.win(1) {
                            break 'main;
                        }
                        state = thread_rng()
                            .choose(&state.bot_play(-1, 7, &mut table))
                            .unwrap()
                            .clone();
                        if state.win(-1) {
                            break 'main;
                        }
                        moves = state.possibilities(1);
                        index = -1;
                    }
                }
                _ => {}
            }
        }
        // Grab the size of the canvas
        let (cols, rows) = canvas.size();

        // Main render loop, draws the circle to canvas
        for i in 0..cols * rows {
            let y = i / cols;
            let x = i % cols;
            let mut cell = canvas.get_mut(x, y).unwrap();
            cell.set_ch(' ');
        }

        let draw_state = if index >= 0 {
            moves[index as usize].clone()
        } else {
            state.clone()
        };

        let (a, b) = (cols as isize / 2, rows as isize / 2);
        if index == -1 {
            canvas
                .get_mut(a as usize, b as usize)
                .unwrap()
                .set_ch('~');
        }
        for i in 0..24 {
            let x = a + 2 * pos[i].0;
            let y = b + pos[i].1;
            let mut cell = canvas.get_mut(x as usize, y as usize).unwrap();
            let ch = match draw_state.board[i] {
                0 => '.',
                1 => '◎',
                -1 => '◉',
                _ => ' ',
            };
            cell.set_ch(ch);
        }

        // draw the canvas, dialog window and swap buffers
        canvas.draw_into(&mut term);
        term.swap_buffers().unwrap();
    }

    // let mut player = 1;
    // let mut depth = 7;
    // loop {
    //     let mut new_state;
    //     loop {
    //         let before = time::Instant::now();

    //         new_state = thread_rng()
    //             .choose(&state.bot_play(player, depth, &mut table))
    //             .unwrap()
    //             .clone();

    //         if time::Instant::now() - before > time::Duration::from_secs(1) {
    //             break;
    //         } else {
    //             depth += 1;
    //         }
    //     }
    //     state = new_state;
    //     player = -player;

    //     println!("{} table={} depth={}", state, table.len(), depth);
    //     if state.win(-player) {
    //         break;
    //     }
    // }
}

/* Players : +1 and -1
 *
 * 00       01       02
 *    08    09    10
 *       16 17 18
 * 07 15 23    19 11 03
 *       22 21 20
 *    14    13    12
 * 06       05       04
 */

static D4ROTATION: [usize; 3 * 8] = [
    2, 3, 4, 5, 6, 7, 0, 1, 10, 11, 12, 13, 14, 15, 8, 9, 18, 19, 20, 21, 22, 23, 16, 17,
];

static D4MIRROR: [usize; 3 * 8] = [
    2, 1, 0, 7, 6, 5, 4, 3, 10, 9, 8, 15, 14, 13, 12, 11, 18, 17, 16, 23, 22, 21, 20, 19,
];

lazy_static! {
    static ref CONNECTED: [Vec<usize>; 3 * 8] = [
        vec![1,7],   vec![0,2,9],     vec![1,3],   vec![2,4,11],     vec![3,5],   vec![4,6,13],     vec![5,7],   vec![0,6,15], // 0-7
        vec![9,15],  vec![1,8,17,10], vec![9,11],  vec![3,10,19,12], vec![11,13], vec![5,12,14,21], vec![13,15], vec![7,14,23,8], //8-15
        vec![17,23], vec![16,18,9],   vec![17,19], vec![18,11,20],   vec![19,21], vec![20,22,13],   vec![21,23], vec![15,16,22] // 16-23
    ];
}

static MILLS: [[usize; 3]; 16] = [
    [0, 1, 2],
    [8, 9, 10],
    [16, 17, 18],
    [7, 15, 23],
    [19, 11, 3],
    [22, 21, 20],
    [14, 13, 12],
    [6, 5, 4],
    [0, 7, 6],
    [8, 15, 14],
    [16, 23, 22],
    [1, 9, 17],
    [21, 13, 5],
    [18, 19, 20],
    [10, 11, 12],
    [2, 3, 4],
];

static MILLSFROM: [[[usize; 3]; 2]; 3 * 8] = [
    [[0, 1, 2], [0, 7, 6]],
    [[0, 1, 2], [1, 9, 17]],
    [[0, 1, 2], [2, 3, 4]],
    [[19, 11, 3], [2, 3, 4]],
    [[6, 5, 4], [2, 3, 4]],
    [[6, 5, 4], [21, 13, 5]],
    [[6, 5, 4], [0, 7, 6]],
    [[7, 15, 23], [0, 7, 6]],
    [[8, 9, 10], [8, 15, 14]],
    [[8, 9, 10], [1, 9, 17]],
    [[8, 9, 10], [10, 11, 12]],
    [[19, 11, 3], [10, 11, 12]],
    [[14, 13, 12], [10, 11, 12]],
    [[14, 13, 12], [21, 13, 5]],
    [[14, 13, 12], [8, 15, 14]],
    [[7, 15, 23], [8, 15, 14]],
    [[16, 17, 18], [16, 23, 22]],
    [[16, 17, 18], [1, 9, 17]],
    [[16, 17, 18], [18, 19, 20]],
    [[19, 11, 3], [18, 19, 20]],
    [[22, 21, 20], [18, 19, 20]],
    [[22, 21, 20], [21, 13, 5]],
    [[22, 21, 20], [16, 23, 22]],
    [[7, 15, 23], [16, 23, 22]],
];
