// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use gen_frames::box_resize::box_resize;

fn ramp(sw: usize, sh: usize) -> Vec<u8> {
    (0..sh)
        .flat_map(|y| (0..sw).map(move |x| (((y * sw + x) * 17) % 256) as u8))
        .collect()
}

fn check(sw: usize, sh: usize, dw: usize, dh: usize, expected: &[u8]) {
    let got = box_resize(&ramp(sw, sh), sw, sh, dw, dh);
    assert_eq!(got, expected, "{sw}x{sh} -> {dw}x{dh}");
}

#[test]
fn matches_pillow_box_vectors() {
    check(4, 4, 2, 2, &[43, 77, 179, 213]);
    check(5, 3, 2, 2, &[60, 103, 187, 230]);
    check(3, 3, 2, 2, &[35, 60, 111, 136]);
    check(8, 8, 3, 2, &[93, 136, 157, 125, 168, 103]);
    check(9, 9, 3, 3, &[85, 136, 130, 145, 111, 134, 149, 143, 109]);
    check(2, 2, 2, 2, &[0, 17, 34, 51]);
    check(10, 1, 3, 1, &[17, 77, 136]);
    check(1, 10, 1, 3, &[17, 77, 136]);
    check(5, 5, 2, 2, &[102, 145, 101, 102]);
    let special: Vec<u8> = (0..8)
        .flat_map(|i| (0..8).map(move |j| ((i * 3 + j * 5) % 256) as u8))
        .collect();
    let got = box_resize(&special, 8, 8, 3, 2);
    assert_eq!(got, &[10, 23, 35, 22, 35, 47]);
}
