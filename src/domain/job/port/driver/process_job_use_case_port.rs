pub trait ProcessJobUseCasePort {
   async fn execute(&self) -> ();
}
