use constants::*;
use dns_sector::*;
use rr_iterator::*;

#[derive(Debug)]
pub struct ResponseIterator<'t> {
    rr_iterator: RRIterator<'t>,
}

pub type AnswerIterator<'t> = ResponseIterator<'t>;
pub type NameServersIterator<'t> = ResponseIterator<'t>;
pub type AdditionalIterator<'t> = ResponseIterator<'t>;

impl<'t> TypedIterable for ResponseIterator<'t> {}

impl<'t> DNSIterable for ResponseIterator<'t> {
    #[inline]
    fn offset(&self) -> Option<usize> {
        self.rr_iterator.offset
    }

    #[inline]
    fn raw(&self) -> RRRaw {
        RRRaw {
            packet: &self.rr_iterator.parsed_packet.dns_sector.packet,
            offset: self.rr_iterator.offset.unwrap(),
            name_end: self.rr_iterator.name_end,
        }
    }

    #[inline]
    fn raw_mut(&mut self) -> RRRawMut {
        RRRawMut {
            packet: &mut self.rr_iterator.parsed_packet.dns_sector.packet,
            offset: self.rr_iterator.offset.unwrap(),
            name_end: self.rr_iterator.name_end,
        }
    }

    fn next(mut self) -> Option<Self> {
        {
            let rr_iterator = &mut self.rr_iterator;
            let parsed_packet = &mut rr_iterator.parsed_packet;
            if rr_iterator.offset.is_none() {
                let (count, offset) = match rr_iterator.section {
                    Section::Answer => {
                        (DNSSector::ancount(&parsed_packet.dns_sector.packet),
                         parsed_packet.offset_answers)
                    }
                    Section::NameServers => {
                        (DNSSector::nscount(&parsed_packet.dns_sector.packet),
                         parsed_packet.offset_nameservers)
                    }
                    Section::Additional => {
                        (DNSSector::arcount(&parsed_packet.dns_sector.packet),
                         parsed_packet.offset_additional)
                    }                    
                    _ => unreachable!("Unexpected section"),
                };
                if count == 0 {
                    return None;
                }
                rr_iterator.rrs_left = count;
                rr_iterator.offset_next = offset.unwrap();
            }
            if rr_iterator.rrs_left == 0 {
                return None;
            }
            rr_iterator.rrs_left -= 1;
            rr_iterator.offset = Some(rr_iterator.offset_next);
            rr_iterator.name_end = RRIterator::skip_name(&parsed_packet.dns_sector.packet,
                                                         rr_iterator.offset.unwrap());
            let offset_next = RRIterator::skip_rdata(&parsed_packet.dns_sector.packet,
                                                     rr_iterator.name_end);
            rr_iterator.offset_next = offset_next;
        }
        self.maybe_skip_opt_section()
    }
}

impl<'t> ResponseIterator<'t> {
    pub fn new(rr_iterator: RRIterator<'t>) -> Self {
        ResponseIterator { rr_iterator }
    }

    fn maybe_skip_opt_section(mut self) -> Option<Self> {
        if self.rr_type() == Type::OPT.into() {
            let rr_iterator = &mut self.rr_iterator;
            debug_assert_eq!(rr_iterator.section, Section::Additional);
            let parsed_packet = &mut rr_iterator.parsed_packet;
            if rr_iterator.rrs_left == 0 {
                return None;
            }
            rr_iterator.offset = Some(rr_iterator.offset_next);
            rr_iterator.name_end = RRIterator::skip_name(&parsed_packet.dns_sector.packet,
                                                         rr_iterator.offset.unwrap());
            let offset_next = RRIterator::skip_rdata(&parsed_packet.dns_sector.packet,
                                                     rr_iterator.name_end);
            rr_iterator.offset_next = offset_next;
        }
        debug_assert!(self.rr_type() != Type::OPT.into());
        Some(self)
    }
}
