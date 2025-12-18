use std::sync::Once;

static INIT: Once = Once::new();
/// because logger can only init once, so we use Once to ensure it
pub fn init_logger() {
    INIT.call_once(env_logger::init);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1() -> anyhow::Result<()> {
        init_logger();
        Ok(())
    }
}
