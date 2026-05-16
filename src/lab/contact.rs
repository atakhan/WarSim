use super::field::{FormationField, PressureProfile, pressure_profile_weight};

#[derive(Clone, Copy, Debug)]
pub struct ContactRequest {
    pub front_column: usize,
    pub rows: usize,
    pub row_range: ContactRowRange,
    pub incoming_profile: PressureProfile,
    pub normal_pressure: f32,
    pub compression: f32,
    pub disruption: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ContactRowRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub struct ContactBoundary {
    pub front_column: usize,
    pub samples: Vec<ContactSample>,
}

#[derive(Clone, Copy, Debug)]
pub struct ContactSample {
    pub row: usize,
    pub normal_pressure: f32,
    pub compression: f32,
    pub disruption: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ContactFront {
    pub front_column: usize,
    pub rows: usize,
    pub row_spacing: f32,
    pub lateral_center: f32,
    pub front_position: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ContactDetection {
    pub contact_distance: f32,
    pub base_pressure: f32,
}

impl ContactRowRange {
    #[cfg(test)]
    pub fn full(rows: usize) -> Self {
        Self {
            start: 0,
            end: rows,
        }
    }

    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}

impl ContactRequest {
    pub fn resolve(self) -> ContactBoundary {
        let samples = (self.row_range.start..self.row_range.end.min(self.rows))
            .map(|row| ContactSample {
                row,
                normal_pressure: self.normal_pressure
                    * pressure_profile_weight(self.incoming_profile, row, self.rows),
                compression: self.compression,
                disruption: self.disruption,
            })
            .collect();

        ContactBoundary {
            front_column: self.front_column,
            samples,
        }
    }
}

pub fn detect_contact_request(
    target: ContactFront,
    incoming: ContactFront,
    incoming_profile: PressureProfile,
    detection: ContactDetection,
) -> Option<ContactRequest> {
    let front_gap = (target.front_position - incoming.front_position).abs();
    let compression = (detection.contact_distance - front_gap).max(0.0);
    if compression <= 0.0 {
        return None;
    }

    let row_range = target.overlap_rows(incoming)?;
    let overlap_ratio = row_range.len() as f32 / target.rows.max(1) as f32;

    Some(ContactRequest {
        front_column: target.front_column,
        rows: target.rows,
        row_range,
        incoming_profile,
        normal_pressure: detection.base_pressure * (compression / detection.contact_distance),
        compression,
        disruption: 1.0 - overlap_ratio,
    })
}

impl ContactBoundary {
    pub fn apply_to_field(&self, field: &mut FormationField, dt: f32) {
        debug_assert!(self.front_column < field.width);

        for sample in &self.samples {
            if sample.row >= field.height {
                continue;
            }
            debug_assert!(sample.compression >= 0.0);
            debug_assert!(sample.disruption >= 0.0);

            let index = field.index(self.front_column, sample.row);
            field.pressure[index] += sample.normal_pressure * dt;
        }
    }
}

impl ContactFront {
    fn overlap_rows(self, incoming: Self) -> Option<ContactRowRange> {
        let (incoming_min, incoming_max) = incoming.lateral_interval();
        let mut start = None;
        let mut end = 0;

        for row in 0..self.rows {
            let row_center = self.row_center(row);
            if row_center >= incoming_min && row_center <= incoming_max {
                start.get_or_insert(row);
                end = row + 1;
            }
        }

        let row_range = ContactRowRange { start: start?, end };
        (!row_range.is_empty()).then_some(row_range)
    }

    fn lateral_interval(self) -> (f32, f32) {
        let half_width = self.rows as f32 * self.row_spacing * 0.5;
        (
            self.lateral_center - half_width,
            self.lateral_center + half_width,
        )
    }

    fn row_center(self, row: usize) -> f32 {
        let center_row = (self.rows as f32 - 1.0) * 0.5;
        self.lateral_center + (row as f32 - center_row) * self.row_spacing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(incoming_profile: PressureProfile) -> ContactRequest {
        ContactRequest {
            front_column: 0,
            rows: 5,
            row_range: ContactRowRange::full(5),
            incoming_profile,
            normal_pressure: 10.0,
            compression: 0.4,
            disruption: 0.2,
        }
    }

    #[test]
    fn wedge_contact_focuses_pressure_toward_center() {
        let boundary = request(PressureProfile::Wedge).resolve();
        let center = boundary
            .samples
            .get(boundary.samples.len() / 2)
            .expect("center sample");
        let edge = boundary.samples.first().expect("edge sample");

        assert!(center.normal_pressure > edge.normal_pressure);
        assert_eq!(center.compression, 0.4);
        assert_eq!(center.disruption, 0.2);
    }

    #[test]
    fn line_contact_has_wider_active_width_than_wedge() {
        let line = request(PressureProfile::Line).resolve();
        let wedge = request(PressureProfile::Wedge).resolve();
        let active_width = |boundary: &ContactBoundary| {
            boundary
                .samples
                .iter()
                .filter(|sample| sample.normal_pressure >= 5.0)
                .count()
        };

        assert!(active_width(&line) > active_width(&wedge));
    }

    #[test]
    fn contact_boundary_applies_pressure_to_target_front_column() {
        let boundary = request(PressureProfile::Line).resolve();
        let mut field = FormationField::new(3, 5);

        boundary.apply_to_field(&mut field, 0.5);

        for row in 0..field.height {
            assert!(field.pressure[field.index(0, row)] > 0.0);
            assert_eq!(field.pressure[field.index(1, row)], 0.0);
            assert_eq!(field.pressure[field.index(2, row)], 0.0);
        }
    }

    #[test]
    fn contact_detection_returns_none_before_fronts_touch() {
        let target = front(0.0, 0.0);
        let incoming = front(2.0, 0.0);

        let contact = detect_contact_request(
            target,
            incoming,
            PressureProfile::Line,
            ContactDetection {
                contact_distance: 1.0,
                base_pressure: 10.0,
            },
        );

        assert!(contact.is_none());
    }

    #[test]
    fn contact_detection_scales_pressure_by_compression() {
        let target = front(0.0, 0.0);
        let incoming = front(0.75, 0.0);

        let contact = detect_contact_request(
            target,
            incoming,
            PressureProfile::Line,
            ContactDetection {
                contact_distance: 1.0,
                base_pressure: 12.0,
            },
        )
        .expect("contact");

        assert_eq!(contact.row_range.start, 0);
        assert_eq!(contact.row_range.end, 5);
        assert!((contact.compression - 0.25).abs() < f32::EPSILON);
        assert!((contact.normal_pressure - 3.0).abs() < f32::EPSILON);
        assert_eq!(contact.disruption, 0.0);
    }

    #[test]
    fn contact_detection_limits_rows_to_lateral_overlap() {
        let target = front(0.0, 0.0);
        let incoming = front(0.5, 2.5);

        let contact = detect_contact_request(
            target,
            incoming,
            PressureProfile::Line,
            ContactDetection {
                contact_distance: 1.0,
                base_pressure: 10.0,
            },
        )
        .expect("contact");

        assert_eq!(contact.row_range.start, 2);
        assert_eq!(contact.row_range.end, 5);
        assert!(contact.disruption > 0.0);
    }

    fn front(front_position: f32, lateral_center: f32) -> ContactFront {
        ContactFront {
            front_column: 0,
            rows: 5,
            row_spacing: 1.0,
            lateral_center,
            front_position,
        }
    }
}
