use solana_program::pubkey;
use solana_program::pubkey::Pubkey;

macro_rules! define_token {
    ($mod_name:ident, $mint:expr, $decimals:expr, $price_feed:expr) => {
        pub mod $mod_name {
            use super::*;

            pub const MINT: Pubkey = pubkey!($mint);
            pub const DECIMALS: u8 = $decimals;
            pub const PRICE_FEED: [u8; 32] = $price_feed;
        }
    };
}

define_token!(
    usdc,
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    6,
    [
        234, 160, 32, 198, 28, 196, 121, 113, 40, 19, 70, 28, 225, 83, 137, 74, 150, 166, 192, 11,
        33, 237, 12, 252, 39, 152, 209, 249, 169, 233, 201, 74,
    ]
);

define_token!(
    usdt,
    "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
    6,
    [
        43, 137, 185, 220, 143, 223, 159, 52, 112, 154, 91, 16, 107, 71, 47, 15, 57, 187, 108, 169,
        206, 4, 176, 253, 127, 46, 151, 22, 136, 226, 229, 59,
    ]
);

define_token!(
    sol,
    "So11111111111111111111111111111111111111112",
    9,
    [
        239, 13, 139, 111, 218, 44, 235, 164, 29, 161, 93, 64, 149, 209, 218, 57, 42, 13, 47, 142,
        208, 198, 199, 188, 15, 76, 250, 200, 194, 128, 181, 109,
    ]
);

define_token!(
    fida,
    "EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp",
    6,
    [
        200, 6, 87, 183, 246, 243, 234, 194, 114, 24, 208, 157, 90, 78, 84, 228, 123, 37, 118, 141,
        159, 94, 16, 172, 21, 254, 44, 249, 0, 136, 20, 0,
    ]
);

define_token!(
    msol,
    "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
    9,
    [
        194, 40, 154, 106, 67, 210, 206, 145, 198, 245, 92, 174, 195, 112, 244, 172, 195, 138, 46,
        212, 119, 245, 136, 19, 51, 76, 109, 3, 116, 159, 242, 164,
    ]
);

define_token!(
    bonk,
    "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
    5,
    [
        114, 176, 33, 33, 124, 163, 254, 104, 146, 42, 25, 170, 249, 144, 16, 156, 185, 216, 78,
        154, 208, 4, 180, 210, 2, 90, 214, 245, 41, 49, 68, 25,
    ]
);

define_token!(
    bat,
    "EPeUFDgHRxs9xxEPVaL6kfGQvCon7jmAWKVUHuux1Tpz",
    8,
    [
        142, 134, 15, 183, 78, 96, 229, 115, 107, 69, 93, 130, 246, 11, 55, 40, 4, 156, 52, 142,
        148, 150, 26, 221, 95, 150, 27, 2, 253, 238, 37, 53,
    ]
);

define_token!(
    pyth,
    "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3",
    6,
    [
        11, 191, 40, 233, 168, 65, 161, 204, 120, 143, 106, 54, 27, 23, 202, 7, 45, 14, 163, 9,
        138, 30, 93, 241, 195, 146, 45, 6, 113, 149, 121, 255,
    ]
);

define_token!(
    bsol,
    "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1",
    9,
    [
        137, 135, 83, 121, 231, 15, 143, 186, 220, 23, 174, 243, 21, 173, 243, 168, 213, 209, 96,
        184, 17, 67, 85, 55, 224, 60, 151, 232, 170, 201, 125, 156,
    ]
);

define_token!(
    inj,
    "6McPRfPV6bY1e9hLxWyG54W9i9Epq75QBvXg2oetBVTB",
    9,
    [
        122, 91, 193, 210, 181, 106, 208, 41, 4, 140, 214, 57, 100, 179, 173, 39, 118, 234, 223,
        129, 46, 220, 26, 67, 163, 20, 6, 203, 84, 191, 245, 146,
    ]
);
