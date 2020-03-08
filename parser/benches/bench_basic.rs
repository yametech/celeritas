#![feature(test)]
extern crate test;
use parser::parse_redis_value;
use test::Bencher;

#[bench]
fn bench_basic_roundtrip_all_type(b: &mut Bencher) {
    b.iter(|| {
        let set = b"~6\r\n+orange\r\n#t\r\n:1111\r\n(321328139271389216321689\r\n,1.23\r\n~1\r\n*3\r\n$3\r\nset\r\n$1\r\na\r\n$1\r\n1\r\n";
        assert_eq!(
            parse_redis_value(&set[..]).unwrap().as_bytes(),
            set.to_vec()
        );
    })
}

#[bench]
fn bench_basic_roundtrip_command(b:&mut Bencher) {
    b.iter(|| {
        let cmd = b"*3\r\n$3\r\nset\r\n$6\r\nmy_key\r\n$8\r\nmy_value\r\n";
        assert_eq!(
            parse_redis_value(&cmd[..]).unwrap().as_bytes(),
            cmd.to_vec()
        )
    })
}
