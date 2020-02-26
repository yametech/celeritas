pub enum CommandType {
    String,
    Set,
    List,
    Zset,
    Hash,
    // and other
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
