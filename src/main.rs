pub struct CPU {
    pub register_a: u8,
    pub status: u8,
    pub program_counter:u16, // program counter register will help us to track our current position in the program
    pub register_x:u8,
    pub register_y:u8,
    memory: [u8; 0xFFFF] // 8bits(1byte), 65536 bytes, 65k

    //load program code into memory, starting at 0x8000 address. [0x8000.. 0xFFFF] is reserved for program ROM
}

pub struct OpCode { 
    pub opcode:u8,
    pub name:String,
    pub bytes:u8,
    pub cycles:u8,
    pub addr_mode:AddressingMode,
}


#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode { 

    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing
}
impl OpCode {
    pub fn new(&mut self, opcode:u8, name:String, bytes:u8, cycles:u8, addr_mode:AddressingMode) -> self {
        OpCode {
            self.opcode = opcode;
            self.name = name;
            self.bytes = bytes;
            self.cycles = cycles;
            self.addr_mode = addr_mode;
        }
    }
}

impl CPU {

    pub fn new() -> Self {
        CPU {
            register_a: 0,
            status:0,
            program_counter:0, 
            register_x: 0,
            register_y:0,
            memory: [0;0xFFFF]
        }
    }

    //CPU works in a constant cycle
    //- Fetch next execution instruction from the instruction memory
    //- Decode the instruction
    //- Execute the instruction
    //- Repeat the cycle
    //
    pub fn reset(&mut self){
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC); // Where Program Counter value is stored
    }
    fn mem_read(&self, addr:u16) -> u8 { //mem_size is 64k and it can be accessed with 16 bits addr
        self.memory[addr as usize]
    }
    fn mem_write(&mut self, addr:u16, data:u8){
        self.memory[addr as usize] = data;
    }
    fn mem_read_u16(&self, pos:u16) -> u16{ // NES is little-endian
        // i.e.)Actual Data = 0xFF_00
        //                 hi_lo
        // Stored in memory = 0x00_FF 
        // memory[pos]   = 0x00 //lo
        // memory[pos+1] = 0xFF //hi
        let lo = self.mem_read(pos) as u16;
        //  lo = 0x0000
        let hi = self.mem_read(pos + 1) as u16;
        //  hi = 0x00FF
        (hi << 8) | (lo as u16)
        // hi = 0xFF00
        // lo = 0x0000
        // hi | lo = 0xFF00
    }
    fn mem_write_u16(&mut self, pos:u16, data:u16){
        //i.e.)Actual data = 0xFF_00
        //mem_lo = 00
        //mem_hi = FF
        // hi = (data >> 8) as u8
        // lo = (data & 0x00ff) as u8

        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
    
    fn get_operand_address(&self, mode: &AddressingMode) -> u16{
        match mode {
            AddressingMode::Immediate => self.program_counter,
            
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
    
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter) as u16,
    
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
    
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
    
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
    
            AddressingMode::Indirect_Y=> {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);

                deref
            }
    
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }

        }
    }
    pub fn load_and_run(&mut self, program:Vec<u8>){
        self.load(program);
        self.reset(); 

        self.run();
    }

    pub fn load(&mut self, program:Vec<u8>){
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000); // 0xFFFC (Program Counter is set to 0x8000)
    }

    pub fn run(&mut self){

        loop {

            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1;
            
            match opcode {
                0xA9 => { // LDA(0xA9) operation code
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xA5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 1;
                }
                
                0x85 => {// STA
                    self.sta(AddressingMode::ZeroPage);
                    self.program_counter += 1;

                }
                0x95 => {
                    self.sta(AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0xAA => self.tax(), //TAX (0xAA)
                0xE8 => self.inx(), //INX

                0x00 => return, // BRK(0x00)
                _ => todo!()
            }
        }

    }
    fn lda(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn inx(&mut self){
        if self.register_x == 0xff {
            self.register_x = 0;
        } else {
            self.register_x += 1;
        }
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn sta(&mut self, mode: AddressingMode){
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }
    fn update_zero_and_negative_flags(&mut self, _result: u8){

        if _result == 0 {
            // it's essential to set or unset CPU flag status depending on the results
            self.status = self.status | 0b0000_0010;

        } else {
            self.status = self.status & 0b1111_1101;
        }

        if _result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;

        } else {
            self.status = self.status & 0b0111_1111;
        }
    }
    pub fn interpret(&mut self, program: Vec<u8>){
        // interpret method takes a mutable reference to self as we know that we will need to
        // modify `register_a` during the execution.


    }

}

pub static ref CPU_OPS_CODES: Vec<OpCode> = vec! [
    OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
    OpCode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),


    OpCode::new(0xa9, "LDA", 2, 2, AddressingMode::Immediate),
    OpCode::new(0xa5, "LDA", 2, 3, AddressingMode::ZeroPage),
    OpCode::new(0xb5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
    OpCode::new(0xad, "LDA", 3, 4, AddressingMode::Absolute),
    OpCode::new(0xbd, "LDA", 3, 4, AddressingMode::Absolute_X),
    OpCode::new(0xb9, "LDA", 3, 4, AddressingMode::Absolute_Y),
    OpCode::new(0xa1, "LDA", 2, 6, AddressingMode::Indirect_X),
    OpCode::new(0xb1, "LDA", 2, 5, AddressingMode::Indirect_Y),
];


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data(){

        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0)
    }
    #[test]
    fn test_0xa9_lda_zero_flag(){
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10)
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x(){
        let mut cpu = CPU::new();
        cpu.load(vec![0xaa, 0x00]);
        cpu.reset();
        cpu.register_a = 10;
        cpu.run();

        assert_eq!(cpu.register_x, 10)
    }
    #[test]
    fn test_5_ops_working_together(){
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow(){
        let mut cpu = CPU::new();
        cpu.load(vec![0xe8, 0xe8, 0x00]);
        cpu.reset();
        cpu.register_x = 0xff;
        cpu.run();
        assert_eq!(cpu.register_x,1)
    }

    #[test]
    fn test_lda_from_memory(){
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }
}


fn main() {
    println!("Hello, world!");
}
