
use nes::cpu::CPU;
use nes::board::CPUBus;

fn main() {

	// create a cpu for test
	let bus = CPUBus::new();
	let mut cpu = CPU::new(bus);

	let code1 = vec![
        0xa9, 0x01,
        0x8d, 0x00, 0x02,
        0xa9, 0x05,
        0x8d, 0x01, 0x02,
        0xa9, 0x08,
        0x8d, 0x02, 0x02
    ];

    let addr = 0x0000;
    cpu.load_data(addr, &code1);

	cpu.power_up();




	loop {
		cpu.step();
	}
}
