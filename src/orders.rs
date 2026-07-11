//! Typed constructors for Schwab order specifications.

use chrono::NaiveDate;
use std::str::FromStr;

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
        if underlying.as_ref().len() > 6 || strike <= Money::ZERO {
            return Err(Error::InvalidOrder("invalid option symbol components".into()));
        }
        Ok(Self { underlying, expiration, put_call, strike })
    }

    pub fn to_occ_symbol(&self) -> String {
        let contract = match self.put_call { PutCall::Put => 'P', PutCall::Call => 'C' };
        let strike = (self.strike * Money::new(1000, 0)).trunc().to_string();
        format!("{:<6}{}{}{:0>8}", self.underlying, self.expiration.format("%y%m%d"), contract, strike)
    }

    pub fn parse_occ_symbol(value: &str) -> Result<Self> {
        if value.len() < 16 {
            return Err(Error::InvalidOrder("OCC option symbol is too short".into()));
        }
        let (underlying, suffix) = value.split_at(6);
        let expiration = NaiveDate::parse_from_str(&suffix[..6], "%y%m%d")
            .map_err(|_| Error::InvalidOrder("invalid OCC expiration date".into()))?;
        let put_call = match &suffix[6..7] {
            "P" => PutCall::Put,
            "C" => PutCall::Call,
            _ => return Err(Error::InvalidOrder("OCC option symbol must contain P or C".into())),
        };
        let strike = Money::from_str(&suffix[7..])
            .map_err(|_| Error::InvalidOrder("invalid OCC strike".into()))?
            / Money::new(1000, 0);
        Self::new(Symbol::new(underlying.trim_end())?, expiration, put_call, strike)
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
    pub fn complex_strategy(mut self, value: impl Into<String>) -> Self {
        self.order.complex_order_strategy_type = Some(value.into());
        self
    }

    pub fn equity_leg(mut self, instruction: Instruction, symbol: Symbol, quantity: Money) -> Result<Self> {
        self.push_leg(instruction, "EQUITY", symbol, quantity)?;
        Ok(self)
    }

    pub fn option_leg(mut self, instruction: Instruction, symbol: OptionSymbol, quantity: Money) -> Result<Self> {
        self.push_leg(instruction, "OPTION", Symbol::new(symbol.to_occ_symbol())?, quantity)?;
        Ok(self)
    }

    fn push_leg(&mut self, instruction: Instruction, asset_type: &str, symbol: Symbol, quantity: Money) -> Result<()> {
        if quantity <= Money::ZERO { return Err(Error::InvalidOrder("leg quantity must be positive".into())); }
        self.order.order_leg_collection.push(OrderLeg {
            instrument: Some(OrderInstrument { asset_type: Some(asset_type.into()), symbol: Some(symbol.to_string()), ..Default::default() }),
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

pub fn equity_sell_short_market(symbol: Symbol, quantity: Money) -> Result<Order> {
    OrderBuilder::market().equity_leg(Instruction::SellShort, symbol, quantity)?.build()
}

pub fn equity_sell_short_limit(symbol: Symbol, quantity: Money, price: Money) -> Result<Order> {
    OrderBuilder::limit(price).equity_leg(Instruction::SellShort, symbol, quantity)?.build()
}

pub fn equity_buy_to_cover_market(symbol: Symbol, quantity: Money) -> Result<Order> {
    OrderBuilder::market().equity_leg(Instruction::BuyToCover, symbol, quantity)?.build()
}

pub fn equity_buy_to_cover_limit(symbol: Symbol, quantity: Money, price: Money) -> Result<Order> {
    OrderBuilder::limit(price).equity_leg(Instruction::BuyToCover, symbol, quantity)?.build()
}

pub fn option_buy_to_open_market(symbol: OptionSymbol, quantity: Money) -> Result<Order> {
    OrderBuilder::market().option_leg(Instruction::BuyToOpen, symbol, quantity)?.build()
}

pub fn option_buy_to_open_limit(symbol: OptionSymbol, quantity: Money, price: Money) -> Result<Order> {
    OrderBuilder::limit(price).option_leg(Instruction::BuyToOpen, symbol, quantity)?.build()
}

pub fn option_sell_to_open_market(symbol: OptionSymbol, quantity: Money) -> Result<Order> {
    OrderBuilder::market().option_leg(Instruction::SellToOpen, symbol, quantity)?.build()
}

pub fn option_sell_to_open_limit(symbol: OptionSymbol, quantity: Money, price: Money) -> Result<Order> {
    OrderBuilder::limit(price).option_leg(Instruction::SellToOpen, symbol, quantity)?.build()
}

pub fn option_buy_to_close_market(symbol: OptionSymbol, quantity: Money) -> Result<Order> {
    OrderBuilder::market().option_leg(Instruction::BuyToClose, symbol, quantity)?.build()
}

pub fn option_buy_to_close_limit(symbol: OptionSymbol, quantity: Money, price: Money) -> Result<Order> {
    OrderBuilder::limit(price).option_leg(Instruction::BuyToClose, symbol, quantity)?.build()
}

pub fn option_sell_to_close_market(symbol: OptionSymbol, quantity: Money) -> Result<Order> {
    OrderBuilder::market().option_leg(Instruction::SellToClose, symbol, quantity)?.build()
}

pub fn option_sell_to_close_limit(symbol: OptionSymbol, quantity: Money, price: Money) -> Result<Order> {
    OrderBuilder::limit(price).option_leg(Instruction::SellToClose, symbol, quantity)?.build()
}

/// Build a two-leg vertical option spread. Callers specify the exact instructions
/// because debit/credit and opening/closing depend on the strategy direction.
pub fn vertical_spread(
    first: (Instruction, OptionSymbol),
    second: (Instruction, OptionSymbol),
    quantity: Money,
    price: Money,
    order_type: OrderType,
) -> Result<Order> {
    if !matches!(order_type, OrderType::NetDebit | OrderType::NetCredit) {
        return Err(Error::InvalidOrder("vertical spreads require NET_DEBIT or NET_CREDIT".into()));
    }
    OrderBuilder::new(order_type)
        .price(price)
        .quantity(quantity)
        .complex_strategy("VERTICAL")
        .option_leg(first.0, first.1, quantity)?
        .option_leg(second.0, second.1, quantity)?
        .build()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    use super::{equity_buy_limit, vertical_spread, OptionSymbol, PutCall};
    use crate::models::orders::{Instruction, OrderType};
    use crate::types::Symbol;

    #[test]
    fn builds_a_typed_equity_limit_order() {
        let order = equity_buy_limit(Symbol::new("AAPL").unwrap(), Decimal::ONE, Decimal::new(18525, 2)).unwrap();
        assert_eq!(order.order_leg_collection[0].instrument.as_ref().unwrap().asset_type.as_deref(), Some("EQUITY"));
        assert_eq!(order.price, Some(Decimal::new(18525, 2)));
    }

    #[test]
    fn builds_occ_option_symbols() {
        let option = OptionSymbol::new(Symbol::new("SPXW").unwrap(), NaiveDate::from_ymd_opt(2024, 4, 20).unwrap(), PutCall::Call, Decimal::new(5040, 0)).unwrap();
        assert_eq!(option.to_occ_symbol(), "SPXW  240420C05040000");
        assert_eq!(OptionSymbol::parse_occ_symbol("SPXW  240420C05040000").unwrap(), option);
    }

    #[test]
    fn builds_a_vertical_spread() {
        let expiration = NaiveDate::from_ymd_opt(2024, 4, 20).unwrap();
        let long = OptionSymbol::new(Symbol::new("SPXW").unwrap(), expiration, PutCall::Call, Decimal::new(5000, 0)).unwrap();
        let short = OptionSymbol::new(Symbol::new("SPXW").unwrap(), expiration, PutCall::Call, Decimal::new(5040, 0)).unwrap();
        let order = vertical_spread(
            (Instruction::BuyToOpen, long),
            (Instruction::SellToOpen, short),
            Decimal::ONE,
            Decimal::new(125, 2),
            OrderType::NetDebit,
        ).unwrap();
        assert_eq!(order.complex_order_strategy_type.as_deref(), Some("VERTICAL"));
        assert_eq!(order.order_leg_collection.len(), 2);
    }
}
