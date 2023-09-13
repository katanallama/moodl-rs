// commands/command.rs
//
use async_trait::async_trait;
use {eyre::Result, termimad::MadSkin};

#[async_trait]
pub trait Command {
    async fn execute(&mut self) -> Result<()>;
}

pub struct DefaultCommand<'a> {
    _skin: &'a MadSkin,
}

impl<'a> DefaultCommand<'a> {
    pub fn new(_skin: &'a MadSkin) -> Self {
        Self { _skin }
    }
}

#[async_trait]
impl<'a> Command for DefaultCommand<'a> {
    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}
