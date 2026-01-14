mod client;
mod model;
mod requests;
mod responses;
pub use client::*;
pub use model::*;
pub use requests::*;
pub use responses::*;

// Re-export commonly used account stream types at the crate root for convenience
pub use responses::stream::{
    AccountBalanceMessage, DltaPointsData, DltaPointsMessage, OrderInfoMessage, StreamMessage,
    StreamSubType, StreamType,
};

// Re-export market stream types at the crate root for convenience
pub use responses::stream::{
    MarketDepthMessage, MarketPriceMessage, MarketStreamMessage, OhlcMessage, PriceLevel, Trade,
};

// Re-export stream reconnection types and events
pub use client::{
    MarketStreamEvent, MarketStreamHandle, ReconnectConfig, StreamError, StreamEvent, StreamHandle,
};
