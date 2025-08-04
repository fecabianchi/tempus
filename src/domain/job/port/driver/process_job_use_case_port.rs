use crate::error::Result;

pub trait ProcessJobUseCasePort {
   async fn execute(&self) -> Result<()>;
}
