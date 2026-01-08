pub struct Wallet{
    account: Account,
}

impl Wallet{
    pub fn new() -> Self{
        Self{
            account: Account::new()
        }
    }
}


struct Account;

impl Account{
    fn new() -> Self{
        Self
    }
}