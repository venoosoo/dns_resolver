use crate::dns_header::DnsHeader;
use crate::dns_question::DnsQuestion;

pub struct DnsPacket {
    pub header: DnsHeader,
    pub question: DnsQuestion
}



impl DnsPacket {
    pub fn write_packet(&self, buf: &mut[u8; 512]) {
        self.header.write_header(buf);
        self.question.write_question(buf);
    }
}