

use crate::dns_header;
use clap::ValueEnum;

#[derive(Debug)]
pub struct IpAnswer {
    // will need later for parsing multiple sites at onse
    name_offset: u16,
    class: u16,
    // for caching
    ttl: u32,
    pub rdata: RData,
}

#[derive(Debug)]
pub enum RData {
    A([u8; 4]),
    AAAA([u8; 16]),
    MX {
        preference: u16,
        exchange_offset: u16,
    },
    CNAME {
        name_offset: u16,
    },
    Unknown {
        rtype: u16,
        data: Vec<u8>,
    },
}

#[derive(Debug)]
pub struct DnsResponse {
    pub header: dns_header::DnsHeader,
    pub answers: Vec<IpAnswer>,
}


#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum RecordType {
    A = 1,
    MX = 15,
    TXT = 16,
    AAAA = 28,
}

impl RecordType {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            1 => Some(Self::A),
            15 => Some(Self::MX),
            16 => Some(Self::TXT),
            28 => Some(Self::AAAA),
            _ => None,
        }
    }
}


impl DnsResponse {


    fn decode_name(buf: &[u8], mut ptr: usize) -> Result<String, &'static str> {
        let mut labels = Vec::new();
        let mut jumped = false;
        let mut seen = 0;

        loop {
            if seen > buf.len() {
                return Err("Name compression loop detected");
            }
            seen += 1;

            let len = buf[ptr];

            // pointer
            if len & 0xC0 == 0xC0 {
                let b2 = buf[ptr + 1];
                let offset = (((len as u16) << 8) | b2 as u16) & 0x3FFF;
                ptr = offset as usize;
                jumped = true;
                continue;
            }

            // end
            if len == 0 {
                if !jumped {
                    ptr += 1;
                }
                break;
            }

            ptr += 1;
            let label = &buf[ptr..ptr + len as usize];
            labels.push(String::from_utf8_lossy(label).to_string());
            ptr += len as usize;
        }

        Ok(labels.join("."))
    }

    fn read_name(buf: &[u8], mut ptr: usize) -> Result<(u16, usize), &'static str> {
        let first_byte = buf[ptr];
        if first_byte & 0xC0 == 0xC0 {
            let raw = u16::from_be_bytes([buf[ptr], buf[ptr + 1]]);
            let offset = raw & 0x3FFF;
            ptr += 2;
            Ok((offset, ptr))
        } else {
            // inline name
            let start_ptr = ptr;
            while buf[ptr] != 0 {
                let len = buf[ptr] as usize;
                ptr += 1 + len;
                if ptr >= buf.len() {
                    return Err("Invalid name in response");
                }
            }
            ptr += 1;
            Ok((start_ptr as u16, ptr))
        }
    }



    pub fn pretty_print(&self, buf: &[u8]) {
        for ans in &self.answers {
            match &ans.rdata {
                RData::A(ip) => {
                    println!(
                        "A     {}.{}.{}.{}   TTL {}",
                        ip[0], ip[1], ip[2], ip[3], ans.ttl
                    );
                }

                RData::AAAA(ip) => {
                    use std::net::Ipv6Addr;
                    let addr = Ipv6Addr::from(*ip);
                    println!("AAAA  {}   TTL {}", addr, ans.ttl);
                }

                RData::MX {
                    preference,
                    exchange_offset,
                } => {
                    if let Ok(name) = Self::decode_name(buf, *exchange_offset as usize) {
                        println!(
                            "MX    {} {}   TTL {}",
                            preference, name, ans.ttl
                        );
                    } else {
                        println!("MX    {} <invalid-name>", preference);
                    }
                }

                RData::CNAME { name_offset } => {
                    if let Ok(name) = Self::decode_name(buf, *name_offset as usize) {
                        println!("CNAME {}   TTL {}", name, ans.ttl);
                    } else {
                        println!("CNAME <invalid-name>");
                    }
                }

                RData::Unknown { rtype, data } => {
                    println!(
                        "TYPE{} {:?}   TTL {}",
                        rtype, data, ans.ttl
                    );
                }
            }
        }
    }

    pub fn parse_response(&mut self, buf: &[u8; 512], question_size: usize) -> Result<(), &str> {
        if self.header.id != u16::from_be_bytes([buf[0], buf[1]]) {
            println!("the id from response is not matching");
        }
        let mut ptr = 12; // skip header first
        ptr += question_size; // skip question

        // parse answers
        let ancount = u16::from_be_bytes([buf[6], buf[7]]);
        
        for _ in 0..ancount {
            let (name_offset, new_ptr) = DnsResponse::read_name(buf, ptr)?;
            ptr = new_ptr;

            let rtype = u16::from_be_bytes([buf[ptr], buf[ptr+1]]);
            ptr += 2;
            let class = u16::from_be_bytes([buf[ptr], buf[ptr+1]]);
            ptr += 2;
            let ttl = u32::from_be_bytes([buf[ptr], buf[ptr+1], buf[ptr+2], buf[ptr+3]]);
            ptr += 4;
            let rdlength = u16::from_be_bytes([buf[ptr], buf[ptr+1]]);
            ptr += 2;
            match RecordType::from_u16(rtype) {
                Some(RecordType::A) => {
                    let data: &[u8] = &buf[ptr..ptr + rdlength as usize];
                    let final_data: [u8;4] = data.try_into().expect("Expected 4 bytes");
                    ptr += rdlength as usize;

                    let rdata:RData = RData::A(final_data);

                    self.answers.push(IpAnswer {
                        name_offset,
                        class,
                        ttl,
                        rdata,
        
                    });
                }
                Some(RecordType::AAAA) => {
                    let data: &[u8] = &buf[ptr..ptr + rdlength as usize];
                    let final_data: [u8;16] = data.try_into().expect("Expected 6 bytes");
                    ptr += rdlength as usize;

                    let rdata:RData = RData::AAAA(final_data);

                    self.answers.push(IpAnswer {
                        name_offset,
                        class,
                        ttl,
                        rdata,

                })}

                Some(RecordType::MX) => {
                    let preference = u16::from_be_bytes([buf[ptr], buf[ptr + 1]]);
                    ptr += 2;
                    let (exchange_offset, new_ptr) = Self::read_name(buf, ptr)?;
                    ptr = new_ptr;

                    let rdata = RData::MX {
                        preference,
                        exchange_offset,
                    };

                    self.answers.push(IpAnswer {
                        name_offset,
                        class,
                        ttl,
                        rdata
                    });
                }

                _ => {
                    let data = buf[ptr..ptr + rdlength as usize].to_vec();
                    ptr += rdlength as usize;

                    let rdata = RData::Unknown {
                        rtype,
                        data,
                    };

                    self.answers.push(IpAnswer {
                        name_offset,
                        class,
                        ttl,
                        rdata,
                    });
                }

            }

        }
    Ok(())
    }
    
}
