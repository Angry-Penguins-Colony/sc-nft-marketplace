elrond_wasm::imports!();
elrond_wasm::derive_imports!();


#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct MarketData<M: ManagedTypeApi> {
    pub min_price: BigUint<M>,
    pub average_price: BigUint<M>,
    pub total_listed: u64,
    pub total_volume: BigUint<M>,
}

#[elrond_wasm::module]
pub trait MarketDataModule:
    crate::auction::AuctionModule
    + crate::token_distribution::TokenDistributionModule
    + crate::events::EventsModule
    + crate::common_util_functions::CommonUtilFunctions
    + elrond_wasm_modules::pause::PauseModule
{
    #[view(getMarketData)]
    fn get_market_data_of_collection(&self, collection: TokenIdentifier<Self::Api>) -> MarketData<Self::Api> {
        let mut total_listed = 0;
        let mut total_volume = BigUint::zero();
        let mut min_price = BigUint::zero();
        let mut average_price = BigUint::zero();
        let mut total_price = BigUint::zero();


        for id in 1..=self.last_valid_auction_id().get() {
            let opt_listed_auction = self.auction_by_id(id);

            if !opt_listed_auction.is_empty(){
                let auction = opt_listed_auction.get();
                if auction.auctioned_tokens.token_identifier == collection {
                    total_listed += 1;
                    total_price += &auction.current_bid;

                    if min_price == BigUint::zero() || &auction.current_bid < &min_price {
                        min_price = auction.current_bid;
                    }
                }
            } else {
                let opt_bought_auction = self.bought_auction_by_id(id);

                if !opt_bought_auction.is_empty() {
                    let auction = opt_bought_auction.get();

                    if auction.auctioned_tokens.token_identifier == collection {
                        total_volume += auction.price;
                    }
                }
            }
        }

        if total_listed > 0 {
            average_price = total_price / BigUint::from(total_listed);
        }

        MarketData {
            min_price,
            average_price,
            total_listed,
            total_volume,
        }    
   }
}
