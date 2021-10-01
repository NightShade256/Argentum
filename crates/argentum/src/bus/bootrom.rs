/// SameBoot for the DMG, made by LIJI.
pub static DMG_BOOT_ROM: &[u8; 0x100] = include_bytes!("bootrom/dmg_boot.bin");

/// SameBoot for the CGB, made by LIJI.
pub static CGB_BOOT_ROM: &[u8; 0x900] = include_bytes!("bootrom/cgb_boot.bin");
