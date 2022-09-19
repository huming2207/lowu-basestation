use bbqueue::{Consumer, Producer, BBBuffer};

static UART_BBQ: BBBuffer<1024> = BBBuffer::new();

const SLIP_END: u8 = 0xc0;
const SLIP_ESC: u8 = 0xdb;
const SLIP_ESC_END: u8 = 0xdc;
const SLIP_ESC_ESC: u8 = 0xdd;

pub(crate) struct CmdBroker {
    bbq_cons: Consumer<'static, 1024>,
    bbq_prod: Producer<'static, 1024>,
    pkt_start: bool,
    pkt_esc: bool,
}

impl CmdBroker {
    pub fn init() -> CmdBroker {
        let (prod, cons) = UART_BBQ.try_split().unwrap();
        CmdBroker { bbq_cons: cons, bbq_prod: prod, pkt_start: false, pkt_esc: false }
    }

    pub fn decode_and_enqueue(&mut self, new_byte: u8) -> bool {
        // Scenario 0: SLIP_END
        if self.pkt_start && new_byte == SLIP_END {
            self.pkt_start = false;
            self.pkt_esc = false;
            return true;
        } else if !self.pkt_start && new_byte == SLIP_END {
            self.pkt_start = true;
            self.pkt_esc = false;
            return false;
        }  
        
        // Scenario 1: SLIP_ESC
        if new_byte == SLIP_ESC {
            self.pkt_esc = true;
            return false; // No need to add SLIP_ESC in to buffer
        }

        // Scenario 2: other bytes
        let mut granted = self.bbq_prod.grant_exact(1).unwrap();
        if !self.pkt_esc {
            granted[0] = new_byte;
        } else {
            if new_byte == SLIP_ESC_END {
                granted[0] = SLIP_END;
            } else if new_byte == SLIP_ESC_ESC {
                granted[0] = SLIP_ESC;
            } else {
                defmt::error!("SLIP ESC with unknown following bytes: 0x{:02x}", new_byte);
            }

            self.pkt_esc = false;
        }

        granted.commit(1);
        return false;
    }

    pub fn read(&mut self) {
        let reader = self.bbq_cons.read().unwrap();
        
    }
}