a ='''  case 0x00: this->implied_addressing();     this->BRK();  cycles -= 7; return BRK_INSTRUCTION; break;
        case 0x01: this->indirect_x_addressing();  this->ORA();  cycles -= 6; break;
        case 0x04: this->zero_page_addressing();    this->NOP();  cycles -= 1; break;
        case 0x05: this->zero_page_addressing();    this->ORA();  cycles -= 3; break;
        case 0x06: this->zero_page_addressing();    this->ASL();  cycles -= 5; break;
        case 0x08: this->implied_addressing();     this->PHP();  cycles -= 3; break;
        case 0x09: this->immediate_addressing();   this->ORA();  cycles -= 2; break;
        case 0x0A: this->accumulator_addressing(); this->ASLA(); cycles -= 2; break;
        case 0x0C: this->absolute_addressing();    this->NOP();  cycles -= 1; break;
        case 0x0D: this->absolute_addressing();    this->ORA();  cycles -= 4; break;
        case 0x0E: this->absolute_addressing();    this->ASL();  cycles -= 6; break;
        case 0x10: this->relative_addressing();    this->BPL();  cycles -= 2; break;
        case 0x11: this->indirect_y_addressing();  this->ORA();  cycles -= 5; break;
        case 0x14: this->zero_page_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0x15: this->zero_page_x_addressing();  this->ORA();  cycles -= 4; break;
        case 0x16: this->zero_page_x_addressing();  this->ASL();  cycles -= 6; break;
        case 0x18: this->implied_addressing();     this->CLC();  cycles -= 2; break;
        case 0x19: this->absolute_y_addressing();  this->ORA();  cycles -= 4; break;
        case 0x1A: this->accumulator_addressing(); this->NOP();  cycles -= 1; break;
        case 0x1C: this->absolute_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0x1D: this->absolute_x_addressing();  this->ORA();  cycles -= 4; break;
        case 0x1E: this->absolute_x_addressing();  this->ASL();  cycles -= 7; break;
        case 0x20: this->absolute_addressing();    this->JSR();  cycles -= 6; break;
        case 0x21: this->indirect_x_addressing();  this->AND();  cycles -= 6; break;
        case 0x24: this->zero_page_addressing();    this->BIT();  cycles -= 3; break;
        case 0x25: this->zero_page_addressing();    this->AND();  cycles -= 3; break;
        case 0x26: this->zero_page_addressing();    this->ROL();  cycles -= 5; break;
        case 0x28: this->implied_addressing();     this->PLP();  cycles -= 3; break;
        case 0x29: this->immediate_addressing();   this->AND();  cycles -= 2; break;
        case 0x2A: this->accumulator_addressing(); this->ROLA(); cycles -= 2; break;
        case 0x2C: this->absolute_addressing();    this->BIT();  cycles -= 4; break;
        case 0x2D: this->absolute_addressing();    this->AND();  cycles -= 2; break;
        case 0x2E: this->absolute_addressing();    this->ROL();  cycles -= 6; break;
        case 0x30: this->relative_addressing();    this->BMI();  cycles -= 2; break;
        case 0x31: this->indirect_y_addressing();  this->AND();  cycles -= 5; break;
        case 0x34: this->zero_page_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0x35: this->zero_page_x_addressing();  this->AND();  cycles -= 4; break;
        case 0x36: this->zero_page_x_addressing();  this->ROL();  cycles -= 6; break;
        case 0x38: this->implied_addressing();     this->SEC();  cycles -= 2; break;
        case 0x39: this->absolute_y_addressing();  this->AND();  cycles -= 4; break;
        case 0x3A: this->accumulator_addressing(); this->NOP();  cycles -= 1; break;
        case 0x3C: this->absolute_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0x3D: this->absolute_x_addressing();  this->AND();  cycles -= 4; break;
        case 0x3E: this->absolute_x_addressing();  this->ROL();  cycles -= 7; break;
        case 0x40: this->implied_addressing();     this->RTI();  cycles -= 6; break;
        case 0x41: this->indirect_x_addressing();  this->EOR();  cycles -= 6; break;
        case 0x44: this->zero_page_addressing();    this->NOP();  cycles -= 1; break;
        case 0x45: this->zero_page_addressing();    this->EOR();  cycles -= 3; break;
        case 0x46: this->zero_page_addressing();    this->LSR();  cycles -= 5; break;
        case 0x48: this->implied_addressing();     this->PHA();  cycles -= 3; break;
        case 0x49: this->immediate_addressing();   this->EOR();  cycles -= 2; break;
        case 0x4A: this->accumulator_addressing(); this->LSRA(); cycles -= 2; break;
        case 0x4C: this->absolute_addressing();    this->JMP();  cycles -= 3; break;
        case 0x4D: this->absolute_addressing();    this->EOR();  cycles -= 4; break;
        case 0x4E: this->absolute_addressing();    this->LSR();  cycles -= 6; break;
        case 0x50: this->relative_addressing();    this->BVC();  cycles -= 2; break;
        case 0x51: this->indirect_y_addressing();  this->EOR();  cycles -= 5; break;
        case 0x54: this->zero_page_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0x55: this->zero_page_x_addressing();  this->EOR();  cycles -= 4; break;
        case 0x56: this->zero_page_x_addressing();  this->LSR();  cycles -= 6; break;
        case 0x59: this->absolute_y_addressing();  this->EOR();  cycles -= 4; break;
        case 0x5A: this->accumulator_addressing(); this->NOP();  cycles -= 1; break;
        case 0x5C: this->absolute_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0x5D: this->absolute_x_addressing();  this->EOR();  cycles -= 4; break;
        case 0x5E: this->absolute_x_addressing();  this->LSR();  cycles -= 7; break;
        case 0x60: this->implied_addressing();     this->RTS();  cycles -= 6; break;
        case 0x61: this->indirect_x_addressing();  this->ADC();  cycles -= 6; break;
        case 0x64: this->zero_page_addressing();    this->NOP();  cycles -= 1; break;
        case 0x65: this->zero_page_addressing();    this->ADC();  cycles -= 3; break;
        case 0x66: this->zero_page_addressing();    this->ROR();  cycles -= 5; break;
        case 0x68: this->implied_addressing();     this->PLA();  cycles -= 4; break;
        case 0x69: this->immediate_addressing();   this->ADC();  cycles -= 2; break;
        case 0x6A: this->accumulator_addressing(); this->RORA(); cycles -= 2; break;
        case 0x6C: this->indirect_addressing();    this->JMP();  cycles -= 5; break;
        case 0x6D: this->absolute_addressing();    this->ADC();  cycles -= 4; break;
        case 0x6E: this->absolute_addressing();    this->ROR();  cycles -= 6; break;
        case 0x70: this->relative_addressing();    this->BVS();  cycles -= 2; break;
        case 0x71: this->indirect_y_addressing();  this->ADC();  cycles -= 5; break;
        case 0x74: this->zero_page_addressing();    this->NOP();  cycles -= 1; break;
        case 0x75: this->zero_page_x_addressing();  this->ADC();  cycles -= 4; break;
        case 0x76: this->zero_page_x_addressing();  this->ROR();  cycles -= 6; break;
        case 0x78: this->implied_addressing();     this->SEI();  cycles -= 2; break;
        case 0x79: this->absolute_y_addressing();  this->ADC();  cycles -= 4; break;
        case 0x7A: this->accumulator_addressing(); this->NOP();  cycles -= 1; break;
        case 0x7C: this->absolute_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0x7D: this->absolute_x_addressing();  this->ADC();  cycles -= 4; break;
        case 0x7E: this->absolute_x_addressing();  this->ROR();  cycles -= 7; break;
        case 0x80: this->immediate_addressing();   this->NOP();  cycles -= 1; break;
        case 0x81: this->indirect_x_addressing();  this->STA();  cycles -= 6; break;
        case 0x84: this->zero_page_addressing();    this->STY();  cycles -= 3; break;
        case 0x85: this->zero_page_addressing();    this->STA();  cycles -= 3; break;
        case 0x86: this->zero_page_addressing();    this->STX();  cycles -= 3; break;
        case 0x88: this->implied_addressing();     this->DEY();  cycles -= 2; break;
        case 0x8A: this->implied_addressing();     this->TXA();  cycles -= 2; break;
        case 0x8C: this->absolute_addressing();    this->STY();  cycles -= 4; break;
        case 0x8D: this->absolute_addressing();    this->STA();  cycles -= 4; break;
        case 0x8E: this->absolute_addressing();    this->STX();  cycles -= 4; break;
        case 0x90: this->relative_addressing();    this->BCC();  cycles -= 2; break;
        case 0x91: this->indirect_y_addressing();  this->STA();  cycles -= 6; break;
        case 0x94: this->zero_page_x_addressing();  this->STY();  cycles -= 4; break;
        case 0x95: this->zero_page_x_addressing();  this->STA();  cycles -= 4; break;
        case 0x96: this->zero_page_y_addressing();  this->STX();  cycles -= 4; break;
        case 0x98: this->implied_addressing();     this->TYA();  cycles -= 2; break;
        case 0x99: this->absolute_y_addressing();  this->STA();  cycles -= 5; break;
        case 0x9A: this->implied_addressing();     this->TXS();  cycles -= 2; break;
        case 0x9D: this->absolute_x_addressing();  this->STA();  cycles -= 5; break;
        case 0xA0: this->immediate_addressing();   this->LDY();  cycles -= 2; break;
        case 0xA1: this->indirect_x_addressing();  this->LDA();  cycles -= 6; break;
        case 0xA2: this->immediate_addressing();   this->LDX();  cycles -= 2; break;
        case 0xA4: this->zero_page_addressing();    this->LDY();  cycles -= 3; break;
        case 0xA5: this->zero_page_addressing();    this->LDA();  cycles -= 3; break;
        case 0xA6: this->zero_page_addressing();    this->LDX();  cycles -= 3; break;
        case 0xA8: this->implied_addressing();     this->TAY();  cycles -= 3; break;
        case 0xA9: this->immediate_addressing();   this->LDA();  cycles -= 2; break;
        case 0xAA: this->implied_addressing();     this->TAX();  cycles -= 2; break;
        case 0xAC: this->absolute_addressing();    this->LDY();  cycles -= 4; break;
        case 0xAD: this->absolute_addressing();    this->LDA();  cycles -= 4; break;
        case 0xAE: this->absolute_addressing();    this->LDX();  cycles -= 4; break;
        case 0xB0: this->relative_addressing();    this->BCS();  cycles -= 2; break;
        case 0xB1: this->indirect_y_addressing();  this->LDA();  cycles -= 5; break;
        case 0xB4: this->zero_page_x_addressing();  this->LDY();  cycles -= 4; break;
        case 0xB5: this->zero_page_x_addressing();  this->LDA();  cycles -= 4; break;
        case 0xB6: this->zero_page_y_addressing();  this->LDX();  cycles -= 4; break;
        case 0xB8: this->implied_addressing();     this->CLV();  cycles -= 2; break;
        case 0xB9: this->absolute_y_addressing();  this->LDA();  cycles -= 4; break;
        case 0xBA: this->implied_addressing();     this->TSX();  cycles -= 2; break;
        case 0xBC: this->absolute_x_addressing();  this->LDY();  cycles -= 4; break;
        case 0xBD: this->absolute_x_addressing();  this->LDA();  cycles -= 4; break;
        case 0xBE: this->absolute_y_addressing();  this->LDX();  cycles -= 4; break;
        case 0xC0: this->immediate_addressing();   this->CPY();  cycles -= 2; break;
        case 0xC1: this->indirect_x_addressing();  this->CMP();  cycles -= 6; break;
        case 0xC4: this->zero_page_addressing();    this->CPY();  cycles -= 3; break;
        case 0xC5: this->zero_page_addressing();    this->CMP();  cycles -= 3; break;
        case 0xC6: this->zero_page_addressing();    this->DEC();  cycles -= 5; break;
        case 0xC8: this->implied_addressing();     this->INY();  cycles -= 2; break;
        case 0xC9: this->immediate_addressing();   this->CMP();  cycles -= 2; break;
        case 0xCA: this->implied_addressing();     this->DEX();  cycles -= 2; break;
        case 0xCC: this->absolute_addressing();    this->CPY();  cycles -= 4; break;
        case 0xCD: this->absolute_addressing();    this->CMP();  cycles -= 4; break;
        case 0xCE: this->absolute_addressing();    this->DEC();  cycles -= 6; break;
        case 0xD0: this->relative_addressing();    this->BNE();  cycles -= 2; break;
        case 0xD1: this->indirect_y_addressing();  this->CMP();  cycles -= 5; break;
        case 0xD4: this->zero_page_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0xD5: this->zero_page_x_addressing();  this->CMP();  cycles -= 5; break;
        case 0xD6: this->zero_page_x_addressing();  this->DEC();  cycles -= 6; break;
        case 0xD8: this->implied_addressing();     this->CLD();  cycles -= 2; break;
        case 0xD9: this->absolute_y_addressing();  this->CMP();  cycles -= 4; break;
        case 0xDA: this->accumulator_addressing(); this->NOP();  cycles -= 1; break;
        case 0xDC: this->absolute_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0xDD: this->absolute_x_addressing();  this->CMP();  cycles -= 4; break;
        case 0xDE: this->absolute_x_addressing();  this->DEC();  cycles -= 7; break;
        case 0xE0: this->immediate_addressing();   this->CPX();  cycles -= 2; break;
        case 0xE1: this->indirect_x_addressing();  this->SBC();  cycles -= 6; break;
        case 0xE4: this->zero_page_addressing();    this->CPX();  cycles -= 3; break;
        case 0xE5: this->zero_page_addressing();    this->SBC();  cycles -= 3; break;
        case 0xE6: this->zero_page_addressing();    this->INC();  cycles -= 5; break;
        case 0xE8: this->implied_addressing();     this->INX();  cycles -= 2; break;
        case 0xE9: this->immediate_addressing();   this->SBC();  cycles -= 2; break;
        case 0xEA: this->accumulator_addressing(); this->NOP();  cycles -= 2; break;
        case 0xEC: this->absolute_addressing();    this->CPX();  cycles -= 4; break;
        case 0xED: this->absolute_addressing();    this->SBC();  cycles -= 4; break;
        case 0xEE: this->absolute_addressing();    this->INC();  cycles -= 6; break;
        case 0xF0: this->relative_addressing();    this->BEQ();  cycles -= 2; break;
        case 0xF1: this->indirect_y_addressing();  this->SBC();  cycles -= 5; break;
        case 0xF4: this->zero_page_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0xF5: this->zero_page_x_addressing();  this->SBC();  cycles -= 4; break;
        case 0xF6: this->zero_page_x_addressing();  this->INC();  cycles -= 6; break;
        case 0xF8: this->implied_addressing();     this->SED();  cycles -= 2; break;
        case 0xF9: this->absolute_y_addressing();  this->SBC();  cycles -= 4; break;
        case 0xFA: this->accumulator_addressing(); this->NOP();  cycles -= 1; break;
        case 0xFC: this->absolute_x_addressing();  this->NOP();  cycles -= 1; break;
        case 0xFD: this->absolute_x_addressing();  this->SBC();  cycles -= 4; break;
        case 0xFE: this->absolute_x_addressing();  this->INC();  cycles -= 7; break;
'''


import re


p = r'.*?case\s(?P<id>.*):.*?this->(?P<addr>.*)_addressing.*this->(?P<func>.*)\(\);.*cycles\s-=\s(?P<cycle>\d);\s.*'

if __name__ == '__main__':

    ins = a.split("break")

    for entry in ins:
        if "case" in entry:
            entry = entry.replace("\r\n", " ").replace("\n", " ")
            m = re.match(p, entry)
            # print(m.groupdict())
            d = m.groupdict()
            d["func"] = d["func"].lower()
            # d['addr'] = d['addr'].replace("absolute", "abs")
            s = "{id} => {{ self.{addr}();\tself.{func}();\tself.cycles_delay+={cycle}; }},".format(**d)
            print(s)




