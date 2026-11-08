const MAX_ZOOM: f64 = 1.0e24;

#[derive(Clone, Copy, Debug, Default)]
pub struct Precise {
    hi: f64,
    lo: f64,
}

impl Precise {
    pub fn from_f64(value: f64) -> Self {
        Self { hi: value, lo: 0.0 }
    }

    pub fn to_f64(self) -> f64 {
        self.hi + self.lo
    }

    pub fn add(self, other: Self) -> Self {
        let sum = self.hi + other.hi;
        let virtual_other = sum - self.hi;
        let error =
            (self.hi - (sum - virtual_other)) + (other.hi - virtual_other) + self.lo + other.lo;
        let hi = sum + error;
        let lo = error - (hi - sum);
        Self { hi, lo }
    }

    pub fn sub(self, other: Self) -> Self {
        self.add(Self {
            hi: -other.hi,
            lo: -other.lo,
        })
    }

    pub fn mul(self, other: Self) -> Self {
        let product = self.hi * other.hi;
        let error = self.hi.mul_add(other.hi, -product)
            + self.hi * other.lo
            + self.lo * other.hi
            + self.lo * other.lo;
        let hi = product + error;
        let lo = error - (hi - product);
        Self { hi, lo }
    }

    pub fn add_f64(self, value: f64) -> Self {
        self.add(Self::from_f64(value))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ComplexPoint {
    pub x: Precise,
    pub y: Precise,
}

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub center: ComplexPoint,
    pub zoom: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: ComplexPoint {
                x: Precise::from_f64(-0.5),
                y: Precise::from_f64(0.0),
            },
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn units_per_pixel(self, height: f64) -> f64 {
        4.0 / self.zoom / height.max(1.0)
    }

    pub fn complex_at(self, x: f64, y: f64, width: f64, height: f64) -> ComplexPoint {
        let units = self.units_per_pixel(height);
        ComplexPoint {
            x: self.center.x.add_f64((x - width / 2.0) * units),
            y: self.center.y.add_f64((height / 2.0 - y) * units),
        }
    }

    pub fn zoom_at(&mut self, x: f64, y: f64, width: f64, height: f64, factor: f64) {
        let anchor = self.complex_at(x, y, width, height);
        self.zoom = (self.zoom * factor.clamp(0.2, 5.0)).clamp(0.01, MAX_ZOOM);
        let units = self.units_per_pixel(height);
        self.center.x = anchor.x.add_f64(-(x - width / 2.0) * units);
        self.center.y = anchor.y.add_f64(-(height / 2.0 - y) * units);
    }

    pub fn pan_pixels(&mut self, dx: f64, dy: f64, height: f64) {
        let units = self.units_per_pixel(height);
        self.center.x = self.center.x.add_f64(-dx * units);
        self.center.y = self.center.y.add_f64(dy * units);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_the_complex_point_under_the_cursor_fixed() {
        let mut camera = Camera::default();
        let before = camera.complex_at(137.0, 211.0, 800.0, 600.0);
        camera.zoom_at(137.0, 211.0, 800.0, 600.0, 2.0);
        let after = camera.complex_at(137.0, 211.0, 800.0, 600.0);
        assert!(before.x.sub(after.x).to_f64().abs() < 1e-15);
        assert!(before.y.sub(after.y).to_f64().abs() < 1e-15);
    }

    #[test]
    fn extended_coordinates_preserve_cursor_anchor_at_deep_zoom() {
        let mut camera = Camera::default();
        let anchor = camera.complex_at(137.0, 211.0, 800.0, 600.0);
        for _ in 0..80 {
            camera.zoom_at(137.0, 211.0, 800.0, 600.0, 2.0);
        }
        let after = camera.complex_at(137.0, 211.0, 800.0, 600.0);
        assert!((camera.zoom - 1.0e24).abs() < 1.0e10);
        assert!(anchor.x.sub(after.x).to_f64().abs() < 1e-30);
        assert!(anchor.y.sub(after.y).to_f64().abs() < 1e-30);
    }
}
