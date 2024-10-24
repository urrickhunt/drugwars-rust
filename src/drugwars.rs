// drugwars.rs 🦀
// 40th Anniversary Drugwars in Rust
// urrick hunt

use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::process;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
#[allow(unused_imports)]
use {
    std::os::windows::io::AsRawHandle,
    winapi::shared::minwindef::DWORD,
    winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode},
    winapi::um::handleapi::INVALID_HANDLE_VALUE,
    winapi::um::processenv::GetStdHandle,
    winapi::um::winbase::STD_INPUT_HANDLE,
    winapi::um::wincon::{ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT},
    winapi::um::winnt::HANDLE,
};

#[cfg(unix)]
use {
    nix::libc::STDIN_FILENO,
    nix::sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg},
};

use chrono::{Datelike, NaiveDate};
use terminal_size::{terminal_size, Height, Width};

fn clear_screen() {
    if cfg!(windows) {
        process::Command::new("cmd")
            .args(["/C", "cls"])
            .status()
            .unwrap();
    } else {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }
}

static VERSION: &str = "0.6.11";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Drug {
    Cocaine,
    Heroin,
    Acid,
    Weed,
    Speed,
    Ludes,
}

impl Drug {
    fn as_str(&self) -> &'static str {
        match self {
            Drug::Cocaine => "COCAINE",
            Drug::Heroin => "HEROIN",
            Drug::Acid => "ACID",
            Drug::Weed => "WEED",
            Drug::Speed => "SPEED",
            Drug::Ludes => "LUDES",
        }
    }
}

struct GameState {
    day: i64,
    days_left: i64,
    geo: String,
    cash: i64,
    guns: i64,
    bank: i64,
    debt: i64,
    maxloan: i64,
    hold: i64,
    stash: HashMap<Drug, i64>,
    trench_coat: HashMap<Drug, i64>,
    state: String,
    damage: i64,
    cops: i64,
    prices: HashMap<Drug, i64>,
    width: usize,
    height: usize,
    wid: usize,
    gunprice: i64,
    coatspace: i64,
    coatprice: i64,
}

impl GameState {
    fn new() -> Self {
        let mut game = GameState {
            day: 0,
            days_left: 31,
            geo: "BRONX".to_string(),
            cash: 2000,
            guns: 0,
            bank: 0,
            debt: 5500,
            maxloan: 9450,
            hold: 100,
            stash: HashMap::new(),
            trench_coat: HashMap::new(),
            state: "begin".to_string(),
            damage: 0,
            cops: 0,
            prices: HashMap::new(),
            width: 80,
            height: 24,
            wid: 40,
            gunprice: 0,
            coatspace: 0,
            coatprice: 0,
        };

        for drug in [
            Drug::Cocaine,
            Drug::Heroin,
            Drug::Acid,
            Drug::Weed,
            Drug::Speed,
            Drug::Ludes,
        ]
        .iter()
        {
            game.stash.insert(*drug, 0);
            game.trench_coat.insert(*drug, 0);
            game.prices.insert(*drug, 0);
        }

        game.term_info();
        game
    }

    fn roll_prices(&mut self) {
        let mut rng = rand::thread_rng();

        let price_ranges = [
            (Drug::Cocaine, 1500..=3000),
            (Drug::Heroin, 500..=1400),
            (Drug::Acid, 100..=450),
            (Drug::Weed, 30..=90),
            (Drug::Speed, 7..=25),
            (Drug::Ludes, 1..=6),
        ];

        for (drug, range) in price_ranges.iter() {
            let price = rng.gen_range(range.clone()) * 10;
            self.prices.insert(*drug, price);
        }
    }

    fn term_info(&mut self) {
        if let Some((Width(w), Height(h))) = terminal_size() {
            self.width = w as usize;
            self.height = h as usize;
            self.wid = self.width / 2;
        }
    }

    fn format_number(n: i64) -> String {
        let num = n.abs();
        let sign = if n < 0 { "-" } else { "" };
        let s = num.to_string();
        let mut result = String::new();
        let chars: Vec<char> = s.chars().rev().collect();
        for (i, c) in chars.iter().enumerate() {
            if i != 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(*c);
        }
        result.push_str(sign);
        result.chars().rev().collect()
    }

    fn game_date_str(&self, days_to_add: i64) -> String {
        let start_date = NaiveDate::from_ymd_opt(1983, 12, 4).unwrap();
        let game_date = start_date + chrono::Duration::days(days_to_add);
        format!(
            "{:02} / {:02} / {:02}",
            game_date.month(),
            game_date.day(),
            game_date.year() % 100
        )
    }

    fn start_game(&mut self) {
        self.term_info();
        clear_screen();
        println!("\n\n");
        println!(
            "{}\x1B[1;32mDRUG WARS\x1B[0m",
            " ".repeat(self.wid - 5)
        );
        println!();
        println!(
            "{}\x1B[1;32mA GAME BASED ON\x1B[0m",
            " ".repeat(self.wid - 8)
        );
        println!();
        println!(
            "{}\x1B[1;32mTHE NEW YORK DRUG MARKET\x1B[0m",
            " ".repeat(self.wid - 12)
        );
        println!("\n\n\n\n");
        println!(
            "{}ORIGINAL BY JOHN E. DELL (1984)",
            " ".repeat(self.wid - 15)
        );
        println!();
        println!(
            "{}\x1B[35mRUST BY URRICK HUNT (2024)\x1B[0m",
            " ".repeat(self.wid - 13)
        );
        println!("\n\n\n\n");

        print!(
            "{}\x1B[33mDO YOU WANT INSTRUCTIONS?\x1B[0m ",
            " ".repeat(self.wid - 13)
        );
        io::stdout().flush().unwrap();
        loop {
            let reply = self.getch().unwrap().to_lowercase().next().unwrap();

            if reply == 'y' {
                self.instructions();
                break;
            } else if reply == 'n' {
                self.roll_event();
                self.main_menu();
                break;
            }
        }
    }

    fn instructions(&mut self) {
        self.term_info();
        clear_screen();
        println!("\n\n");
        println!(
            "{}\x1B[1;32mDRUG WARS\x1B[0m",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}This is a game of buying, selling, and",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}fighting. The object of the game is to",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}pay off your debt to the loan shark.",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}Then, make as much money as you can in a",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}1 month period. If you deal too heavily",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}in  drugs,  you  might  run  into  the",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}police !!  Your main drug stash will be",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}in the Bronx. (It's a nice neighborhood)",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}The prices of drugs per unit are:",
            " ".repeat(self.wid - 20)
        );
        println!("{}", " ".repeat(self.wid - 20));
        println!("{}", " ".repeat(self.wid - 20));
        println!(
            "{}      \x1B[35mCOCAINE     15000-30000\x1B[0m",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}      \x1B[35mHEROIN      5000-14000\x1B[0m",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}      \x1B[35mACID        1000-4500\x1B[0m",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}      \x1B[35mWEED        300-900\x1B[0m",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}      \x1B[35mSPEED       70-250\x1B[0m",
            " ".repeat(self.wid - 20)
        );
        println!(
            "{}      \x1B[35mLUDES       10-60\x1B[0m",
            " ".repeat(self.wid - 20)
        );
        println!("{}", " ".repeat(self.wid - 20));
        println!("{}", " ".repeat(self.wid - 20));
        println!("{}", " ".repeat(self.wid - 20));
        print!(
            "{}    \x1B[33m(HIT ANY KEY TO START GAME)\x1B[0m",
            " ".repeat(self.wid - 20)
        );
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
        self.roll_event();
        self.main_menu();
    }

    fn hud(&mut self) {
        self.term_info();
        clear_screen();

        let game_date = self.game_date_str(self.day);
        let bank_formatted = Self::format_number(self.bank);
        let debt_formatted = Self::format_number(self.debt);
        let cash_formatted = Self::format_number(self.cash);

        println!("\n");
        println!(
            "DATE {}                        HOLD  \x1B[33m{}\x1B[0m",
            game_date, self.hold
        );
        println!(
            "DAYS LEFT      \x1B[35m{:02}\x1B[0m                        \x1B[34m{}\x1B[0m\n",
            self.days_left, self.geo
        );

        println!("  ┌─────────────────────────┬─────────────────────────┐");
        println!("  │        STASH            │       TRENCH COAT       │");
        println!("  ├─────────────────────────┼─────────────────────────┤");
        for drug in [
            Drug::Cocaine,
            Drug::Heroin,
            Drug::Acid,
            Drug::Weed,
            Drug::Speed,
            Drug::Ludes,
        ]
        .iter()
        {
            let stash_amount = self.stash.get(drug).unwrap_or(&0);
            let trench_amount = self.trench_coat.get(drug).unwrap_or(&0);
            println!(
                "  │ {:<9}  {:<6}       │ {:<9}  {:<6}       │",
                drug.as_str(),
                stash_amount,
                drug.as_str(),
                trench_amount
            );
        }
        println!("  │                         │                         │");
        println!(
            "  │ BANK       \x1B[36m{:<12}\x1B[0m │ GUNS       {:<6}       │",
            bank_formatted, self.guns
        );
        println!(
            "  │ DEBT       \x1B[35m{:<8}\x1B[0m     │ CASH       \x1B[32m{:<12}\x1B[0m │",
            debt_formatted, cash_formatted
        );
        println!("  └─────────────────────────┴─────────────────────────┘\n");
    }

    fn show_prices(&self) {
        println!("HEY DUDE, THE PRICES OF DRUGS HERE ARE:");
        println!();
        println!(
            "    COCAINE    {:<11}    WEED       {:<11}",
            Self::format_number(*self.prices.get(&Drug::Cocaine).unwrap()),
            Self::format_number(*self.prices.get(&Drug::Weed).unwrap())
        );
        println!(
            "    HEROIN     {:<11}    SPEED      {:<11}",
            Self::format_number(*self.prices.get(&Drug::Heroin).unwrap()),
            Self::format_number(*self.prices.get(&Drug::Speed).unwrap())
        );
        println!(
            "    ACID       {:<11}    LUDES      {:<11}",
            Self::format_number(*self.prices.get(&Drug::Acid).unwrap()),
            Self::format_number(*self.prices.get(&Drug::Ludes).unwrap())
        );
        println!();
    }

    fn yn_prompt<F1, F2>(&mut self, prompt: &str, yes_action: F1, no_action: F2)
    where
        F1: FnOnce(&mut GameState),
        F2: FnOnce(&mut GameState),
    {
        print!("{prompt}");
        io::stdout().flush().unwrap();
        loop {
            let reply = self.getch().unwrap();
            match reply {
                'y' | 'Y' => {
                    yes_action(self);
                    break;
                }
                'n' | 'N' => {
                    no_action(self);
                    break;
                }
                _ => {
                    continue;
                }
            }
        }
    }

    fn loan(&mut self) {
        self.hud();
        self.yn_prompt(
            "DO YOU WANT TO VISIT THE LOAN SHARK? ",
            GameState::repay,
            GameState::stash,
        );
    }

    fn repay(&mut self) {
        self.hud();
        print!("HOW MUCH TO REPAY? ");
        io::stdout().flush().unwrap();
        let amount = self.read_number_input();
        if amount == 0 {
            self.borrow();
        } else if amount > self.cash || amount > self.debt {
            self.repay();
        } else {
            self.cash -= amount;
            self.debt -= amount;
            self.borrow();
        }
    }

    fn borrow(&mut self) {
        self.hud();
        print!("HOW MUCH TO BORROW? ");
        io::stdout().flush().unwrap();
        let amount = self.read_number_input();
        if amount == 0 {
            self.stash();
        } else if (amount + self.debt) <= self.maxloan {
            self.debt += amount;
            self.cash += amount;
            self.stash();
        } else {
            println!("YOU THINK HE IS CRAZY MAN !!!");
            thread::sleep(Duration::from_secs(1));
            self.borrow();
        }
    }

    fn stash(&mut self) {
        self.hud();
        self.yn_prompt(
            "DO YOU WISH TO TRANSFER DRUGS TO YOUR STASH? ",
            GameState::stashing,
            GameState::banking,
        );
    }

    fn stashing(&mut self) {
        self.hud();
        print!("WHICH DRUG DO YOU WANT TO STASH OR TAKE? ");
        io::stdout().flush().unwrap();
        let reply = self.getch().unwrap().to_lowercase().next().unwrap();
        println!("{reply}");
        if let Some(drug) = GameState::get_drug_from_char(reply) {
            self.stash_deposit(drug);
        } else {
            self.stash();
        }
    }

    fn stash_deposit(&mut self, drug: Drug) {
        self.hud();
        print!("HOW MUCH {} DO YOU WANT TO STASH? ", drug.as_str());
        io::stdout().flush().unwrap();
        let amount = self.read_number_input();

        if amount != 0 {
            let trench_amount_value = *self.trench_coat.get(&drug).unwrap_or(&0);
            if amount > 0 && amount <= trench_amount_value {
                if let Some(trench_amount) = self.trench_coat.get_mut(&drug) {
                    *trench_amount -= amount;
                }
                if let Some(stash_amount) = self.stash.get_mut(&drug) {
                    *stash_amount += amount;
                }
                self.hold += amount;
            } else {
                self.stash();
                return;
            }
        }

        self.hud();
        print!("HOW MUCH {} DO YOU WANT TO TAKE? ", drug.as_str());
        io::stdout().flush().unwrap();
        let amount = self.read_number_input();

        if amount != 0 {
            let stash_amount_value = *self.stash.get(&drug).unwrap_or(&0);
            if amount > 0 && amount <= stash_amount_value && self.hold - amount >= 0 {
                if let Some(stash_amount) = self.stash.get_mut(&drug) {
                    *stash_amount -= amount;
                }
                if let Some(trench_amount) = self.trench_coat.get_mut(&drug) {
                    *trench_amount += amount;
                }
                self.hold -= amount;
            } else {
                self.stash();
                return;
            }
        }

        self.banking();
    }

    fn banking(&mut self) {
        self.hud();
        self.yn_prompt(
            "DO YOU WISH TO VISIT THE BANK? ",
            GameState::visit_bank,
            GameState::main_menu,
        );
    }

    fn visit_bank(&mut self) {
        self.hud();
        print!("HOW MUCH TO DEPOSIT? ");
        io::stdout().flush().unwrap();
        let amount = self.read_number_input();
        if amount > 0 && amount <= self.cash {
            self.bank += amount;
            self.cash -= amount;
        }
        self.hud();
        print!("HOW MUCH TO WITHDRAW? ");
        io::stdout().flush().unwrap();
        let amount = self.read_number_input();
        if amount > 0 && amount <= self.bank {
            self.bank -= amount;
            self.cash += amount;
        }
        self.main_menu();
    }

    fn main_menu(&mut self) {
        self.hud();
        self.show_prices();
        if self.state == "begin" {
            self.state = "normal".to_string();
            if self.day == 0 {
                self.loan();
            }
        } else if self.state == "BRONXDO" {
            self.state = "normal".to_string();
            self.loan();
        } else if self.state == "normal" {
            self.buy_sell_jet();
        }
    }

    fn buy_sell_jet(&mut self) {
        print!("WILL YOU BUY, SELL OR JET? ");
        io::stdout().flush().unwrap();
        loop {
            let reply = self.getch().unwrap();
            match reply {
                'b' | 'B' => {
                    println!();
                    self.buying();
                    break;
                }
                's' | 'S' => {
                    println!();
                    self.selling();
                    break;
                }
                'j' | 'J' => {
                    println!();
                    self.jet();
                    break;
                }
                _ => {
                    continue;
                }
            }
        }
    }

    fn buying(&mut self) {
        print!("WHAT WILL YOU BUY? ");
        io::stdout().flush().unwrap();
        let reply = self.getch().unwrap().to_lowercase().next().unwrap();
        println!("{reply}");
        if let Some(drug) = GameState::get_drug_from_char(reply) {
            self.buy_drug(drug);
        } else {
            self.main_menu();
        }
    }

    fn buy_drug(&mut self, drug: Drug) {
        self.hud();
        self.show_prices();

        let price = *self.prices.get(&drug).unwrap_or(&0);
        let afford = if price > 0 {
            self.cash / price
        } else {
            0
        };

        println!("YOU CAN AFFORD ( {afford} )");
        print!("HOW MUCH {} DO YOU WANT TO BUY? ", drug.as_str());
        io::stdout().flush().unwrap();
        let amount = self.read_number_input();

        if amount == 0 {
            self.main_menu();
        } else if amount > 0 && amount <= afford && (self.hold - amount) >= 0 {
            if let Some(entry) = self.trench_coat.get_mut(&drug) {
                *entry += amount;
            }
            self.cash -= amount * price;
            self.hold -= amount;
            self.main_menu();
        } else {
            self.main_menu();
        }
    }

    fn selling(&mut self) {
        print!("WHAT WILL YOU SELL? ");
        io::stdout().flush().unwrap();
        let reply = self.getch().unwrap().to_lowercase().next().unwrap();
        println!("{reply}");
        if let Some(drug) = GameState::get_drug_from_char(reply) {
            self.sell_drug(drug);
        } else {
            self.main_menu();
        }
    }

    fn sell_drug(&mut self, drug: Drug) {
        self.hud();
        self.show_prices();

        let price = *self.prices.get(&drug).unwrap_or(&0);
        let trench_amount_value = *self.trench_coat.get(&drug).unwrap_or(&0);

        println!("YOU CAN SELL ( {trench_amount_value} )");
        print!("HOW MUCH {} DO YOU WANT TO SELL? ", drug.as_str());
        io::stdout().flush().unwrap();
        let amount = self.read_number_input();

        if amount == 0 {
            self.main_menu();
        } else if amount > 0 && amount <= trench_amount_value {
            if let Some(trench_amount) = self.trench_coat.get_mut(&drug) {
                *trench_amount -= amount;
            }
            self.cash += amount * price;
            self.hold += amount;
            self.main_menu();
        } else {
            self.main_menu();
        }
    }

    fn jet(&mut self) {
        self.hud();
        println!();
        println!("   1) BRONX        2) GHETTO          3) CENTRAL PARK");
        println!("   4) MANHATTAN    5) CONEY ISLAND    6) BROOKLYN");
        println!();
        print!("WHERE TO DUDE: ");
        io::stdout().flush().unwrap();
        let reply = self.getch().unwrap();
        println!("{reply}");
        match reply {
            '1' => {
                self.geo = "BRONX".to_string();
                self.state = "BRONXDO".to_string();
                self.new_day();
            }
            '2' => {
                self.geo = "GHETTO".to_string();
                self.new_day();
            }
            '3' => {
                self.geo = "CENTRAL PARK".to_string();
                self.new_day();
            }
            '4' => {
                self.geo = "MANHATTAN".to_string();
                self.new_day();
            }
            '5' => {
                self.geo = "CONEY ISLAND".to_string();
                self.new_day();
            }
            '6' => {
                self.geo = "BROOKLYN".to_string();
                self.new_day();
            }
            _ => self.main_menu(),
        }
    }

    fn new_day(&mut self) {
        self.hud();
        self.days_left -= 1;
        if self.days_left <= 0 {
            self.you_win();
        } else {
            self.day += 1;
            self.roll_prices();
            self.debt = self.debt * 110 / 100;
            self.bank = self.bank * 105 / 100;
            self.roll_event();
            self.roll_fight();
            self.main_menu();
        }
    }

    fn you_win(&self) {
        clear_screen();
        let total_money = self.bank + self.cash - self.debt;
        let total_money_display = Self::format_number(total_money);

        let score = if total_money > 50_000_000 {
            100
        } else if total_money >= 25_000_000 {
            99
        } else if total_money >= 10_000_000 {
            98
        } else {
            let calculated_score = (total_money * 100) / 10_000_000;
            calculated_score.clamp(0, 97)
        };

        println!(
            "\x1B[38;2;255;202;128mGAME OVER\x1B[0m\nYOU SURVIVED FOR \x1B[33m{}\x1B[0m DAYS!",
            self.day
        );
        println!("YOUR TOTAL MONEY: \x1B[32m{}\x1B[0m", total_money_display);
        println!("YOUR SCORE: \x1B[35m{}\x1B[0m OUT OF 100", score);

        if score == 100 {
            println!("DEALER RANK: \x1B[36mGANGSTA MOTHERFUCKER ... YOU ARE MY HERO\x1B[0m");
        } else if score == 99 {
            println!("DEALER RANK: \x1B[36mHUSTLER FUCKER ... YOU THA DOPE MAN\x1B[0m");
        } else if score == 98 {
            println!("DEALER RANK: \x1B[36mPABLO ESCOBAR ... YOU ARE A GOD\x1B[0m");
        } else if (76..=97).contains(&score) {
            println!("DEALER RANK: \x1B[36mKINGPIN ... GOD DAMN\x1B[0m");
        } else if (51..=75).contains(&score) {
            println!("DEALER RANK: \x1B[36mRUN THE TOWN ... PRETTY GOOD\x1B[0m");
        } else if (31..=50).contains(&score) {
            println!("DEALER RANK: \x1B[36mOWN THE BLOCK ... NOT BAD\x1B[0m");
        } else if score <= 30 {
            println!("DEALER RANK: \x1B[36mSMALL TIME PUSHA ... WEAK\x1B[0m");
        }
        process::exit(0);
    }

    fn roll_fight(&mut self) {
        let mut rng = rand::thread_rng();
        let fight_chance = rng.gen_range(1..=100) / (self.hold + 1);
        if fight_chance >= 1 {
            self.cops = fight_chance / 9 + 2;
            self.hud();
            print!(
                "\x1B[31mOFFICER HARDASS AND {} OF HIS DEPUTIES ARE CHASING YOU !!!!!\x1B[0m",
                self.cops
            );
            io::stdout().flush().unwrap();
            self.wait_for_key_press();
            self.fight();
        }
    }

    fn fight_hud(&mut self) {
        self.term_info();
        clear_screen();
        println!();
        let bar = "█".repeat(self.width);
        println!("\x1B[35m{}\x1B[0m", bar);
        println!();
        println!(
            "{}DAMAGE    \x1B[35m{}\x1B[0m       COPS    \x1B[36m{}\x1B[0m       GUNS    \x1B[34m{}\x1B[0m",
            " ".repeat(self.width / 8),
            self.damage,
            self.cops,
            self.guns
        );
        println!();
        println!("\x1B[35m{}\x1B[0m", bar);
        println!("\n\n");
    }

    fn fight(&mut self) {
        self.term_info();
        self.fight_hud();

        if self.damage >= 50 {
            print!("\x1B[31mTHEY WASTED YOU MAN !! WHAT A DRAG !!!\x1B[0m ");
            io::stdout().flush().unwrap();
            self.wait_for_key_press();
            self.you_win();
        }

        if self.guns == 0 {
            print!("WILL YOU RUN? ");
            io::stdout().flush().unwrap();
            let reply = self.getch().unwrap().to_lowercase().next().unwrap();
            println!("{reply}");
            if reply == 'r' || reply == 'y' {
                let mut rng = rand::thread_rng();
                let getaway = rng.gen_range(1..=2);
                if getaway == 1 {
                    self.fight_hud();
                    print!("\x1B[36mYOU LOST THEM IN THE ALLEYS !!\x1B[0m ");
                    io::stdout().flush().unwrap();
                    self.wait_for_key_press();
                    self.check_doctor();
                } else {
                    self.fight_hud();
                    self.firing_on_you();
                }
            } else {
                self.fight();
            }
        } else {
            print!("WILL YOU RUN OR FIGHT? ");
            io::stdout().flush().unwrap();
            let reply = self.getch().unwrap().to_lowercase().next().unwrap();
            println!("{reply}");
            if reply == 'r' {
                let mut rng = rand::thread_rng();
                let getaway = rng.gen_range(1..=2);
                if getaway == 1 {
                    self.fight_hud();
                    print!("\x1B[36mYOU LOST THEM IN THE ALLEYS !!\x1B[0m ");
                    io::stdout().flush().unwrap();
                    self.wait_for_key_press();
                    self.check_doctor();
                } else {
                    self.fight_hud();
                    self.firing_on_you();
                }
            } else if reply == 'f' {
                self.fight_hud();
                print!("YOU'RE FIRING ON THEM!! ");
                io::stdout().flush().unwrap();
                self.wait_for_key_press();
                let mut rng = rand::thread_rng();
                let kill_them = rng.gen_range(0..=self.guns * 2);
                if kill_them == 0 {
                    self.fight_hud();
                    print!("YOU MISSED THEM !! ");
                    io::stdout().flush().unwrap();
                    self.wait_for_key_press();
                    self.firing_on_you();
                } else {
                    self.fight_hud();
                    self.cops -= 1;
                    if self.cops <= 0 {
                        self.fight_hud();
                        print!("\x1B[32mYOU KILLED ALL OF THEM!!!!\x1B[0m ");
                        io::stdout().flush().unwrap();
                        self.wait_for_key_press();
                        self.fight_reward();
                    } else {
                        print!("\x1B[33mYOU KILLED ONE!!\x1B[0m ");
                        io::stdout().flush().unwrap();
                        self.wait_for_key_press();
                        self.firing_on_you();
                    }
                }
            } else {
                self.fight();
            }
        }
    }

    fn firing_on_you(&mut self) {
        self.fight_hud();
        print!("THEY ARE FIRING ON YOU MAN !! ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
        let mut rng = rand::thread_rng();
        let damage_hit = rng.gen_range(0..=3) * self.cops - rng.gen_range(2..=18);
        if damage_hit <= 0 {
            self.fight_hud();
            print!("THEY MISSED !! ");
            io::stdout().flush().unwrap();
            self.wait_for_key_press();
            self.fight();
        } else {
            self.damage += damage_hit;
            self.fight_hud();
            print!("\x1B[31mYOU'VE BEEN HIT !!\x1B[0m ");
            io::stdout().flush().unwrap();
            self.wait_for_key_press();
            self.fight();
        }
    }

    fn check_doctor(&mut self) {
        if self.damage > 10 {
            self.doctor();
        } else {
            self.main_menu();
        }
    }

    fn doctor(&mut self) {
        self.fight_hud();
        let mut rng = rand::thread_rng();
        let doc_price_multiplier = rng.gen_range(200..=1000);
        let total_cost = self.damage * doc_price_multiplier / 10;

        self.yn_prompt(
            &format!(
                "\x1B[36mWILL YOU PAY {} DOLLARS TO HAVE A DOCTOR SEW YOU UP?\x1B[0m ",
                total_cost
            ),
            |s| {
                if s.cash >= total_cost {
                    s.cash -= total_cost;
                    s.damage = 0;
                }
                s.main_menu();
            },
            GameState::main_menu,
        );
    }

    fn fight_reward(&mut self) {
        self.fight_hud();
        let mut rng = rand::thread_rng();
        let fight_reward = rng.gen_range(200..=1000);
        self.cash += fight_reward;

        print!(
            "\x1B[32mYOU FOUND {} DOLLARS ON OFFICER HARDASS' CARCASS !!\x1B[0m ",
            fight_reward
        );
        io::stdout().flush().unwrap();
        self.wait_for_key_press();

        self.check_doctor();
    }

    fn roll_event(&mut self) {
        let mut possible_events: Vec<fn(&mut GameState)> = Vec::new();

        if *self.trench_coat.get(&Drug::Weed).unwrap_or(&0) > 1 {
            possible_events.push(GameState::brownies);
        }
        if self.hold < 32 {
            possible_events.push(GameState::policedogs);
        }
        if self.hold > 32 {
            possible_events.push(GameState::finddrugs);
        }

        possible_events.push(GameState::paraquat);
        possible_events.push(GameState::mugged);
        possible_events.push(GameState::cokebust);
        possible_events.push(GameState::addicts);
        possible_events.push(GameState::weedbottomout);
        possible_events.push(GameState::coatsale);
        possible_events.push(GameState::cheapheroin);
        possible_events.push(GameState::cheapcocaine);
        possible_events.push(GameState::cheapludes);
        possible_events.push(GameState::cheapacid);
        possible_events.push(GameState::gunsale);

        let mut rng = rand::thread_rng();
        let event = possible_events[rng.gen_range(0..possible_events.len())];

        event(self);
    }

    fn brownies(&mut self) {
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(1..=5);
        let trench_weed = self.trench_coat.get_mut(&Drug::Weed).unwrap();
        let dropped = *trench_weed / n;
        self.hold += dropped;
        *trench_weed -= dropped;
        self.hud();
        print!("\x1B[31mYOUR MAMA MADE SOME BROWNIES AND USED YOUR WEED !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
        print!("\x1B[31mTHEY WERE GREAT !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn paraquat(&mut self) {
        self.hud();
        println!("\x1B[35mTHERE IS SOME WEED THAT SMELLS LIKE PARAQUAT HERE !! IT LOOKS GOOD !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.yn_prompt(
            "\x1B[35mWILL YOU SMOKE IT? \x1B[0m",
            |s| {
                s.hud();
                print!("\x1B[38;2;121;112;169mYOU HALLUCINATE FOR THREE DAYS ON THE WILDEST TRIP YOU EVER IMAGINED !!\x1B[0m ");
                io::stdout().flush().unwrap();
                s.wait_for_key_press();
                print!("\x1B[38;2;121;112;169mTHEN YOU DIE BECAUSE YOUR BRAIN HAS DISINTEGRATED !!\x1B[0m ");
                io::stdout().flush().unwrap();
                s.wait_for_key_press();
                s.you_win();
            },
            GameState::main_menu,
        );
    }

    fn mugged(&mut self) {
        self.cash = self.cash * 4 / 5;
        self.hud();
        print!("\x1B[31mYOU WERE MUGGED IN THE SUBWAY !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn cokebust(&mut self) {
        self.hud();
        let coke_price = self.prices.get_mut(&Drug::Cocaine).unwrap();
        *coke_price *= 6;
        print!("\x1B[36mCOPS MADE A BIG COKE BUST !! PRICES ARE OUTRAGEOUS !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn addicts(&mut self) {
        self.hud();
        let heroin_price = self.prices.get_mut(&Drug::Heroin).unwrap();
        *heroin_price *= 6;
        print!("\x1B[36mADDICTS ARE BUYING HEROIN AT OUTRAGEOUS PRICES !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn weedbottomout(&mut self) {
        self.hud();
        let weed_price = self.prices.get_mut(&Drug::Weed).unwrap();
        *weed_price /= 5;
        print!("\x1B[33mCOLOMBIAN FREIGHTER DUSTED THE COAST GUARD !!  WEED PRICES HAVE BOTTOMED OUT !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn policedogs(&mut self) {
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(2..=5);

        for drug in [
            Drug::Cocaine,
            Drug::Heroin,
            Drug::Acid,
            Drug::Weed,
            Drug::Speed,
            Drug::Ludes,
        ]
        .iter()
        {
            let trench_amount = self.trench_coat.get_mut(drug).unwrap();
            let amount = *trench_amount;
            let dropped = amount / n;
            *trench_amount -= dropped;
            self.hold += dropped;
        }

        self.hud();
        print!(
            "\x1B[31mPOLICE DOGS CHASE YOU {} BLOCKS !!\x1B[0m ",
            n
        );
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
        print!(
            "\x1B[31mYOU DROPPED SOME DRUGS !! THAT'S A DRAG MAN !!\x1B[0m "
        );
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn cheapcocaine(&mut self) {
        self.hud();
        let coke_price = self.prices.get_mut(&Drug::Cocaine).unwrap();
        *coke_price /= 6;
        print!("\x1B[32mPIGS ARE SELLING CHEAP COCAINE FROM LAST WEEKS RAID !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn cheapheroin(&mut self) {
        self.hud();
        let heroin_price = self.prices.get_mut(&Drug::Heroin).unwrap();
        *heroin_price /= 6;
        print!("\x1B[32mPIGS ARE SELLING CHEAP HEROIN FROM LAST WEEKS RAID !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn finddrugs(&mut self) {
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(1..=32);
        let drug = match rng.gen_range(1..=6) {
            1 => Drug::Cocaine,
            2 => Drug::Heroin,
            3 => Drug::Acid,
            4 => Drug::Weed,
            5 => Drug::Speed,
            6 => Drug::Ludes,
            _ => Drug::Cocaine,
        };

        let trench_amount = self.trench_coat.get_mut(&drug).unwrap();
        *trench_amount += n;
        self.hold -= n;
        self.hud();
        print!(
            "\x1B[36mYOU FIND {} UNITS OF {} ON A DEAD DUDE IN THE SUBWAY !!\x1B[0m ",
            n,
            drug.as_str()
        );
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn cheapludes(&mut self) {
        self.hud();
        let ludes_price = self.prices.get_mut(&Drug::Ludes).unwrap();
        *ludes_price /= 6;
        print!("\x1B[33mRIVAL DRUG DEALERS RAIDED A PHARMACY AND ARE SELLING CHEAP LUDES !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn cheapacid(&mut self) {
        self.hud();
        let acid_price = self.prices.get_mut(&Drug::Acid).unwrap();
        *acid_price /= 10;
        print!("\x1B[33mTHE MARKET HAS BEEN FLOODED WITH CHEAP HOME MADE ACID !!\x1B[0m ");
        io::stdout().flush().unwrap();
        self.wait_for_key_press();
    }

    fn gunsale(&mut self) {
        self.hud();
        let gunstock = ["RUGER", ".38 SPECIAL", "SATURDAY NIGHT SPECIAL", "BARETTA"];
        let mut rng = rand::thread_rng();
        let shufguns = rng.gen_range(0..gunstock.len());
        self.gunprice = rng.gen_range(250..=500);
        print!(
            "\x1B[34mWILL YOU BUY A {} FOR {}?\x1B[0m ",
            gunstock[shufguns], self.gunprice
        );
        io::stdout().flush().unwrap();
        self.yn_prompt(
            "",
            |s| {
                if s.cash >= s.gunprice {
                    s.cash -= s.gunprice;
                    s.guns += 1;
                }
            },
            |_| {},
        );
    }

    fn coatsale(&mut self) {
        self.hud();
        let mut rng = rand::thread_rng();
        self.coatspace = rng.gen_range(32..=64);
        self.coatprice = rng.gen_range(150..=400);
        print!(
            "\x1B[38;2;255;202;128mWILL YOU BUY A NEW TRENCH COAT WITH MORE POCKETS FOR {}?\x1B[0m ",
            self.coatprice
        );
        io::stdout().flush().unwrap();
        self.yn_prompt(
            "",
            |s| {
                if s.cash >= s.coatprice {
                    s.cash -= s.coatprice;
                    s.hold += s.coatspace;
                }
            },
            |_| {},
        );
    }

    fn get_drug_from_char(c: char) -> Option<Drug> {
        match c.to_lowercase().next().unwrap() {
            'c' => Some(Drug::Cocaine),
            'h' => Some(Drug::Heroin),
            'a' => Some(Drug::Acid),
            'w' => Some(Drug::Weed),
            's' => Some(Drug::Speed),
            'l' => Some(Drug::Ludes),
            _ => None,
        }
    }

    fn read_number_input(&self) -> i64 {
        let mut reply = String::new();
        io::stdin().read_line(&mut reply).unwrap();
        reply.trim().parse().unwrap_or(0)
    }

    fn getch(&mut self) -> io::Result<char> {
        #[cfg(unix)]
        {
            use std::io::Read;

            let stdin = io::stdin();
            let mut handle = stdin.lock();

            let mut termios = tcgetattr(STDIN_FILENO).unwrap();
            let old_termios = termios.clone();

            cfmakeraw(&mut termios);
            tcsetattr(STDIN_FILENO, SetArg::TCSANOW, &termios).unwrap();

            let mut buffer = [0; 1];
            let res = handle.read_exact(&mut buffer);

            tcsetattr(STDIN_FILENO, SetArg::TCSANOW, &old_termios).unwrap();

            match res {
                Ok(()) => {
                    if buffer[0] == 3 {
                        self.you_win();
                        process::exit(0);
                    } else {
                        Ok(buffer[0] as char)
                    }
                }
                Err(e) => Err(e),
            }
        }

        #[cfg(windows)]
        #[allow(unused_imports)]
        {
            use winapi::um::consoleapi::ReadConsoleA;
            use winapi::um::winnt::CHAR;

            unsafe {
                let handle = GetStdHandle(STD_INPUT_HANDLE);
                if handle == INVALID_HANDLE_VALUE {
                    return Err(io::Error::last_os_error());
                }

                let mut mode: DWORD = 0;
                if GetConsoleMode(handle, &mut mode) == 0 {
                    return Err(io::Error::last_os_error());
                }

                let new_mode = mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
                if SetConsoleMode(handle, new_mode) == 0 {
                    return Err(io::Error::last_os_error());
                }

                let mut buffer = [0u8; 1];
                let mut bytes_read = 0;
                let result = winapi::um::fileapi::ReadFile(
                    handle,
                    buffer.as_mut_ptr() as *mut _,
                    1,
                    &mut bytes_read,
                    std::ptr::null_mut(),
                );

                SetConsoleMode(handle, mode);

                if result == 0 {
                    return Err(io::Error::last_os_error());
                }

                if buffer[0] == 3 {
                    self.you_win();
                    process::exit(0);
                } else {
                    Ok(buffer[0] as char)
                }
            }
        }
    }

    fn wait_for_key_press(&mut self) {
        let _ = self.getch();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        println!("drugwars.rs {VERSION}");
        process::exit(0);
    }

    let mut game = GameState::new();
    game.roll_prices();
    game.start_game();
}
