use super::field::{FormationField, PressureProfile, pressure_profile_weight};
use super::model::FormationSide;
use super::motion::compression_from_gap;
use super::scenario::LabScenario;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactRequest {
    pub front_column: usize,
    pub rows: usize,
    pub row_range: ContactRowRange,
    pub incoming_profile: PressureProfile,
    pub normal_pressure: f32,
    pub compression: f32,
    pub disruption: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactRowRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub struct ContactBoundary {
    pub samples: Vec<ContactSample>,
}

#[derive(Clone, Copy, Debug)]
pub struct ContactSample {
    pub column: usize,
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
                column: self.front_column,
                row,
                normal_pressure: self.normal_pressure
                    * pressure_profile_weight(self.incoming_profile, row, self.rows),
                compression: self.compression,
                disruption: self.disruption,
            })
            .collect();

        ContactBoundary { samples }
    }
}

#[derive(Clone, Copy)]
pub struct BoundaryContactInput {
    pub scenario: LabScenario,
    pub target_side: FormationSide,
    pub target: ContactFront,
    pub incoming: ContactFront,
    pub incoming_profile: PressureProfile,
    pub detection: ContactDetection,
    pub dt: f32,
    /// Layer 3 v0: геометрический thrust героя, усиливает boundary (не пишет в поле напрямую).
    pub gic_impulse: Option<super::gic::GicImpulse>,
}

#[derive(Clone, Copy, Debug)]
pub struct ContactDiagnostics {
    pub overlap_ratio: f32,
    pub disruption: f32,
    pub compression: f32,
    pub row_range: ContactRowRange,
}

pub fn contact_diagnostics(
    target: ContactFront,
    incoming: ContactFront,
    detection: ContactDetection,
) -> Option<ContactDiagnostics> {
    let front_gap = (target.front_position - incoming.front_position).abs();
    let compression = compression_from_gap(front_gap, detection.contact_distance);
    if compression <= 0.0 {
        return None;
    }

    let row_range = target.overlap_rows(incoming)?;
    let overlap_ratio = row_range.len() as f32 / target.rows.max(1) as f32;

    Some(ContactDiagnostics {
        overlap_ratio,
        disruption: 1.0 - overlap_ratio,
        compression,
        row_range,
    })
}

pub fn detect_contact_request(
    target: ContactFront,
    incoming: ContactFront,
    incoming_profile: PressureProfile,
    detection: ContactDetection,
) -> Option<ContactRequest> {
    let diagnostics = contact_diagnostics(target, incoming, detection)?;

    Some(ContactRequest {
        front_column: target.front_column,
        rows: target.rows,
        row_range: diagnostics.row_range,
        incoming_profile,
        normal_pressure: detection.base_pressure
            * (diagnostics.compression / detection.contact_distance),
        compression: diagnostics.compression,
        disruption: diagnostics.disruption,
    })
}

pub fn flank_contact_boundary(
    width: usize,
    height: usize,
    base_pressure: f32,
    compression: f32,
) -> ContactBoundary {
    let flank_row = height.saturating_sub(1);
    let max_column = (width.saturating_sub(1)).max(1) as f32;
    let samples = (0..width)
        .map(|column| {
            let depth = column as f32 / max_column;
            let depth_weight = 0.9 - depth * 0.35;
            ContactSample {
                column,
                row: flank_row,
                normal_pressure: base_pressure * 0.72 * depth_weight,
                compression,
                disruption: 0.0,
            }
        })
        .collect();

    ContactBoundary { samples }
}

pub fn apply_boundary_contacts(
    probe: super::probe::ContactProbeKind,
    input: BoundaryContactInput,
    field: &mut FormationField,
    avian_cache: Option<&super::avian::AvianContactCache>,
) {
    let probe_input = super::probe::ContactProbeInput {
        target: input.target,
        incoming: input.incoming,
        incoming_profile: input.incoming_profile,
        detection: input.detection,
    };

    let contact = super::probe::detect_with_probe(
        probe,
        probe_input,
        input.target_side,
        avian_cache,
    );

    let mut boundary = contact.map(|c| c.resolve());
    if let Some(gic) = input.gic_impulse
        && gic.target_side == input.target_side
        && !gic.is_empty()
    {
        match &mut boundary {
            Some(b) => gic.merge_into_boundary(b),
            None => boundary = Some(gic.to_boundary()),
        }
    }

    if let Some(boundary) = boundary {
        boundary.apply_to_field(field, input.dt);
        if let Some(contact) = contact
            && contact.disruption > 0.05
        {
            apply_exposed_front_disruption(
                field,
                contact.front_column,
                contact.row_range,
                contact.disruption,
                contact.compression,
                input.dt,
            );
        }
    }

    if input.scenario == LabScenario::FlankPressure && input.target_side == FormationSide::Blue {
        flank_contact_boundary(
            field.width,
            field.height,
            input.detection.base_pressure,
            1.0,
        )
        .apply_to_field(field, input.dt);
    }
}

/// Скорость потери организованности от disruption при контакте (1/с при disruption=1).
pub const DISRUPTION_ORGANIZATION_RATE: f32 = 1.35;

impl ContactBoundary {
    pub fn apply_to_field(&self, field: &mut FormationField, dt: f32) {
        for sample in &self.samples {
            if sample.column >= field.width || sample.row >= field.height {
                continue;
            }
            debug_assert!(sample.compression >= 0.0);
            debug_assert!(sample.disruption >= 0.0);

            let index = field.index(sample.column, sample.row);
            field.pressure[index] += sample.normal_pressure * dt;

            if sample.disruption > 0.0 {
                let contact_intensity =
                    sample.disruption * (0.3 + sample.compression.min(1.0) * 0.7);
                let loss = contact_intensity * dt * DISRUPTION_ORGANIZATION_RATE;
                field.organization[index] =
                    (field.organization[index] - loss).max(super::field::MIN_SLOT_ORGANIZATION);
            }
        }
    }
}

fn apply_exposed_front_disruption(
    field: &mut FormationField,
    front_column: usize,
    covered_rows: ContactRowRange,
    disruption: f32,
    compression: f32,
    dt: f32,
) {
    let exposure = disruption * (0.25 + compression.min(1.0) * 0.75) * 0.55;
    if exposure <= 0.0 {
        return;
    }

    let loss = exposure * dt * DISRUPTION_ORGANIZATION_RATE;
    for row in 0..field.height {
        if row >= covered_rows.start && row < covered_rows.end {
            continue;
        }
        let index = field.index(front_column, row);
        field.organization[index] =
            (field.organization[index] - loss).max(super::field::MIN_SLOT_ORGANIZATION);
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
    fn flank_contact_boundary_is_stronger_near_the_outer_edge() {
        let boundary = flank_contact_boundary(4, 3, 10.0, 1.0);
        let flank_row = 2;

        assert!(boundary.samples[0].normal_pressure > boundary.samples[3].normal_pressure);
        assert_eq!(boundary.samples[0].row, flank_row);
        assert_eq!(boundary.samples[3].row, flank_row);
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
    fn disruption_reduces_front_organization() {
        let boundary = ContactBoundary {
            samples: vec![ContactSample {
                column: 0,
                row: 2,
                normal_pressure: 4.0,
                compression: 0.8,
                disruption: 0.5,
            }],
        };
        let mut field = FormationField::new(3, 5);

        boundary.apply_to_field(&mut field, 0.5);

        assert!(field.organization[field.index(0, 2)] < 1.0);
        assert_eq!(field.organization[field.index(0, 1)], 1.0);
    }

    #[test]
    fn exposed_front_rows_lose_organization_on_partial_overlap() {
        let mut field = FormationField::new(3, 5);
        apply_exposed_front_disruption(
            &mut field,
            0,
            ContactRowRange { start: 2, end: 5 },
            0.6,
            0.8,
            0.5,
        );

        assert!(field.organization[field.index(0, 0)] < 1.0);
        assert!(field.organization[field.index(0, 2)] >= 1.0);
    }

    #[test]
    fn gic_impulse_merges_into_existing_boundary() {
        let mut boundary = request(PressureProfile::Line).resolve();
        let before = boundary.samples[2].normal_pressure;
        let gic = crate::lab::gic::GicImpulse {
            target_side: FormationSide::Blue,
            front_column: 0,
            rows: 5,
            row_range: ContactRowRange { start: 2, end: 3 },
            pressure_boost: 4.0,
            compression: 0.5,
            disruption: 0.0,
            from_shapecast: false,
        };
        gic.merge_into_boundary(&mut boundary);
        assert!(boundary.samples[2].normal_pressure > before);
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
