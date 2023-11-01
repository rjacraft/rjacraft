use rjacraft_protocol::chunk::*;

fn generate_number(sec: &mut Section<u32, 16>, n: i32, mut x: usize, y: usize, z: usize) {
    const BLOCK: u32 = 9235;

    let string = n.to_string();

    for c in string.chars() {
        match c {
            '-' => {
                //
                //
                // xxx
                //
                //
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
            }
            '0' => {
                // xxx
                // x x
                // x x
                // x x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 0] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '1' => {
                //  x
                //  x
                //  x
                //  x
                //  x
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 3][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 1][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
            }
            '2' => {
                // xxx
                //   x
                // xxx
                // x
                // xxx
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
            }
            '3' => {
                // xxx
                //   x
                // xxx
                //   x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '4' => {
                // x x
                // x x
                // xxx
                //   x
                //   x
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '5' => {
                // xxx
                // x
                // xxx
                //   x
                // xxx
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
            }
            '6' => {
                // xxx
                // x
                // xxx
                // x x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 0] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '7' => {
                // xxx
                //   x
                //   x
                //  x
                //  x
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
            }
            '8' => {
                // xxx
                // x x
                // xxx
                // x x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '9' => {
                // xxx
                // x x
                // xxx
                //   x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            _ => panic!(),
        }

        x += 4;
    }
}

pub fn generate(output: &mut Column<16>, chunk_x: i32, chunk_z: i32) {
    for sx in 0..16 {
        for y in 0..256 {
            for sz in 0..16 {
                let x = 16 * chunk_x + sx as i32;
                let z = 16 * chunk_z + sz as i32;
                let s = y / 16;
                let sy = y % 16;

                if y <= 40 {
                    // stone base
                    output.blockstates[s][sy][sz][sx] = 1;
                } else if y == 41 {
                    // planks
                    // 0123456789abcdef
                    //  xx xx xx xx xx
                    if sx % 3 != 0 && sz % 3 != 0 {
                        output.blockstates[s][sy][sz][sx] = 15;
                    }
                } else if y == 42 {
                    // a stained glass pattern with a prime interval
                    output.blockstates[s][sy][sz][sx] = 5946 + i32::unsigned_abs((x + z) % 13);
                }

                output.sky_light.world[s][sy][sz][sx] =
                    if sx < 8 { 8 + sx as u8 } else { 22 - sx as u8 };
            }
        }
    }

    // the chunk coordinates
    generate_number(&mut output.blockstates[4], chunk_x, 1, 7, 1);
    generate_number(&mut output.blockstates[4], chunk_z, 1, 1, 1);

    // single-valued section
    if chunk_x == 310 && chunk_z == -64 {
        for sx in 0..16 {
            for sy in 0..16 {
                for sz in 0..16 {
                    output.blockstates[3][sy][sz][sx] = 1;
                }
            }
        }
    }

    // noisy section, biggest possible LUT
    if chunk_x == 312 && chunk_z == -64 {
        for sx in 0..16 {
            for sy in 0..16 {
                for sz in 0..16 {
                    output.blockstates[3][sy][sz][sx] = ((16 * sy) + sz) as u32;
                }
            }
        }
    }

    // noisy section, biggest possible array
    if chunk_x == 314 && chunk_z == -64 {
        for sx in 0..16 {
            for sy in 0..16 {
                for sz in 0..16 {
                    output.blockstates[3][sy][sz][sx] = ((16 * 16 * sx) + (16 * sy) + sz) as u32;
                }
            }
        }
    }
}
