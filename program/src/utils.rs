use borsh::{BorshDeserialize, BorshSerialize};
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct BetAcc {
    pub sport: u8,
    pub league: u32,
    pub event: u64,
    pub period: u8,  //need rules for this, 0,, 1, 2...
    pub mkt: u16, //0 for moneyline, 1 for home, 2 for away, 3 for draw, 200 + spread * 2, 1000 + total * 2
    pub player: u32, //4 bytes of first initial, last initial, 2nd letter of last name, 3rd letter of last name, count spaces (ex de xxx), if blank set to 0
    pub stake0: u64,
    pub stake1: u64,
    pub wallet0: [u8; 32],
    pub wallet1: [u8; 32],
    pub rent_payer: [u8; 32],
    pub is_free_bet: bool,
    pub placed_at: u64, //https://stackoverflow.com/questions/72223450/how-to-get-the-current-time-in-solana-program-without-using-any-external-systemp
    pub to_aggregate: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CancelDelay {
    pub is_real: bool,
    pub seconds: u8,
}

pub const EMPTY_WALLET: [u8; 32] = [0; 32];

pub const POOL: [u8; 32] = [
    36, 72, 94, 114, 132, 225, 24, 60, 198, 3, 99, 170, 222, 13, 99, 85, 216, 113, 112, 141, 102,
    74, 146, 96, 56, 36, 11, 95, 123, 3, 27, 194,
];

pub const PBMM: [u8; 32] = [
    5, 202, 24, 56, 131, 111, 203, 156, 106, 68, 185, 161, 229, 194, 11, 95, 141, 149, 209, 42, 8,
    93, 165, 215, 83, 97, 15, 14, 27, 207, 86, 178,
];

pub const ADMIN:[u8; 32] = [232, 166, 95, 126, 248, 155, 162, 93, 189, 238, 126, 247, 103, 87, 122, 15, 74, 245, 250, 181, 251, 116, 215, 190, 226, 34, 136, 11, 108, 33, 242, 149];

pub fn equal_wallets(wallet1: [u8; 32], wallet2: [u8; 32]) -> bool {
    for i in 0..32 {
        if wallet1[i] != wallet2[i] {
            return false;
        }
    }
    return true;
}
pub fn correct_pool(wallet: [u8; 32]) -> bool {
    return equal_wallets(wallet, POOL);
}
pub fn blank_wallet(wallet: [u8; 32]) -> bool {
    return equal_wallets(wallet, EMPTY_WALLET);
}

pub fn bytes_to_num(data: &[u8], start: usize, end: usize) -> u64 {
    let mut output: u64 = 0;
    let max_pow = end - start;
    for i in 0..max_pow {
        output = output + data[start + i] as u64 * u64::pow(256, i as u32);
    }
    return output;
}

pub fn ids_match(bet_account: &BetAcc, instruction_data: &[u8]) -> bool {
    let sports_equal = instruction_data[0] == bet_account.sport;
    let leagues_equal = bytes_to_num(instruction_data, 1, 5) as u32 == bet_account.league;
    let events_equal = bytes_to_num(instruction_data, 5, 13) == bet_account.event;
    let periods_equal = instruction_data[13] == bet_account.period;
    let mkts_equal = bytes_to_num(instruction_data, 14, 16) as u16 == bet_account.mkt;
    let players_equal = bytes_to_num(instruction_data, 16, 20) as u32 == bet_account.player;
    return sports_equal
        && leagues_equal
        && events_equal
        && periods_equal
        && mkts_equal
        && players_equal;
}

pub fn blank_acc(bet_account: &BetAcc) -> bool {
    let wallet0_empty = blank_wallet(bet_account.wallet0);
    let wallet1_empty = blank_wallet(bet_account.wallet1);
    let rent_payer_empty = blank_wallet(bet_account.rent_payer);
    return wallet0_empty && wallet1_empty && rent_payer_empty;
}

pub fn set_bet_ids(mut bet_account: &mut BetAcc, instruction_data: &[u8]) {
    bet_account.sport = instruction_data[0];
    bet_account.league = bytes_to_num(instruction_data, 1, 5) as u32;
    bet_account.event = bytes_to_num(instruction_data, 5, 13);
    bet_account.period = instruction_data[13];
    bet_account.mkt = bytes_to_num(instruction_data, 14, 16) as u16;
    bet_account.player = bytes_to_num(instruction_data, 16, 20) as u32;
}
