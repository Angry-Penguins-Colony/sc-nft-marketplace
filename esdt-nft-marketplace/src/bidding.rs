elrond_wasm::imports!();

use crate::auction::{Auction, AuctionType, NFT_AMOUNT};

#[elrond_wasm::module]
pub trait BiddingModule:
    crate::auction::AuctionModule
    + crate::events::EventsModule
    + crate::token_distribution::TokenDistributionModule
    + crate::common_util_functions::CommonUtilFunctions
    + elrond_wasm_modules::pause::PauseModule
{
    #[payable("*")]
    #[endpoint(buySft)]
    fn buy_sft(
        &self,
        auction_id: u64,
        nft_type: TokenIdentifier,
        nft_nonce: u64,
        opt_sft_buy_amount: OptionalValue<BigUint>,
    ) {
        self.require_not_paused();

        let (payment_token, payment_token_nonce, payment_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();
        let mut auction = self.try_get_auction(auction_id);
        let caller = self.blockchain().get_caller();

        let sft_buy_amount = match opt_sft_buy_amount {
            OptionalValue::Some(amt) => amt,
            OptionalValue::None => BigUint::from(NFT_AMOUNT),
        };
        let sft_total_value = &sft_buy_amount * &auction.min_bid;

        self.common_bid_checks(
            &auction,
            &nft_type,
            nft_nonce,
            &payment_token,
            payment_token_nonce,
        );

        require!(sft_buy_amount > 0, "Must buy more than 0");
        require!(
            auction.auction_type == AuctionType::SftOnePerPayment,
            "Cannot buy SFT for this type of auction"
        );
        require!(
            sft_buy_amount <= auction.auctioned_tokens.amount,
            "Not enough SFTs available"
        );
        require!(
            sft_total_value == payment_amount,
            "Wrong amount paid, must pay equal to the selling price"
        );

        auction.current_winner = caller;
        auction.current_bid = payment_amount;
        self.distribute_tokens_after_auction_end(&auction, Some(&sft_buy_amount));

        auction.auctioned_tokens.amount -= &sft_buy_amount;
        if auction.auctioned_tokens.amount == 0 {
            self.auction_by_id(auction_id).clear();
            self.bought_auction_by_id(auction_id).set(&auction);
        } else {
            self.auction_by_id(auction_id).set(&auction);
        }

        self.emit_buy_sft_event(auction_id, auction, sft_buy_amount);
    }

    fn common_bid_checks(
        &self,
        auction: &Auction<Self::Api>,
        nft_type: &TokenIdentifier,
        nft_nonce: u64,
        payment_token: &EgldOrEsdtTokenIdentifier,
        payment_nonce: u64,
    ) {
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();

        require!(
            &auction.auctioned_tokens.token_identifier == nft_type
                && auction.auctioned_tokens.token_nonce == nft_nonce,
            "Auction ID does not match the token"
        );
        require!(
            auction.original_owner != caller,
            "Can't bid on your own token"
        );
        require!(
            current_time >= auction.start_time,
            "Auction hasn't started yet"
        );
        require!(current_time < auction.deadline, "Auction ended already");
        require!(
            payment_token == &auction.payment_token && payment_nonce == auction.payment_nonce,
            "Wrong token used as payment"
        );
    }
}
