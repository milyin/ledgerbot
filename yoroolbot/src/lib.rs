//! Yoroolbot - A library crate for yoroolbot functionality

pub fn hello_yoroolbot() -> String {
    "Hello from Yoroolbot!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_yoroolbot() {
        assert_eq!(hello_yoroolbot(), "Hello from Yoroolbot!");
    }
}