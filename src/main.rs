use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use clap::{Arg, Command, command};
use trust_dns::op::{Message, MessageType, OpCode, Query};
use trust_dns::rr::{Name, RecordType};
use trust_dns::serialize::binary::{BinEncodable, BinEncoder};

fn main() {
    let app = Command::new("resolve")
        .about("A simple to use DNS resolver")
        .arg(Arg::new("dns-server").short('s').default_value("8.8.8.8"))
        // .arg(Arg::new("domain-name").long("domain-name").required(true))             // 如果指定了short, long 这些，那么在输入命令的时候就必须要加上。argo run -q -- --domain-name www.rustinaction.com
        // .arg(Arg::new("domain-name").short('d').required(true))                      // cargo run -q -- -d www.rustinaction.com
        .arg(Arg::new("domain-name").required(true))                 // cargo run -q -- www.rustinaction.com
        .get_matches();

    // 把命令行参数转换为一个有类型的域名
    let domain_name_raw = app.get_one::<String>("domain-name").unwrap();
    let domain_name = Name::from_ascii(&domain_name_raw).unwrap();  // 大概长这样：Name { is_fqdn: false, labels: [aa] }
    println!("{:?}", domain_name);

    // 把命令行参数转换为一个有类型的DNS服务器
    let dns_server_raw = app.get_one::<String>("dns-server").unwrap();
    let dns_sever: SocketAddr = format!("{}:53", dns_server_raw).parse().expect("invalid address");
    println!("{:?}", dns_sever);

    let mut request_as_bytes: Vec<u8> = Vec::with_capacity(512);
    let mut response_as_bytes: Vec<u8> = vec![0; 512];

    let mut msg = Message::new();
    msg.set_id(rand::random::<u16>())                                           // 设置消息的标识号，并使用 rand::random::<u16>() 生成一个 16 位无符号整数作为标识号
        .set_message_type(MessageType::Query)                                   // 设置消息的类型，这里设置为查询（Query）
        .add_query(Query::query(domain_name, RecordType::A))    // 添加一个查询（Query）对象，表示这个消息中需要查询哪个域名的 A 记录（即ipv4）。可以有多个Query。
        .set_op_code(OpCode::Query)                                             // 设置操作码（OpCode），表示这个 DNS 消息是一个查询消息
        .set_recursion_desired(true);                                                       // 设置递归期望（RD），表示这个查询消息需要进行递归查询。如果此DNS服务器不知道答案，可以发出询问其他DNS服务器

    // 使用 BinEncoder 把 Message 类型转换为原始字节
    let mut encoder = BinEncoder::new(&mut request_as_bytes);

    msg.emit(&mut encoder).unwrap();

    // 在随机端口上监听所有的地址
    let localhost = UdpSocket::bind("0.0.0.0:0").expect("cannot bind to local socket");

    let timeout = Duration::from_secs(10);

    localhost.set_read_timeout(Some(timeout)).unwrap();
    localhost.set_nonblocking(false).unwrap();

    let _amt = localhost.send_to(&request_as_bytes, dns_sever).expect("socket misconfigured");

    let (_amt, _remote) = localhost.recv_from(&mut response_as_bytes).expect("timeout reached");

    let dns_message = Message::from_vec(&response_as_bytes).expect("unable to parse response");

    for answer in dns_message.answers() {
        if answer.record_type() == RecordType::A {
            let resource = answer.rdata();
            let ip = resource.to_ip_addr().expect("invalid IP address received");
            println!("{}", ip.to_string());
        }
    }

}
