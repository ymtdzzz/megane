use anyhow::Result;
use async_trait::async_trait;

pub mod loggroup_event_handler;
pub mod input_event_handler;

#[async_trait]
pub trait EventHandler {
    async fn run(&mut self) -> Result<()>;
}
