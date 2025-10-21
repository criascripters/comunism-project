use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;

fn main() {
    let mut a = 0.0_f32;
    let mut b = 0.0_f32;

    // dimensões menores para caber em overlays
    let width = 40;
    let height = 18;
    let size = width * height;

    let mut z = vec![0.0_f32; size];
    let mut buffer = vec![' '; size];

    print!("\x1b[2J"); // limpa a tela
    stdout().flush().unwrap();

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

                // escala ajustada para o tamanho menor
                let x = ((width as f32 / 2.0) + (width as f32 / 3.0) * dd * (l * h * m - t * n))
                    as usize;
                let y = ((height as f32 / 2.0) + (height as f32 / 2.5) * dd * (l * h * n + t * m))
                    as usize;

                let o = x + width * y;
                let nn = (8.0 * ((e * f - c * d * g) * m - c * d * f - e * g - l * d * n)) as i32;

                if y < height && x < width && o < size && dd > z[o] {
                    z[o] = dd;
                    buffer[o] = ".,-~:;=!*#$@"
                        .chars()
                        .nth(if nn > 0 { nn as usize } else { 0 })
                        .unwrap_or(' ');
                }

                i += 0.04; // passo maior = menos pontos, mas ainda suave
            }
            j += 0.07;
        }

        // volta pro inicio da tela
        print!("\x1b[H");

        // renderiza
        for k in 0..size {
            if k > 0 && k % width == 0 {
                print!(
                    "
"
                );
            } else {
                print!("{}", buffer[k]);
            }
        }

        stdout().flush().unwrap();

        a += 0.04;
        b += 0.02;

        thread::sleep(Duration::from_millis(30));
    }
}
