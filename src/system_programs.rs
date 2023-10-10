use solana_program::pubkey::Pubkey;

// -----------------
// System Program List
// -----------------
pub const SYSTEM_PROGRAMS: [&str; 9] = [
    "11111111111111111111111111111111",
    "BPFLoaderUpgradeab1e11111111111111111111111",
    "BPFLoader2111111111111111111111111111111111",
    "Config1111111111111111111111111111111111111",
    "Feature111111111111111111111111111111111111",
    "NativeLoader1111111111111111111111111111111",
    "Stake11111111111111111111111111111111111111",
    "Sysvar1111111111111111111111111111111111111",
    "Vote111111111111111111111111111111111111111",
];

pub fn is_system_program(program_id: &Pubkey) -> bool {
    SYSTEM_PROGRAMS.contains(&program_id.to_string().as_str())
}
