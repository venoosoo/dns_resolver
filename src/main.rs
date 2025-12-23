

use rand::Rng;
use std::{net::UdpSocket};
use clap::Parser;

use crate::parse_answer::RecordType;


mod dns_question;
mod dns_header;
mod dns_packet;
mod parse_answer;


#[derive(Parser)]
#[command(name = "ven_dns")]
#[command[about = "Dns resolver cli", long_about = None]]
struct Cli {
    #[arg(long, required = true, help = "Target domain, e.g. example.com")]
    target: String,

    #[arg(long, value_enum, default_value_t = RecordType::A)]
    r#type: RecordType,

    #[arg(long, default_value="1.1.1.1")]
    server: String,
    
}



fn main() -> Result<(), Box<dyn std::error::Error>>{


    let cli = Cli::parse();

    let mut rng = rand::rng();
    let random_num: u16 = rng.random();

    let mut buffer: [u8; 512] = [0u8; 512];


    let header = dns_header::DnsHeader::new(random_num);


    let rtype = cli.r#type as u16;


    let question = dns_question::DnsQuestion::builder()
        .name(&cli.target)?
        .qtype(rtype)
        .build();


    let packet = dns_packet::DnsPacket {
        header: header,
        question: question,
    };

    packet.write_packet(&mut buffer);



    let used_len = 12 + question.qname_len + 4; // header + question

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let server = format!("{}:53",cli.server);

    socket.connect(server)?;


    socket.send(&buffer[..used_len])?;

    let mut response = [0u8; 512];
    socket.recv(&mut response)?;


    let rec_id = u16::from_be_bytes([response[0], response[1]]);
    let flags = u16::from_be_bytes([response[2], response[3]]);
    let qdcount = u16::from_be_bytes([response[4], response[5]]);
    let ancount = u16::from_be_bytes([response[6], response[7]]);
    let nscount = u16::from_be_bytes([response[8], response[9]]);
    let arcount = u16::from_be_bytes([response[10], response[11]]);

    let response_header = dns_header::DnsHeader {
        id: rec_id,
        flags,
        qdcount,
        ancount,
        nscount,
        arcount,
    };

    let mut parsed_answer = parse_answer::DnsResponse {
        header: response_header,
        answers: Vec::new(),
    };

    parsed_answer.parse_response(&response, question.qname_len + 4)?;



    parsed_answer.pretty_print(&response);



    Ok(())
    

}
