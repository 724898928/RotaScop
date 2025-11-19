pub mod shared;
pub use shared::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

        assert_eq!(4, 4);
    }
}
