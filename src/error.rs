use async_graphql::ErrorExtensions;

pub trait Error {
    fn message(&self) -> String;
    fn code(&self) -> String;
    fn build(&self) -> async_graphql::Error {
        let code = self.code();
        let message = self.message();
        async_graphql::Error::new(format!("{} : {}", code, message)).extend_with(|_, e| e.set("code", code))
    }
}
