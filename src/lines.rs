pub fn bresenham([x0, y0]: [i32; 2], [x1, y1]: [i32; 2]) -> impl Iterator<Item = [i32; 2]> {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    let mut cx = x0;
    let mut cy = y0;

    let first = [[cx, cy]].into_iter();

    let max_iters = ((x1 - x0).abs()).max((y1 - y0).abs());
    let rest = (0..=max_iters).map_while(move |_| {
        if cx == x1 && cy == y1 {
            return None;
        }

        let e2 = 2 * error;
        if e2 >= dy {
            if cx == x1 {
                return None;
            }
            error += dy;
            cx += sx;
        }

        if e2 <= dx {
            if cy == y1 {
                return None;
            }
            error += dx;
            cy += sy;
        }

        Some([cx, cy])
    });
    first.chain(rest)
}

// returns coordinates in whatever input coordinate system is given.
pub fn wu([x0, y0]: [f32; 2], [x1, y1]: [f32; 2]) -> impl Iterator<Item = [i32; 2]> {
    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    let (x0, y0, x1, y1) = if steep {
        (y0, x0, y1, x1)
    } else {
        (x0, y0, x1, y1)
    };
    let (x0, y0, x1, y1) = if x0 > x1 {
        (x1, y1, x0, y0)
    } else {
        (x0, y0, x1, y1)
    };

    let dx = x1 - x0;
    let dy = y1 - y0;

    // TODO maybe use an epsilon here
    let grad = if dx.abs() < 1e-4 { 1. } else { dy / dx };

    // first endpoint
    let x_end = x0.round();
    let y_end = y0 + grad * (x_end - x0);
    let xpxl1 = x0 as i32;
    let ypxl1 = y_end.floor() as i32;
    let iter = if steep {
        [[ypxl1, xpxl1], [ypxl1 + 1, xpxl1]]
    } else {
        [[xpxl1, ypxl1], [xpxl1, ypxl1 + 1]]
    }
    .into_iter();

    let inter_y = y_end + grad;

    // second endpoint
    let x_end = x1.round();
    let y_end = y1 + grad * (x_end * x1);
    let xpxl2 = x_end as i32;
    let ypxl2 = y_end.floor() as i32;
    let end_iter = if steep {
        [[ypxl2, xpxl2], [ypxl2 + 1, xpxl2]]
    } else {
        [[xpxl2, ypxl2], [xpxl2, ypxl2 + 1]]
    }
    .into_iter();

    let inner = (xpxl1 + 1..xpxl2).enumerate().flat_map(move |(i, x)| {
        let iy = (inter_y + i as f32 * grad).floor() as i32;
        if steep {
            [[iy, x], [iy + 1, x]]
        } else {
            [[x, iy], [x, iy + 1]]
        }
        .into_iter()
    });
    iter.chain(end_iter).chain(inner)
}
