#[derive(Debug, Clone, Copy)]
pub struct DnsQuestion {
    pub qname: [u8; 128],
    pub qname_len: usize,
    pub qtype: u16,
    pub qclass: u16,
}

impl DnsQuestion {
    pub fn builder() -> DnsQuestionBuilder {
        DnsQuestionBuilder {
            qname: [0; 128],
            qname_len: 0,
            qtype: 1,  
            qclass: 1,  
        }
    }


    pub fn write_question(&self, buf: &mut[u8; 512]) {
        let mut offset = 12;
        buf[offset..offset + self.qname_len].copy_from_slice(&self.qname[0..self.qname_len]);
        offset += self.qname_len;
        buf[offset..offset + 2].copy_from_slice(&self.qtype.to_be_bytes());
        offset += 2;
        buf[offset..offset + 2].copy_from_slice(&self.qclass.to_be_bytes());
    }
}

#[derive(Debug)]
pub struct DnsQuestionBuilder {
    qname: [u8; 128],
    qname_len: usize,
    qtype: u16,
    qclass: u16,
}


impl DnsQuestionBuilder {



    pub fn name(mut self, domain: &str) -> Result<Self, &'static str> {
        let mut pos = 0;

        for label in domain.split('.') {
            let len = label.len();

            if len == 0 || len > 63 {
                return Err("invalid DNS label length");
            }

            if pos + 1 + len >= 128 {
                return Err("domain name too long");
            }

            self.qname[pos] = len as u8;
            pos += 1;


            self.qname[pos..pos + len].copy_from_slice(label.as_bytes());
            pos += len;

        }
        if pos >= 128 {
            return Err("domain name too long");
        }
        self.qname[pos] = 0;
        pos += 1;

        self.qname_len = pos;
        Ok(self)



    }
    pub fn qtype(mut self, qtype: u16) -> Self {
        self.qtype = qtype;
        self
    }

    pub fn build(self) -> DnsQuestion {
        DnsQuestion {
            qname: self.qname,
            qname_len: self.qname_len,
            qtype: self.qtype,
            qclass: self.qclass,
        }
    }
}




#[test]
fn encodes_google_com_correctly() -> Result<(), &'static str> {
    let q = DnsQuestion::builder()
        .name("google.com")?
        .build();

    let expected = [
        6, b'g', b'o', b'o', b'g', b'l', b'e',
        3, b'c', b'o', b'm',
        0,
    ];

    assert_eq!(
        &q.qname[..q.qname_len],
        &expected
    );

    Ok(())
}

#[test]
fn encodes_chnu_ua_correctly() -> Result<(), &'static str> {
    let q = DnsQuestion::builder()
        .name("chnu.edu.ua")?
        .build();

    let expected = [
        4, b'c', b'h', b'n', b'u',
        3, b'e', b'd', b'u',
        2, b'u', b'a',
        0,
    ];

    assert_eq!(
        &q.qname[..q.qname_len],
        &expected
    );

    Ok(())
}
