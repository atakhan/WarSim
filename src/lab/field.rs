use bevy::prelude::Component;

#[derive(Clone, Copy)]
pub enum PressureProfile {
    Line,
    Wedge,
}

impl PressureProfile {
    pub fn label(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Wedge => "wedge",
        }
    }
}

#[derive(Clone, Copy)]
pub struct FieldMaterial {
    pub stiffness: f32,
    pub forward_multiplier: f32,
    pub lateral_multiplier: f32,
    pub yield_strength: f32,
    pub viscosity: f32,
    pub morale: f32,
    pub fatigue: f32,
}

impl FieldMaterial {
    pub fn effective_yield(self) -> f32 {
        self.yield_strength * self.morale * (1.0 - self.fatigue * 0.55)
    }
}

#[derive(Component)]
pub struct FormationField {
    pub width: usize,
    pub height: usize,
    pub pressure: Vec<f32>,
    pub velocity: Vec<f32>,
    pub fractured: Vec<bool>,
}

impl FormationField {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        Self {
            width,
            height,
            pressure: vec![0.0; len],
            velocity: vec![0.0; len],
            fractured: vec![false; len],
        }
    }

    pub fn index(&self, column: usize, row: usize) -> usize {
        row * self.width + column
    }

    pub fn reset(&mut self) {
        self.pressure.fill(0.0);
        self.velocity.fill(0.0);
        self.fractured.fill(false);
    }

    pub fn metrics(&self) -> FieldMetrics {
        let slot_count = self.pressure.len().max(1);
        let total_pressure = self
            .pressure
            .iter()
            .map(|pressure| pressure.abs())
            .sum::<f32>();
        let peak_pressure = self
            .pressure
            .iter()
            .map(|pressure| pressure.abs())
            .fold(0.0, f32::max);
        let fractured_slots = self
            .fractured
            .iter()
            .filter(|fractured| **fractured)
            .count();

        FieldMetrics {
            average_pressure: total_pressure / slot_count as f32,
            peak_pressure,
            fractured_slots,
            fracture_ratio: fractured_slots as f32 / slot_count as f32,
        }
    }

    pub fn snapshot(&self) -> FieldSnapshot {
        let center_row = self.height / 2;
        let center = self.row_metrics(center_row);
        let front = self.column_metrics(0);
        let rear = self.column_metrics(self.width - 1);
        let lower_flank = self.row_metrics(0);
        let upper_flank = self.row_metrics(self.height - 1);
        let edge = average_region_metrics(lower_flank, upper_flank);

        FieldSnapshot {
            center_pressure: center.average_pressure,
            edge_pressure: edge.average_pressure,
            center_fracture_ratio: center.fracture_ratio,
            edge_fracture_ratio: edge.fracture_ratio,
            front_fracture_ratio: front.fracture_ratio,
            rear_fracture_ratio: rear.fracture_ratio,
            upper_flank_fracture_ratio: upper_flank.fracture_ratio,
            lower_flank_fracture_ratio: lower_flank.fracture_ratio,
        }
    }

    fn row_metrics(&self, row: usize) -> RegionMetrics {
        let mut total_pressure = 0.0;
        let mut fractured = 0;

        for column in 0..self.width {
            let index = self.index(column, row);
            total_pressure += self.pressure[index].abs();
            fractured += usize::from(self.fractured[index]);
        }

        RegionMetrics::new(total_pressure, fractured, self.width)
    }

    fn column_metrics(&self, column: usize) -> RegionMetrics {
        let mut total_pressure = 0.0;
        let mut fractured = 0;

        for row in 0..self.height {
            let index = self.index(column, row);
            total_pressure += self.pressure[index].abs();
            fractured += usize::from(self.fractured[index]);
        }

        RegionMetrics::new(total_pressure, fractured, self.height)
    }
}

pub struct FieldMetrics {
    pub average_pressure: f32,
    pub peak_pressure: f32,
    pub fractured_slots: usize,
    pub fracture_ratio: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct FieldSnapshot {
    pub center_pressure: f32,
    pub edge_pressure: f32,
    pub center_fracture_ratio: f32,
    pub edge_fracture_ratio: f32,
    pub front_fracture_ratio: f32,
    pub rear_fracture_ratio: f32,
    pub upper_flank_fracture_ratio: f32,
    pub lower_flank_fracture_ratio: f32,
}

impl FieldSnapshot {
    pub fn flank_fracture_asymmetry(self) -> f32 {
        (self.upper_flank_fracture_ratio - self.lower_flank_fracture_ratio).abs()
    }
}

#[derive(Clone, Copy)]
struct RegionMetrics {
    average_pressure: f32,
    fracture_ratio: f32,
}

impl RegionMetrics {
    fn new(total_pressure: f32, fractured: usize, slots: usize) -> Self {
        let slots = slots.max(1);
        Self {
            average_pressure: total_pressure / slots as f32,
            fracture_ratio: fractured as f32 / slots as f32,
        }
    }
}

fn average_region_metrics(first: RegionMetrics, second: RegionMetrics) -> RegionMetrics {
    RegionMetrics {
        average_pressure: (first.average_pressure + second.average_pressure) * 0.5,
        fracture_ratio: (first.fracture_ratio + second.fracture_ratio) * 0.5,
    }
}

pub fn inject_front_pressure(
    field: &mut FormationField,
    front_column: usize,
    incoming_profile: PressureProfile,
    impact_strength: f32,
    pulse_period: f32,
    elapsed: f32,
    dt: f32,
) {
    debug_assert!(front_column < field.width);

    let phase = (elapsed / pulse_period.max(0.1) * std::f32::consts::TAU).sin();
    let pulse = 0.35 + 0.65 * phase.max(0.0);

    for row in 0..field.height {
        let index = field.index(front_column, row);
        field.pressure[index] +=
            impact_strength * profile_weight(incoming_profile, row, field.height) * pulse * dt;
    }
}

pub fn inject_flank_pressure(
    field: &mut FormationField,
    impact_strength: f32,
    pulse: f32,
    dt: f32,
) {
    let flank_row = field.height - 1;
    let max_column = (field.width - 1) as f32;

    for column in 0..field.width {
        let depth = column as f32 / max_column.max(1.0);
        let depth_weight = 0.9 - depth * 0.35;
        let index = field.index(column, flank_row);
        field.pressure[index] += impact_strength * 0.72 * depth_weight * pulse * dt;
    }
}

pub fn propagate_pressure_wave(
    material: FieldMaterial,
    field: &mut FormationField,
    field_leak: f32,
    dt: f32,
) {
    let previous_pressure = field.pressure.clone();
    let previous_velocity = field.velocity.clone();
    let previous_fractured = field.fractured.clone();
    let mut next_pressure = previous_pressure.clone();
    let mut next_velocity = previous_velocity.clone();
    let mut next_fractured = previous_fractured.clone();

    for row in 0..field.height {
        for column in 0..field.width {
            let index = field.index(column, row);
            let mut laplacian = 0.0;
            let mut sum_weights = 0.0;
            let mut fractured_neighbours = 0.0_f32;

            for_each_neighbour(
                column,
                row,
                field.width,
                field.height,
                |neighbour_column, neighbour_row| {
                    let neighbour_index = field.index(neighbour_column, neighbour_row);
                    let weight = if neighbour_column != column {
                        material.forward_multiplier
                    } else {
                        material.lateral_multiplier
                    };

                    laplacian += previous_pressure[neighbour_index] * weight;
                    sum_weights += weight;
                    if previous_fractured[neighbour_index] {
                        fractured_neighbours += 1.0;
                    }
                },
            );

            laplacian -= previous_pressure[index] * sum_weights;

            let damping = (1.0 - (material.viscosity + field_leak) * dt).clamp(0.0, 1.0);
            let mut velocity =
                (previous_velocity[index] + material.stiffness * laplacian * dt) * damping;
            let mut pressure = (previous_pressure[index] + velocity * dt)
                * (1.0 - field_leak * dt).clamp(0.0, 1.0);

            let yield_limit = material.effective_yield();
            let contagion_limit =
                yield_limit * (0.92_f32 - fractured_neighbours * 0.12).clamp(0.48, 0.92);

            if pressure.abs() > yield_limit
                || (fractured_neighbours > 0.0 && pressure.abs() > contagion_limit)
            {
                next_fractured[index] = true;
            }

            if previous_fractured[index] || next_fractured[index] {
                pressure *= 0.72;
                velocity *= 0.45;
            }

            next_pressure[index] = pressure;
            next_velocity[index] = velocity;
        }
    }

    field.pressure = next_pressure;
    field.velocity = next_velocity;
    field.fractured = next_fractured;
}

pub fn update_material_from_fractures(
    material: &mut FieldMaterial,
    field: &FormationField,
    dt: f32,
) {
    let fractured = field
        .fractured
        .iter()
        .filter(|fractured| **fractured)
        .count() as f32;
    let fracture_ratio = fractured / field.fractured.len() as f32;

    material.morale = (material.morale - fracture_ratio * 0.05 * dt).clamp(0.15, 1.0);
    material.fatigue = (material.fatigue + fracture_ratio * 0.025 * dt).clamp(0.0, 1.0);
}

fn profile_weight(profile: PressureProfile, row: usize, height: usize) -> f32 {
    let center_row = (height as f32 - 1.0) * 0.5;
    let max_distance = center_row.max(1.0);
    let distance_from_center = (row as f32 - center_row).abs();
    let center_weight = 1.0 - distance_from_center / max_distance;

    match profile {
        PressureProfile::Line => 0.65 + center_weight * 0.25,
        PressureProfile::Wedge => 0.25 + center_weight.powf(2.0) * 1.1,
    }
}

fn for_each_neighbour(
    column: usize,
    row: usize,
    width: usize,
    height: usize,
    mut visit: impl FnMut(usize, usize),
) {
    if column > 0 {
        visit(column - 1, row);
    }
    if column + 1 < width {
        visit(column + 1, row);
    }
    if row > 0 {
        visit(column, row - 1);
    }
    if row + 1 < height {
        visit(column, row + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stable_material() -> FieldMaterial {
        FieldMaterial {
            stiffness: 0.0,
            forward_multiplier: 1.0,
            lateral_multiplier: 1.0,
            yield_strength: 5.0,
            viscosity: 0.0,
            morale: 1.0,
            fatigue: 0.0,
        }
    }

    #[test]
    fn wedge_profile_concentrates_more_pressure_in_center_than_line() {
        let center_row = 2;

        assert!(
            profile_weight(PressureProfile::Wedge, center_row, 5)
                > profile_weight(PressureProfile::Line, center_row, 5)
        );
        assert!(
            profile_weight(PressureProfile::Wedge, 0, 5)
                < profile_weight(PressureProfile::Line, 0, 5)
        );
    }

    #[test]
    fn front_pressure_is_injected_only_on_contact_column() {
        let mut field = FormationField::new(3, 3);

        inject_front_pressure(&mut field, 2, PressureProfile::Line, 10.0, 1.0, 0.0, 1.0);

        for row in 0..field.height {
            assert!(field.pressure[field.index(2, row)] > 0.0);
            assert_eq!(field.pressure[field.index(0, row)], 0.0);
            assert_eq!(field.pressure[field.index(1, row)], 0.0);
        }
    }

    #[test]
    fn flank_pressure_is_stronger_near_the_outer_edge() {
        let mut field = FormationField::new(4, 3);

        inject_flank_pressure(&mut field, 10.0, 1.0, 1.0);

        let flank_row = field.height - 1;
        assert!(
            field.pressure[field.index(0, flank_row)] > field.pressure[field.index(3, flank_row)]
        );
        assert_eq!(field.pressure[field.index(0, 0)], 0.0);
    }

    #[test]
    fn pressure_above_yield_marks_slot_as_fractured() {
        let mut field = FormationField::new(3, 3);
        let center = field.index(1, 1);
        field.pressure[center] = 6.0;

        propagate_pressure_wave(stable_material(), &mut field, 0.0, 1.0);

        assert!(field.fractured[center]);
    }

    #[test]
    fn fractured_neighbour_lowers_local_fracture_threshold() {
        let mut field = FormationField::new(3, 3);
        let center = field.index(1, 1);
        let left = field.index(0, 1);
        field.pressure[center] = 4.2;
        field.fractured[left] = true;

        propagate_pressure_wave(stable_material(), &mut field, 0.0, 1.0);

        assert!(field.fractured[center]);
    }

    #[test]
    fn fractured_slots_damp_pressure_and_velocity() {
        let mut field = FormationField::new(1, 1);
        field.pressure[0] = 10.0;
        field.velocity[0] = 2.0;
        field.fractured[0] = true;

        propagate_pressure_wave(stable_material(), &mut field, 0.0, 1.0);

        assert!(field.pressure[0] < 10.0);
        assert!(field.velocity[0] < 2.0);
    }

    #[test]
    fn fractures_reduce_morale_and_increase_fatigue() {
        let mut field = FormationField::new(2, 2);
        field.fractured[0] = true;
        field.fractured[1] = true;
        let mut material = stable_material();

        update_material_from_fractures(&mut material, &field, 1.0);

        assert!(material.morale < 1.0);
        assert!(material.fatigue > 0.0);
    }

    #[test]
    fn empty_field_stays_stable_without_injected_pressure() {
        let mut field = FormationField::new(3, 3);
        let material = FieldMaterial {
            stiffness: 4.0,
            ..stable_material()
        };

        propagate_pressure_wave(material, &mut field, 0.0, 1.0);

        assert!(field.pressure.iter().all(|pressure| *pressure == 0.0));
        assert!(field.velocity.iter().all(|velocity| *velocity == 0.0));
        assert!(field.fractured.iter().all(|fractured| !fractured));
    }

    #[test]
    fn anisotropy_spreads_pressure_more_along_forward_axis() {
        let mut field = FormationField::new(3, 3);
        let center = field.index(1, 1);
        field.pressure[center] = 1.0;
        let material = FieldMaterial {
            stiffness: 1.0,
            forward_multiplier: 2.0,
            lateral_multiplier: 0.5,
            yield_strength: 100.0,
            ..stable_material()
        };

        propagate_pressure_wave(material, &mut field, 0.0, 0.1);

        let right = field.pressure[field.index(2, 1)];
        let top = field.pressure[field.index(1, 0)];
        assert!(right > top);
    }
}
