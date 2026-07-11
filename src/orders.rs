//! Typed constructors for Schwab order specifications.

use chrono::NaiveDate;

use crate::error::{Error, Result};
use crate::models::orders::{
    Duration, Instruction, Order, OrderInstrument, OrderLeg, OrderStrategyType, OrderType,
    Session,
};
use crate::types::{Money, Symbol};

/// Put or call contract type in an OCC option symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutCall { Put, Call }

/// Validated OCC option symbol components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionSymbol {
    underlying: Symbol,
    expiration: NaiveDate,
    put_call: PutCall,
    strike: Money,
}

impl OptionSymbol {
    pub fn new(underlying: Symbol, expiration: NaiveDate, put_call: PutCall, strike: Money) -> Result<Self> {
        if underlying.0.is_empty() || underlying.0.len() > 6 || strike <= Money::ZERO {
            return Err(Error::InvalidOrder("invalid option symbol components".into()));
        }
        Ok(Self { underlying, expiration, put_call, strike })
    }

    pub fn to_occ_symbol(&self) -> String {
        let contract = match self.put_call { PutCall::Put => 'P', PutCall::Call => 'C' };
        let strike = (self.strike * Money::new(1000, 0)).trunc().to_string();
        format!("{:<6}{}{}{:0>8}", self.underlying.0, self.expiration.format("%y%m%d"), contract, strike)
    }
}

/// Builder for a complete, serializable Schwab order specification.
#[derive(Debug, Clone)]
pub struct OrderBuilder { order: Order }

impl OrderBuilder {
    pub fn new(order_type: OrderType) -> Self {
        Self { order: Order { order_type, session: Session::Normal, duration: Duration::Day,
            order_strategy_type: OrderStrategyType::Single, order_leg_collection: Vec::new(), ..Default::default() } }
    }

    pub fn market() -> Self { Self::new(OrderType::Market) }
    pub fn limit(price: Money) -> Self { Self::new(OrderType::Limit).price(price) }
    pub fn net_debit(price: Money) -> Self { Self::new(OrderType::NetDebit).price(price) }
    pub fn net_credit(price: Money) -> Self { Self::new(OrderType::NetCredit).price(price) }

    pub fn session(mut self, value: Session) -> Self { self.order.session = value; self }
    pub fn duration(mut self, value: Duration) -> Self { self.order.duration = value; self }
    pub fn price(mut self, value: Money) -> Self { self.order.price = Some(value); self }
    pub fn quantity(mut self, value: Money) -> Self { self.order.quantity = Some(value); self }

    pub fn equity_leg(mut self, instruction: Instruction, symbol: Symbol, quantity: Money) -> Result<Self> {
        self.push_leg(instruction, "EQUITY", symbol, quantity)?;
        Ok(self)
    }

    pub fn option_leg(mut self, instruction: Instruction, symbol: OptionSymbol, quantity: Money) -> Result<Self> {
        self.push_leg(instruction, "OPTION", Symbol(symbol.to_occ_symbol()), quantity)?;
        Ok(self)
    }

    fn push_leg(&mut self, instruction: Instruction, asset_type: &str, symbol: Symbol, quantity: Money) -> Result<()> {
        if quantity <= Money::ZERO { return Err(Error::InvalidOrder("leg quantity must be positive".into())); }
        self.order.order_leg_collection.push(OrderLeg {
            instrument: Some(OrderInstrument { asset_type: Some(asset_type.into()), symbol: Some(symbol.0), ..Default::default() }),
            instruction: Some(instruction), quantity: Some(quantity), ..Default::default()
        });
        Ok(())
    }

    pub fn build(self) -> Result<Order> {
        if self.order.order_leg_collection.is_empty() { return Err(Error::InvalidOrder("an order needs at least one leg".into())); }
        Ok(self.order)
    }
}

/// Compose two orders so execution of the first activates the second.
pub fn first_triggers_second(first: Order, second: Order) -> Order {
    Order { order_strategy_type: OrderStrategyType::Trigger, child_order_strategies: Some(vec![first, second]), ..Default::default() }
}

/// Compose two orders so execution of either cancels the other.
pub fn one_cancels_other(first: Order, second: Order) -> Order {
    Order { order_strategy_type: OrderStrategyType::Oco, child_order_strategies: Some(vec![first, second]), ..Default::default() }
}

pub fn equity_buy_market(symbol: Symbol, quantity: Money) -> Result<Order> {
    OrderBuilder::market().equity_leg(Instruction::Buy, symbol, quantity)?.build()
}

pub fn equity_buy_limit(symbol: Symbol, quantity: Money, price: Money) -> Result<Order> {
    OrderBuilder::limit(price).equity_leg(Instruction::Buy, symbol, quantity)?.build()
}

pub fn equity_sell_market(symbol: Symbol, quantity: Money) -> Result<Order> {
    OrderBuilder::market().equity_leg(Instruction::Sell, symbol, quantity)?.build()
}

pub fn equity_sell_limit(symbol: Symbol, quantity: Money, price: Money) -> Result<Order> {
    OrderBuilder::limit(price).equity_leg(Instruction::Sell, symbol, quantity)?.build()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    use super::{equity_buy_limit, OptionSymbol, PutCall};
    use crate::types::Symbol;

    #[test]
    fn builds_a_typed_equity_limit_order() {
        let order = equity_buy_limit(Symbol("AAPL".into()), Decimal::ONE, Decimal::new(18525, 2)).unwrap();
        assert_eq!(order.order_leg_collection[0].instrument.as_ref().unwrap().asset_type.as_deref(), Some("EQUITY"));
        assert_eq!(order.price, Some(Decimal::new(18525, 2)));
    }

    #[test]
    fn builds_occ_option_symbols() {
        let option = OptionSymbol::new(Symbol("SPXW".into()), NaiveDate::from_ymd_opt(2024, 4, 20).unwrap(), PutCall::Call, Decimal::new(5040, 0)).unwrap();
        assert_eq!(option.to_occ_symbol(), "SPXW  240420C05040000");
    }
}
