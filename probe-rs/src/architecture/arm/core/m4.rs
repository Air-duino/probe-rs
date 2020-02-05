use crate::core::Breakpoint;
use crate::core::{
    BasicRegisterAddresses, CoreInformation, CoreInterface, CoreRegister, CoreRegisterAddress,
};
use crate::error::Error;
use crate::memory::Memory;
use crate::{DebugProbeError, Session};
use bitfield::bitfield;

use std::mem::size_of;

bitfield! {
    #[derive(Copy, Clone)]
    pub struct Dhcsr(u32);
    impl Debug;
    pub s_reset_st, _: 25;
    pub s_retire_st, _: 24;
    pub s_lockup, _: 19;
    pub s_sleep, _: 18;
    pub s_halt, _: 17;
    pub s_regrdy, _: 16;
    pub c_snapstall, set_c_snapstall: 5;
    pub c_maskings, set_c_maskints: 3;
    pub c_step, set_c_step: 2;
    pub c_halt, set_c_halt: 1;
    pub c_debugen, set_c_debugen: 0;
}

impl Dhcsr {
    /// This function sets the bit to enable writes to this register.
    ///
    /// C1.6.3 Debug Halting Control and Status Register, DHCSR:
    /// Debug key:
    /// Software must write 0xA05F to this field to enable write accesses to bits
    /// [15:0], otherwise the processor ignores the write access.
    pub fn enable_write(&mut self) {
        self.0 &= !(0xffff << 16);
        self.0 |= 0xa05f << 16;
    }
}

impl From<u32> for Dhcsr {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Dhcsr> for u32 {
    fn from(value: Dhcsr) -> Self {
        value.0
    }
}

impl CoreRegister for Dhcsr {
    const ADDRESS: u32 = 0xE000_EDF0;
    const NAME: &'static str = "DHCSR";
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct Dcrsr(u32);
    impl Debug;
    pub _, set_regwnr: 16;
    pub _, set_regsel: 6,0;
}

impl From<u32> for Dcrsr {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Dcrsr> for u32 {
    fn from(value: Dcrsr) -> Self {
        value.0
    }
}

impl CoreRegister for Dcrsr {
    const ADDRESS: u32 = 0xE000_EDF4;
    const NAME: &'static str = "DCRSR";
}

#[derive(Debug, Copy, Clone)]
pub struct Dcrdr(u32);

impl From<u32> for Dcrdr {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Dcrdr> for u32 {
    fn from(value: Dcrdr) -> Self {
        value.0
    }
}

impl CoreRegister for Dcrdr {
    const ADDRESS: u32 = 0xE000_EDF8;
    const NAME: &'static str = "DCRDR";
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct Aircr(u32);
    impl Debug;
    pub get_vectkeystat, set_vectkey: 31,16;
    pub endianness, set_endianness: 15;
    pub prigroup, set_prigroup: 10,8;
    pub sysresetreq, set_sysresetreq: 2;
    pub vectclractive, set_vectclractive: 1;
    pub vectreset, set_vectreset: 0;
}

impl From<u32> for Aircr {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Aircr> for u32 {
    fn from(value: Aircr) -> Self {
        value.0
    }
}

impl Aircr {
    pub fn vectkey(&mut self) {
        self.set_vectkey(0x05FA);
    }

    pub fn vectkeystat(&self) -> bool {
        self.get_vectkeystat() == 0xFA05
    }
}

impl CoreRegister for Aircr {
    const ADDRESS: u32 = 0xE000_ED0C;
    const NAME: &'static str = "AIRCR";
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct Demcr(u32);
    impl Debug;
    /// Global enable for DWT and ITM features
    pub trcena, set_trcena: 24;
    /// DebugMonitor semaphore bit
    pub mon_req, set_mon_req: 19;
    /// Step the processor?
    pub mon_step, set_mon_step: 18;
    /// Sets or clears the pending state of the DebugMonitor exception
    pub mon_pend, set_mon_pend: 17;
    /// Enable the DebugMonitor exception
    pub mon_en, set_mon_en: 16;
    /// Enable halting debug trap on a HardFault exception
    pub vc_harderr, set_vc_harderr: 10;
    /// Enable halting debug trap on a fault occurring during exception entry
    /// or exception return
    pub vc_interr, set_vc_interr: 9;
    /// Enable halting debug trap on a BusFault exception
    pub vc_buserr, set_vc_buserr: 8;
    /// Enable halting debug trap on a UsageFault exception caused by a state
    /// information error, for example an Undefined Instruction exception
    pub vc_staterr, set_vc_staterr: 7;
    /// Enable halting debug trap on a UsageFault exception caused by a
    /// checking error, for example an alignment check error
    pub vc_chkerr, set_vc_chkerr: 6;
    /// Enable halting debug trap on a UsageFault caused by an access to a
    /// Coprocessor
    pub vc_nocperr, set_vc_nocperr: 5;
    /// Enable halting debug trap on a MemManage exception.
    pub vc_mmerr, set_vc_mmerr: 4;
    /// Enable Reset Vector Catch
    pub vc_corereset, set_vc_corereset: 0;
}

impl From<u32> for Demcr {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Demcr> for u32 {
    fn from(value: Demcr) -> Self {
        value.0
    }
}

impl CoreRegister for Demcr {
    const ADDRESS: u32 = 0xe000_edfc;
    const NAME: &'static str = "DEMCR";
}

bitfield! {
    #[derive(Copy,Clone)]
    pub struct FpCtrl(u32);
    impl Debug;

    pub rev, _: 31, 28;
    num_code_1, _: 14, 12;
    pub num_lit, _: 11, 8;
    num_code_0, _: 7, 4;
    pub _, set_key: 1;
    pub enable, set_enable: 0;
}

impl FpCtrl {
    pub fn num_code(&self) -> u32 {
        (self.num_code_1() << 4) | self.num_code_0()
    }
}

impl CoreRegister for FpCtrl {
    const ADDRESS: u32 = 0xE000_2000;
    const NAME: &'static str = "FP_CTRL";
}

impl From<u32> for FpCtrl {
    fn from(value: u32) -> Self {
        FpCtrl(value)
    }
}

impl From<FpCtrl> for u32 {
    fn from(value: FpCtrl) -> Self {
        value.0
    }
}
bitfield! {
    #[derive(Copy,Clone)]
    pub struct FpCompX(u32);
    impl Debug;

    pub replace, set_replace: 31, 30;
    pub comp, set_comp: 28, 2;
    pub enable, set_enable: 0;
}

impl CoreRegister for FpCompX {
    const ADDRESS: u32 = 0xE000_2008;
    const NAME: &'static str = "FP_CTRL";
}

impl From<u32> for FpCompX {
    fn from(value: u32) -> Self {
        FpCompX(value)
    }
}

impl From<FpCompX> for u32 {
    fn from(value: FpCompX) -> Self {
        value.0
    }
}

impl FpCompX {
    /// Get the correct register configuration which enables
    /// a hardware breakpoint at the given address.
    fn breakpoint_configuration(address: u32) -> Self {
        let mut reg = FpCompX::from(0);

        let comp_val = (address & 0x1f_ff_ff_fc) >> 2;

        // the replace value decides if the upper or lower half
        // word is matched for the break point
        let replace_val = if (address & 0x3) == 0 {
            0b01 // lower half word
        } else {
            0b10 // upper half word
        };

        reg.set_replace(replace_val);
        reg.set_comp(comp_val);
        reg.set_enable(true);

        reg
    }
}

pub const REGISTERS: BasicRegisterAddresses = BasicRegisterAddresses {
    R0: CoreRegisterAddress(0b000_0000),
    R1: CoreRegisterAddress(0b000_0001),
    R2: CoreRegisterAddress(0b000_0010),
    R3: CoreRegisterAddress(0b000_0011),
    R4: CoreRegisterAddress(0b000_0100),
    R5: CoreRegisterAddress(0b0_0101),
    R6: CoreRegisterAddress(0b0_0110),
    R7: CoreRegisterAddress(0b0_0111),
    R8: CoreRegisterAddress(0b0_1000),
    R9: CoreRegisterAddress(0b000_1001),
    PC: CoreRegisterAddress(0b000_1111),
    SP: CoreRegisterAddress(0b000_1101),
    LR: CoreRegisterAddress(0b000_1110),
    XPSR: CoreRegisterAddress(0b001_0000),
};

pub const MSP: CoreRegisterAddress = CoreRegisterAddress(0b000_1001);
pub const PSP: CoreRegisterAddress = CoreRegisterAddress(0b000_1010);

#[derive(Clone)]
pub struct M4 {
    memory: Memory,
    session: Session,

    hw_breakpoints_enabled: bool,
    active_breakpoints: Vec<Breakpoint>,
}

impl M4 {
    pub fn new(session: Session, memory: Memory) -> Self {
        Self {
            session,
            memory,
            hw_breakpoints_enabled: false,
            active_breakpoints: vec![],
        }
    }

    fn wait_for_core_register_transfer(&self) -> Result<(), Error> {
        // now we have to poll the dhcsr register, until the dhcsr.s_regrdy bit is set
        // (see C1-292, cortex m0 arm)
        for _ in 0..100 {
            let dhcsr_val = Dhcsr(self.memory.read32(Dhcsr::ADDRESS)?);

            if dhcsr_val.s_regrdy() {
                return Ok(());
            }
        }
        Err(Error::Probe(DebugProbeError::Timeout))
    }
}

impl CoreInterface for M4 {
    fn wait_for_core_halted(&self) -> Result<(), Error> {
        // Wait until halted state is active again.
        for _ in 0..100 {
            let dhcsr_val = Dhcsr(self.memory.read32(Dhcsr::ADDRESS)?);
            if dhcsr_val.s_halt() {
                return Ok(());
            }
        }
        Err(Error::Probe(DebugProbeError::Timeout))
    }

    fn core_halted(&self) -> Result<bool, Error> {
        // Wait until halted state is active again.
        let dhcsr_val = Dhcsr(self.memory.read32(Dhcsr::ADDRESS)?);

        if dhcsr_val.s_halt() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn read_core_reg(&self, addr: CoreRegisterAddress) -> Result<u32, Error> {
        // Write the DCRSR value to select the register we want to read.
        let mut dcrsr_val = Dcrsr(0);
        dcrsr_val.set_regwnr(false); // Perform a read.
        dcrsr_val.set_regsel(addr.into()); // The address of the register to read.

        self.memory.write32(Dcrsr::ADDRESS, dcrsr_val.into())?;

        self.wait_for_core_register_transfer()?;

        self.memory.read32(Dcrdr::ADDRESS).map_err(From::from)
    }

    fn write_core_reg(&self, addr: CoreRegisterAddress, value: u32) -> Result<(), Error> {
        let result: Result<(), Error> = self
            .memory
            .write32(Dcrdr::ADDRESS, value)
            .map_err(From::from);
        result?;

        // write the DCRSR value to select the register we want to write.
        let mut dcrsr_val = Dcrsr(0);
        dcrsr_val.set_regwnr(true); // Perform a write.
        dcrsr_val.set_regsel(addr.into()); // The address of the register to write.

        self.memory.write32(Dcrsr::ADDRESS, dcrsr_val.into())?;

        self.wait_for_core_register_transfer()
    }

    fn halt(&self) -> Result<CoreInformation, Error> {
        // TODO: Generic halt support

        let mut value = Dhcsr(0);
        value.set_c_halt(true);
        value.set_c_debugen(true);
        value.enable_write();

        self.memory.write32(Dhcsr::ADDRESS, value.into())?;

        self.wait_for_core_halted()?;

        // try to read the program counter
        let pc_value = self.read_core_reg(REGISTERS.PC)?;

        // get pc
        Ok(CoreInformation { pc: pc_value })
    }

    fn run(&self) -> Result<(), Error> {
        let mut value = Dhcsr(0);
        value.set_c_halt(false);
        value.set_c_debugen(true);
        value.enable_write();

        self.memory
            .write32(Dhcsr::ADDRESS, value.into())
            .map_err(Into::into)
    }

    fn step(&self) -> Result<CoreInformation, Error> {
        let mut value = Dhcsr(0);
        // Leave halted state.
        // Step one instruction.
        value.set_c_step(true);
        value.set_c_halt(false);
        value.set_c_debugen(true);
        value.set_c_maskints(true);
        value.enable_write();

        self.memory.write32(Dhcsr::ADDRESS, value.into())?;

        self.wait_for_core_halted()?;

        // try to read the program counter
        let pc_value = self.read_core_reg(REGISTERS.PC)?;

        // get pc
        Ok(CoreInformation { pc: pc_value })
    }

    fn reset(&self) -> Result<(), Error> {
        // Set THE AIRCR.SYSRESETREQ control bit to 1 to request a reset. (ARM V6 ARM, B1.5.16)
        let mut value = Aircr(0);
        value.vectkey();
        value.set_sysresetreq(true);

        self.memory.write32(Aircr::ADDRESS, value.into())?;

        Ok(())
    }

    fn reset_and_halt(&self) -> Result<CoreInformation, Error> {
        // Ensure debug mode is enabled
        let dhcsr_val = Dhcsr(self.memory.read32(Dhcsr::ADDRESS)?);
        if !dhcsr_val.c_debugen() {
            let mut dhcsr = Dhcsr(0);
            dhcsr.set_c_debugen(true);
            dhcsr.enable_write();
            self.memory.write32(Dhcsr::ADDRESS, dhcsr.into())?;
        }

        // Set the vc_corereset bit in the DEMCR register.
        // This will halt the core after reset.
        let demcr_val = Demcr(self.memory.read32(Demcr::ADDRESS)?);
        if !demcr_val.vc_corereset() {
            let mut demcr_enabled = demcr_val;
            demcr_enabled.set_vc_corereset(true);
            self.memory.write32(Demcr::ADDRESS, demcr_enabled.into())?;
        }

        self.reset()?;

        self.wait_for_core_halted()?;

        const XPSR_THUMB: u32 = 1 << 24;
        let xpsr_value = self.read_core_reg(REGISTERS.XPSR)?;
        if xpsr_value & XPSR_THUMB == 0 {
            self.write_core_reg(REGISTERS.XPSR, xpsr_value | XPSR_THUMB)?;
        }

        self.memory.write32(Demcr::ADDRESS, demcr_val.into())?;

        // try to read the program counter
        let pc_value = self.read_core_reg(REGISTERS.PC)?;

        // get pc
        Ok(CoreInformation { pc: pc_value })
    }

    fn get_available_breakpoint_units(&self) -> Result<u32, Error> {
        let raw_val = self.memory.read32(FpCtrl::ADDRESS)?;

        let reg = FpCtrl::from(raw_val);

        // We currently only support revision 0 of the FPBU, so we return an error
        if reg.rev() == 0 {
            Ok(reg.num_code())
        } else {
            log::warn!("This chip uses FPBU revision {}, which is not yet supported. HW breakpoints are not available.", reg.rev());
            Err(Error::Probe(DebugProbeError::Unknown))
        }
    }

    fn enable_breakpoints(&mut self, state: bool) -> Result<(), Error> {
        let mut val = FpCtrl::from(0);
        val.set_key(true);
        val.set_enable(state);

        self.memory.write32(FpCtrl::ADDRESS, val.into())?;

        self.hw_breakpoints_enabled = true;

        Ok(())
    }

    fn set_breakpoint(&self, bp_unit_index: usize, addr: u32) -> Result<(), Error> {
        let val = FpCompX::breakpoint_configuration(addr);

        let reg_addr = FpCompX::ADDRESS + (bp_unit_index * size_of::<u32>()) as u32;

        self.memory.write32(reg_addr, val.into())?;

        Ok(())
    }

    fn registers<'a>(&self) -> &'a BasicRegisterAddresses {
        &REGISTERS
    }

    fn clear_breakpoint(&self, bp_unit_index: usize) -> Result<(), Error> {
        let mut val = FpCompX::from(0);
        val.set_enable(false);

        let reg_addr = FpCompX::ADDRESS + (bp_unit_index * size_of::<u32>()) as u32;

        self.memory.write32(reg_addr, val.into())?;

        Ok(())
    }

    fn memory(&self) -> Memory {
        self.memory.clone()
    }
    
    fn hw_breakpoints_enabled(&self) -> bool {
        self.hw_breakpoints_enabled
    }
}

#[test]
fn breakpoint_register_value() {
    // Check that the register configuration for the FPBU is
    // calculated correctly.
    //
    // See ARMv7 Architecture Reference Manual, Section C1.11.5
    let address: u32 = 0x0800_09A4;

    let reg = FpCompX::breakpoint_configuration(address);
    let reg_val: u32 = reg.into();

    assert_eq!(0x4800_09A5, reg_val);
}
