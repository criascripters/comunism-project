// rosquita no terminal
// apice da computaria moderna
use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;

fn main() {
    let mut a = 0.0_f32;
    let mut b = 0.0_f32;

    let mut z = [0.0_f32; 1760];
    let mut buffer = [' '; 1760];

    print!("\x1b[2J");

    loop {
        buffer.fill(' ');
        z.fill(0.0);

        let mut j = 0.0_f32;
        while j < 6.28 {
            let mut i = 0.0_f32;
            while i < 6.28 {
                let c = i.sin();
                let d = j.cos();
                let e = j.sin();
                let f = a.sin();
                let g = a.cos();
                let h = d + 2.0;
                let dd = 1.0 / (c * h * f + e * g + 5.0);
                let l = i.cos();
                let m = b.cos();
                let n = b.sin();
                let t = c * h * g - e * f;

                let x = ((40.0 + 30.0 * dd * (l * h * m - t * n)) as i32) as usize;
                let y = ((12.0 + 15.0 * dd * (l * h * n + t * m)) as i32) as usize;
                let o = (x + 80 * y) as usize;
                let nn = (8.0 * ((e * f - c * d * g) * m - c * d * f - e * g - l * d * n)) as i32;

                if 22 > y && y > 0 && x > 0 && 80 > x && dd > z[o] {
                    z[o] = dd;
                    buffer[o] = ".,-~:;=!*#$@"
                        .chars()
                        .nth(if nn > 0 { nn as usize } else { 0 })
                        .unwrap_or(' ');
                }

                i += 0.02;
            }
            j += 0.07;
        }

        print!("\x1b[H");
        for k in 0..1760 {
            print!("{}", if k % 80 != 0 { buffer[k] } else { '\n' });
        }

        stdout().flush().unwrap();

        a += 0.04;
        b += 0.02;

        thread::sleep(Duration::from_millis(30));
    }
}
