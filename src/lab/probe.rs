use super::avian::AvianContactCache;
use super::contact::{
    ContactDetection, ContactFront, ContactRequest, detect_contact_request,
};
use super::field::PressureProfile;
use super::model::FormationSide;

/// Источник Layer 2 contact detection в лаборатории.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContactProbeKind {
    /// Геометрия `ContactFront` + overlap (текущая лаборатория).
    #[default]
    Synthetic,
    /// Avian colliders на переднем ряду; при пустом кэше — fallback в Synthetic.
    Avian,
}

impl ContactProbeKind {
    pub const ALL: [Self; 2] = [Self::Synthetic, Self::Avian];

    pub fn label(self) -> &'static str {
        match self {
            Self::Synthetic => "synthetic (geometry)",
            Self::Avian => "avian (colliders)",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ContactProbeInput {
    pub target: ContactFront,
    pub incoming: ContactFront,
    pub incoming_profile: PressureProfile,
    pub detection: ContactDetection,
}

pub trait ContactProbe {
    fn detect(&self, input: ContactProbeInput) -> Option<ContactRequest>;
}

#[derive(Default)]
pub struct SyntheticContactProbe;

impl ContactProbe for SyntheticContactProbe {
    fn detect(&self, input: ContactProbeInput) -> Option<ContactRequest> {
        detect_contact_request(
            input.target,
            input.incoming,
            input.incoming_profile,
            input.detection,
        )
    }
}

pub fn detect_with_probe(
    kind: ContactProbeKind,
    input: ContactProbeInput,
    target_side: FormationSide,
    avian_cache: Option<&AvianContactCache>,
) -> Option<ContactRequest> {
    match kind {
        ContactProbeKind::Synthetic => SyntheticContactProbe.detect(input),
        ContactProbeKind::Avian => avian_cache
            .and_then(|cache| cache.resolve_request(input, target_side))
            .or_else(|| SyntheticContactProbe.detect(input)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn line_fronts() -> (ContactFront, ContactFront) {
        (
            ContactFront {
                front_column: 0,
                rows: 5,
                row_spacing: 1.0,
                lateral_center: 0.0,
                front_position: 0.0,
            },
            ContactFront {
                front_column: 0,
                rows: 5,
                row_spacing: 1.0,
                lateral_center: 0.0,
                front_position: 0.5,
            },
        )
    }

    fn probe_input(target: ContactFront, incoming: ContactFront) -> ContactProbeInput {
        ContactProbeInput {
            target,
            incoming,
            incoming_profile: PressureProfile::Line,
            detection: ContactDetection {
                contact_distance: 1.0,
                base_pressure: 10.0,
            },
        }
    }

    #[test]
    fn synthetic_probe_matches_legacy_detector() {
        let (target, incoming) = line_fronts();
        let input = probe_input(target, incoming);

        let legacy = detect_contact_request(
            input.target,
            input.incoming,
            input.incoming_profile,
            input.detection,
        );
        let probed = detect_with_probe(ContactProbeKind::Synthetic, input, FormationSide::Blue, None);

        assert_eq!(legacy.map(|c| c.row_range), probed.map(|c| c.row_range));
        assert_eq!(
            legacy.map(|c| c.normal_pressure),
            probed.map(|c| c.normal_pressure)
        );
    }

    #[test]
    fn avian_falls_back_to_synthetic_without_cache() {
        let (target, incoming) = line_fronts();
        let input = probe_input(target, incoming);

        let synthetic = detect_with_probe(ContactProbeKind::Synthetic, input, FormationSide::Blue, None);
        let avian = detect_with_probe(ContactProbeKind::Avian, input, FormationSide::Blue, None);

        assert_eq!(synthetic, avian);
    }
}
