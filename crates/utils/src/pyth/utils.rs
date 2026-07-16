use std::convert::TryInto;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{clock::Clock, pubkey::Pubkey};

macro_rules! check {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}

pub type FeedId = [u8; 32];

pub struct Price {
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VerificationLevel {
    Partial {
        #[allow(unused)]
        num_signatures: u8,
    },
    Full,
}

impl VerificationLevel {
    /// Compare two `VerificationLevel`.
    /// `Full` is always greater than `Partial`, and `Partial` with more signatures is greater than `Partial` with fewer signatures.
    pub fn gte(&self, other: VerificationLevel) -> bool {
        match self {
            VerificationLevel::Full => true,
            VerificationLevel::Partial { num_signatures } => match other {
                VerificationLevel::Full => false,
                VerificationLevel::Partial {
                    num_signatures: other_num_signatures,
                } => *num_signatures >= other_num_signatures,
            },
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetPriceError {
    #[error("This price feed update's age exceeds the requested maximum age")]
    PriceTooOld = 10000, // Big number to avoid conflicts with the SDK user's program error codes
    #[error("This TWAP update's window size is invalid")]
    InvalidWindowSize,
    #[error("The price feed update doesn't match the requested feed id")]
    MismatchedFeedId,
    #[error("This price feed update has a lower verification level than the one requested")]
    InsufficientVerificationLevel,
    #[error("Feed id must be 32 Bytes, that's 64 hex characters or 66 with a 0x prefix")]
    FeedIdMustBe32Bytes,
    #[error("Feed id contains non-hex characters")]
    FeedIdNonHexCharacter,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct PriceFeedMessage {
    /// `FeedId` but avoid the type alias because of compatibility issues with Anchor's `idl-build` feature.
    pub feed_id: [u8; 32],
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    /// The timestamp of this price update in seconds
    pub publish_time: i64,
    /// The timestamp of the previous price update. This field is intended to allow users to
    /// identify the single unique price update for any moment in time:
    /// for any time t, the unique update is the one such that prev_publish_time < t <= publish_time.
    ///
    /// Note that there may not be such an update while we are migrating to the new message-sending logic,
    /// as some price updates on pythnet may not be sent to other chains (because the message-sending
    /// logic may not have triggered). We can solve this problem by making the message-sending mandatory
    /// (which we can do once publishers have migrated over).
    ///
    /// Additionally, this field may be equal to publish_time if the message is sent on a slot where
    /// where the aggregation was unsuccesful. This problem will go away once all publishers have
    /// migrated over to a recent version of pyth-agent.
    pub prev_publish_time: i64,
    pub ema_price: i64,
    pub ema_conf: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct PriceUpdateV2 {
    pub write_authority: Pubkey,
    pub verification_level: VerificationLevel,
    pub price_message: PriceFeedMessage,
    pub posted_slot: u64,
}

impl PriceUpdateV2 {
    /// Get a `Price` from a `PriceUpdateV2` account for a given `FeedId`.
    ///
    /// # Warning
    /// This function does not check :
    /// - How recent the price is
    /// - Whether the price update has been verified
    ///
    /// It is therefore unsafe to use this function without any extra checks, as it allows for the possibility of using unverified or outdated price updates.
    pub fn get_price_unchecked(
        &self,
        feed_id: &FeedId,
    ) -> std::result::Result<Price, GetPriceError> {
        check!(
            self.price_message.feed_id == *feed_id,
            GetPriceError::MismatchedFeedId
        );
        Ok(Price {
            price: self.price_message.price,
            conf: self.price_message.conf,
            exponent: self.price_message.exponent,
            publish_time: self.price_message.publish_time,
        })
    }

    /// Get a `Price` from a `PriceUpdateV2` account for a given `FeedId` no older than `maximum_age` with customizable verification level.
    ///
    /// # Warning
    /// Lowering the verification level from `Full` to `Partial` increases the risk of using a malicious price update.
    /// Please read the documentation for [`VerificationLevel`] for more information.
    ///
    /// # Example
    /// ```ignore
    /// use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, VerificationLevel, PriceUpdateV2};
    /// use anchor_lang::prelude::*;
    ///
    /// const MAXIMUM_AGE : u64 = 30;
    /// const FEED_ID: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d"; // SOL/USD
    ///
    /// #[derive(Accounts)]
    /// #[instruction(amount_in_usd : u64)]
    /// pub struct ReadPriceAccount<'info> {
    ///     pub price_update: Account<'info, PriceUpdateV2>,
    /// }
    ///
    /// pub fn read_price_account(ctx : Context<ReadPriceAccount>) -> Result<()> {
    ///     let price_update = &mut ctx.accounts.price_update;
    ///     let price = price_update.get_price_no_older_than_with_custom_verification_level(&Clock::get()?, MAXIMUM_AGE, &get_feed_id_from_hex(FEED_ID)?, VerificationLevel::Partial{num_signatures: 5})?;
    ///     Ok(())
    /// }
    ///```
    pub fn get_price_no_older_than_with_custom_verification_level(
        &self,
        clock: &Clock,
        maximum_age: u64,
        feed_id: &FeedId,
        verification_level: VerificationLevel,
    ) -> std::result::Result<Price, GetPriceError> {
        check!(
            self.verification_level.gte(verification_level),
            GetPriceError::InsufficientVerificationLevel
        );
        let price = self.get_price_unchecked(feed_id)?;
        check!(
            price
                .publish_time
                .saturating_add(maximum_age.try_into().unwrap())
                >= clock.unix_timestamp,
            GetPriceError::PriceTooOld
        );
        Ok(price)
    }

    /// Get a `Price` from a `PriceUpdateV2` account for a given `FeedId` no older than `maximum_age` with `Full` verification.
    ///
    /// # Example
    /// ```ignore
    /// use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
    /// use anchor_lang::prelude::*;
    ///
    /// const MAXIMUM_AGE : u64 = 30;
    /// const FEED_ID: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d"; // SOL/USD
    ///
    /// #[derive(Accounts)]
    /// #[instruction(amount_in_usd : u64)]
    /// pub struct ReadPriceAccount<'info> {
    ///     pub price_update: Account<'info, PriceUpdateV2>,
    /// }
    ///
    /// pub fn read_price_account(ctx : Context<ReadPriceAccount>) -> Result<()> {
    ///     let price_update = &mut ctx.accounts.price_update;
    ///     let price = price_update.get_price_no_older_than(&Clock::get()?, MAXIMUM_AGE, &get_feed_id_from_hex(FEED_ID)?)?;
    ///     Ok(())
    /// }
    ///```
    pub fn get_price_no_older_than(
        &self,
        clock: &Clock,
        maximum_age: u64,
        feed_id: &FeedId,
    ) -> std::result::Result<Price, GetPriceError> {
        self.get_price_no_older_than_with_custom_verification_level(
            clock,
            maximum_age,
            feed_id,
            VerificationLevel::Full,
        )
    }
}
