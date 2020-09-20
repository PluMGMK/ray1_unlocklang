extern crate pmw1;

use std::env::args;
use std::io::prelude::*;
use std::fs::{File,OpenOptions};

use pmw1::exe::Pmw1Exe;

fn main() -> std::io::Result<()> {
    // Assume the filename of interest is the LAST argument on the command line.
    let exe_name: String = args().next_back().unwrap();

    // Load the whole EXE into memory...
    let binary = {
        println!("Opening {}...", exe_name);

        let mut file = File::open(&exe_name)?;
        let mut buffer: Vec<u8> = Vec::with_capacity(0x100000);
        file.read_to_end(&mut buffer)?;
        buffer.shrink_to_fit();
        buffer
    };

    println!("{} is {} bytes.", exe_name, binary.len());

    assert_eq!(binary[0..2],b"MZ"[..],
               "{} is not an MZ executable!", exe_name);
    assert!(binary.len() >= 0x1c,
            "{} doesn't appear to contain a complete MZ header!",exe_name);

    let mz_header = &binary[0x2..0x1c];
    let mz_header: Vec<u16> = (0..mz_header.len())
        .step_by(2)
        .map(|i| u16::from_le_bytes([mz_header[i], mz_header[i+1]]))
        .collect();

    // Print out some relevant info.
    println!("It begins with an MZ executable, of {} half-KiB blocks.",
             mz_header[1]);
    let total_block_size = mz_header[1] << 9; // Shift left to multiply by 512
    let actual_mz_size =
        if mz_header[0] == 0 {
            println!("Last block is fully used.");
            total_block_size
        } else {
            println!("{} bytes used in last block.", mz_header[0]);
            total_block_size - 512 + mz_header[0]
        } as usize;
    println!("Total MZ executable size is {} bytes.", actual_mz_size);

    assert!(binary.len() > actual_mz_size, "This appears to be a pure MZ executable!");

    // A slice containing just the PMW1 part.
    let mut pmw1_exe = Pmw1Exe::from_bytes(&binary[actual_mz_size..])?;

    pmw1_exe.entry_object_mut()
        .update_data(|data| {
            let mut datavec = data.to_vec();

            // Find where it's zeroing the language setting.
            let zerolang_ops = &mut datavec[0x3939f..0x393a9];
            let expected_ops = &[0x30, 0xff, // xor bh,bh
                                0x31, 0xd2, // xor edx,edx
                                0x88, 0x3d, 0x35, 0xfa, 0x03, 0x00]; // mov ds:0x3fa35, bh
            let corrected_ops = &[0x90, 0x90, 0x90, 0x90, // nop (x4)
                                0x8a, 0x15, 0x35, 0xfa, 0x03, 0x00]; // mov dl, ds:0x3fa35
            assert_ne!(zerolang_ops, corrected_ops,
                       "EXE is already patched!");
            assert_eq!(zerolang_ops, expected_ops,
                       "EXE doesn't look like the Rayman 1 GOG version!");

            // Now that we're happy we need to patch it, load in our substitute bytes
            zerolang_ops.copy_from_slice(corrected_ops);

            datavec
        })?;

    // Create a backup file.
    {
        let filename = format!("{}.BAK",exe_name);
        println!("");
        println!("Attempting to create NEW backup file {}", filename);
        // `create_new` to fail if the backup file already exists.
        // Don't wanna screw up an existing backup...
        let mut outfile = OpenOptions::new().write(true)
                                            .create_new(true)
                                            .open(&filename)?;
        // Write the whole binary back out
        outfile.write_all(&binary)?;
        println!("Backup successful");
    }

    // Write out the patched EXE.
    {
        println!("");
        println!("Attempting to write patched code back to {}", exe_name);
        let mut outfile = File::create(&exe_name)?;
        // Write the DOS stub back out
        outfile.write_all(&binary[..actual_mz_size])?;
        // And the actual PMW1 exe!
        outfile.write_all(&pmw1_exe.as_bytes())?;
        println!("Patching successful");
    }

    Ok(())
}
