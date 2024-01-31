pub mod account;
pub mod header;

#[derive(Debug, PartialEq)]
pub enum Collection {
    Header,
    Account,
    Storage,
}
