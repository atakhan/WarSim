use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::contact::{ContactDetection, ContactFront, contact_diagnostics};
use super::field::{FieldSnapshot, FormationField};
use super::model::{Formation, FormationMaterial, FormationSide};
use super::motion::{FrontPositions, compression_from_gap, front_gap};
use super::scenario::{FormationScenarioQuery, LabScenario, apply_scenario};
use super::avian::AvianContactCache;
use super::probe::ContactProbeKind;
use super::gic::GicImpulse;
use super::simulation::GicLabState;
use super::settings::{FormationMotion, LabSettings};

const RED_PANEL_POS: [f32; 2] = [16.0, 72.0];
const BLUE_PANEL_POS: [f32; 2] = [520.0, 72.0];
const FORMATION_PANEL_WIDTH: f32 = 268.0;

pub fn lab_ui(
    mut contexts: EguiContexts,
    mut settings: ResMut<LabSettings>,
    avian_cache: Res<AvianContactCache>,
    gic_state: Res<GicLabState>,
    mut formations: FormationScenarioQuery,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    if settings.show_help {
        let mut show_help = settings.show_help;
        egui::Window::new("Что происходит на сцене")
            .open(&mut show_help)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Это не бой с атаками и уроном, а первая лаборатория FMP Layer 1.");
                ui.label("Красный и синий строй представлены как материалы формации.");
                ui.label("Пульс давления создаётся через Contact Zone: профиль противника задаёт форму фронтального контакта, фланг — отдельный boundary по верхнему ряду.");
                ui.label("Formation approach сближает строи (оба, только красный или только синий): compression растёт от зазора между фронтами.");
                ui.label("Disruption при неполном фронтальном перекрытии снижает организованность слотов контакта и облегчает локальный fracture.");
                ui.label("Когда локальное давление выше прочности материала, слот ломается: он желтеет, проседает и смещается из строя.");
                ui.label("Сломанные слоты повышают усталость и могут заражать соседей потерей структуры.");
                ui.separator();
                ui.label("Статистика строев — в отдельных окнах Red / Blue formation.");
                ui.separator();
                ui.label("Если «всё одинаково»: возьми Offset contact + red advances only.");
                ui.label("Серые/разъехавшиеся кубики на фронте = падение organization от disruption.");
            });
        settings.show_help = show_help;
    }

    if settings.show_legend {
        let mut show_legend = settings.show_legend;
        egui::Window::new("Легенда")
            .open(&mut show_legend)
            .resizable(false)
            .show(ctx, |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(170, 40, 35),
                    "Красный: атакующая формация.",
                );
                ui.colored_label(
                    egui::Color32::from_rgb(35, 75, 190),
                    "Синий: обороняющаяся формация.",
                );
                ui.colored_label(
                    egui::Color32::WHITE,
                    "Чем ярче куб, тем выше локальное давление.",
                );
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 45),
                    "Жёлтый: fracture, слот строя потерял устойчивость.",
                );
                ui.colored_label(
                    egui::Color32::from_rgb(120, 125, 135),
                    "Серый/разъезд на фронте: disruption снизил organization.",
                );
                ui.label("Смещение назад показывает сжатие формации; боковой разъезд показывает потерю структуры.");
                ui.label("В сценарии Flank pressure верхний ряд синей линии получает дополнительную depth-волну через тот же contact pipeline.");
            });
        settings.show_legend = show_legend;
    }

    let (red_front, blue_front) = collect_fronts(&formations);

    egui::Window::new("FMP Laboratory")
        .default_width(320.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if settings.paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    settings.paused = !settings.paused;
                }

                if ui.button("Reset scenario").clicked() {
                    apply_scenario(settings.scenario, &mut formations);
                }

                if ui.button("Help").clicked() {
                    settings.show_help = true;
                    settings.show_legend = true;
                }
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut settings.show_red_stats, "Red panel");
                ui.checkbox(&mut settings.show_blue_stats, "Blue panel");
            });

            let mut scenario_changed = false;
            egui::ComboBox::from_label("scenario")
                .selected_text(settings.scenario.label())
                .show_ui(ui, |ui| {
                    for scenario in LabScenario::ALL {
                        scenario_changed |= ui
                            .selectable_value(&mut settings.scenario, scenario, scenario.label())
                            .changed();
                    }
                });

            if scenario_changed {
                if settings.scenario.locks_tuning() {
                    settings.apply_guided_preset();
                } else {
                    settings.lock_tuning = false;
                }
                apply_scenario(settings.scenario, &mut formations);
            }

            ui.label(settings.scenario.description());
            ui.label(format!("Watch for: {}", settings.scenario.watch_for()));

            if settings.lock_tuning {
                ui.separator();
                ui.heading("Смотри на сцене");
                for item in settings.scenario.guided_checklist() {
                    ui.label(format!("• {item}"));
                }
                ui.separator();
                ui.label("Режим без слайдеров. Другой сценарий в списке — вернёт ручную подстройку.");
            }

            if !settings.lock_tuning {
                ui.checkbox(&mut settings.gic_enabled, "hero GIC thrust (auto)");
                if settings.gic_enabled {
                    ui.add(
                        egui::Slider::new(&mut settings.gic_auto_period, 0.5..=6.0)
                            .text("thrust period (s)"),
                    );
                }
            } else if settings.gic_enabled {
                ui.label("Hero GIC thrust: включён (каждые 2.5 с)");
            }

            if !settings.lock_tuning {
                ui.add(
                    egui::Slider::new(&mut settings.impact_strength, 0.0..=18.0)
                        .text("boundary pressure"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.contact_distance, 0.5..=12.0)
                        .text("contact distance"),
                );

                ui.label("formation approach");
                egui::ComboBox::from_id_salt("formation_motion")
                    .selected_text(formation_motion_label(settings.formation_motion))
                    .show_ui(ui, |ui| {
                        for motion in [
                            FormationMotion::Static,
                            FormationMotion::ApproachBoth,
                            FormationMotion::ApproachRed,
                            FormationMotion::ApproachBlue,
                        ] {
                            ui.selectable_value(
                                &mut settings.formation_motion,
                                motion,
                                formation_motion_label(motion),
                            );
                        }
                    });
                if settings.formation_motion.is_approaching() {
                    ui.add(
                        egui::Slider::new(&mut settings.approach_speed, 0.0..=2.0)
                            .text("approach speed"),
                    );
                }

                ui.add(egui::Slider::new(&mut settings.field_leak, 0.0..=0.3).text("field leak"));
                ui.add(
                    egui::Slider::new(&mut settings.pulse_period, 0.4..=4.0).text("pulse period"),
                );
                ui.checkbox(&mut settings.show_fractures, "show fractures");

                ui.label("contact probe");
                egui::ComboBox::from_id_salt("contact_probe")
                    .selected_text(settings.contact_probe.label())
                    .show_ui(ui, |ui| {
                        for kind in ContactProbeKind::ALL {
                            ui.selectable_value(&mut settings.contact_probe, kind, kind.label());
                        }
                    });
            }

            ui.separator();
            ui.heading("Contact zone");
            contact_zone_summary(ui, &settings, &avian_cache, red_front, blue_front);
            gic_status_summary(ui, gic_state.last_impulse);

            ui.separator();
            ui.label(format!(
                "Рекомендуемое движение: {}",
                formation_motion_label(suggested_motion_for_scenario(settings.scenario))
            ));
            ui.label("Статистика строев — окна Red / Blue formation по бокам.");
        });

    if settings.show_red_stats {
        let mut open = settings.show_red_stats;
        egui::Window::new("Red formation")
            .open(&mut open)
            .default_pos(RED_PANEL_POS)
            .default_width(FORMATION_PANEL_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                for (name, formation, mut material, field) in formations.iter_mut() {
                    if formation.side != FormationSide::Red {
                        continue;
                    }
                    formation_stats_panel(
                        ui,
                        name.as_str(),
                        &formation,
                        &mut material,
                        &field,
                        egui::Color32::from_rgb(200, 70, 65),
                        None,
                    );
                    break;
                }
            });
        settings.show_red_stats = open;
    }

    if settings.show_blue_stats {
        let mut open = settings.show_blue_stats;
        let defender_contact = match (red_front, blue_front) {
            (Some(red), Some(blue)) => contact_diagnostics(
                blue,
                red,
                ContactDetection {
                    contact_distance: settings.contact_distance,
                    base_pressure: settings.impact_strength,
                },
            ),
            _ => None,
        };

        egui::Window::new("Blue formation")
            .open(&mut open)
            .default_pos(BLUE_PANEL_POS)
            .default_width(FORMATION_PANEL_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                for (name, formation, mut material, field) in formations.iter_mut() {
                    if formation.side != FormationSide::Blue {
                        continue;
                    }
                    formation_stats_panel(
                        ui,
                        name.as_str(),
                        &formation,
                        &mut material,
                        &field,
                        egui::Color32::from_rgb(70, 120, 220),
                        defender_contact,
                    );
                    break;
                }
            });
        settings.show_blue_stats = open;
    }
}

fn collect_fronts(
    formations: &FormationScenarioQuery,
) -> (Option<ContactFront>, Option<ContactFront>) {
    let mut red = None;
    let mut blue = None;

    for (_, formation, _, _) in formations {
        let front = formation.contact_front();
        match formation.side {
            FormationSide::Red => red = Some(front),
            FormationSide::Blue => blue = Some(front),
        }
    }

    (red, blue)
}

fn contact_zone_summary(
    ui: &mut egui::Ui,
    settings: &LabSettings,
    avian_cache: &AvianContactCache,
    red_front: Option<ContactFront>,
    blue_front: Option<ContactFront>,
) {
    let Some((red_front, blue_front)) = red_front.zip(blue_front) else {
        ui.label("Нет данных о фронтах.");
        return;
    };

    let gap = front_gap(FrontPositions {
        red: red_front.front_position,
        blue: blue_front.front_position,
    });
    let compression = compression_from_gap(gap, settings.contact_distance);
    ui.label(format!("front gap: {gap:.2}"));
    ui.label(format!("compression: {compression:.2}"));

    if settings.contact_probe == ContactProbeKind::Avian {
        if let Some(hit) = avian_cache.blue_defense {
            ui.label(format!(
                "avian overlap rows: {}..{}",
                hit.row_range.start, hit.row_range.end
            ));
            ui.label(format!("avian disruption: {:.2}", hit.disruption));
            ui.label(format!("avian penetration: {:.3}", hit.penetration_depth));
            ui.label(format!(
                "compression gap / pen: {:.2} / {:.2} → {:.2}",
                hit.gap_compression, hit.penetration_compression, hit.compression
            ));
            ui.label(format!("impact scale: {:.2}", hit.impact_scale));
        } else {
            ui.label("avian: no collider contact this frame");
        }
    } else if let Some(contact) = contact_diagnostics(
        blue_front,
        red_front,
        ContactDetection {
            contact_distance: settings.contact_distance,
            base_pressure: settings.impact_strength,
        },
    ) {
        ui.label(format!(
            "defender overlap: {:.0}%",
            contact.overlap_ratio * 100.0
        ));
        ui.label(format!("defender disruption: {:.2}", contact.disruption));
    } else {
        ui.label("defender disruption: —");
    }
}

fn gic_status_summary(ui: &mut egui::Ui, last: Option<GicImpulse>) {
    ui.separator();
    ui.label("Hero GIC (Layer 3 v0)");
    if let Some(impulse) = last {
        ui.label(format!(
            "last thrust ({}) rows: {}..{}, boost {:.2}",
            if impulse.from_shapecast {
                "avian shapecast"
            } else {
                "geometry"
            },
            impulse.row_range.start,
            impulse.row_range.end,
            impulse.pressure_boost
        ));
    } else {
        ui.label("last thrust: —");
    }
}

fn formation_stats_panel(
    ui: &mut egui::Ui,
    title: &str,
    formation: &Formation,
    material: &mut FormationMaterial,
    field: &FormationField,
    accent: egui::Color32,
    defender_contact: Option<super::contact::ContactDiagnostics>,
) {
    ui.colored_label(accent, title);
    ui.label(format!("profile: {}", formation.profile.label()));

    if let Some(contact) = defender_contact {
        ui.separator();
        ui.label("Contact (defender)");
        ui.label(format!(
            "overlap: {:.0}% (rows {}..{})",
            contact.overlap_ratio * 100.0,
            contact.row_range.start,
            contact.row_range.end
        ));
        ui.label(format!("disruption: {:.2}", contact.disruption));
        ui.add(
            egui::ProgressBar::new(contact.disruption)
                .text("disruption")
                .desired_width(FORMATION_PANEL_WIDTH - 24.0),
        );
    }

    ui.separator();
    ui.label("Material");
    ui.add(egui::Slider::new(&mut material.stiffness, 1.0..=12.0).text("stiffness"));
    ui.add(
        egui::Slider::new(&mut material.forward_multiplier, 0.1..=3.0).text("fwd multiplier"),
    );
    ui.add(
        egui::Slider::new(&mut material.lateral_multiplier, 0.1..=3.0).text("lat multiplier"),
    );
    ui.add(
        egui::Slider::new(&mut material.yield_strength, 1.0..=10.0).text("yield strength"),
    );
    ui.add(egui::Slider::new(&mut material.viscosity, 0.0..=3.0).text("viscosity"));
    ui.add(egui::Slider::new(&mut material.morale, 0.1..=1.0).text("morale"));
    ui.add(egui::Slider::new(&mut material.fatigue, 0.0..=1.0).text("fatigue"));

    let snapshot = field.snapshot(formation.front_column());
    formation_field_stats(ui, material, field, &snapshot);
}

fn formation_field_stats(
    ui: &mut egui::Ui,
    material: &FormationMaterial,
    field: &FormationField,
    snapshot: &FieldSnapshot,
) {
    let metrics = field.metrics();

    ui.separator();
    ui.label("Layer 2 → field");
    ui.label(format!(
        "front organization min: {:.0}%",
        snapshot.front_organization_min * 100.0
    ));
    ui.add(
        egui::ProgressBar::new(snapshot.front_organization_min)
            .text("front organization")
            .desired_width(FORMATION_PANEL_WIDTH - 24.0),
    );

    ui.separator();
    ui.label("Layer 1 field");
    ui.label(format!("effective yield: {:.2}", material.effective_yield()));
    ui.label(format!("avg pressure: {:.2}", metrics.average_pressure));
    ui.label(format!("peak pressure: {:.2}", metrics.peak_pressure));
    ui.label(format!(
        "center/edge pressure: {:.2}/{:.2}",
        snapshot.center_pressure, snapshot.edge_pressure
    ));
    ui.label(format!(
        "fractured slots: {}/{} ({:.0}%)",
        metrics.fractured_slots,
        field.fractured.len(),
        metrics.fracture_ratio * 100.0
    ));
    ui.add(
        egui::ProgressBar::new(metrics.fracture_ratio)
            .text("formation fracture")
            .desired_width(FORMATION_PANEL_WIDTH - 24.0),
    );
    ui.label(format!(
        "center/edge fracture: {:.0}%/{:.0}%",
        snapshot.center_fracture_ratio * 100.0,
        snapshot.edge_fracture_ratio * 100.0
    ));
    ui.label(format!(
        "front/rear fracture: {:.0}%/{:.0}%",
        snapshot.front_fracture_ratio * 100.0,
        snapshot.rear_fracture_ratio * 100.0
    ));
    ui.label(format!(
        "flank pressure upper/lower: {:.2}/{:.2} (Δ {:.2})",
        snapshot.upper_flank_pressure,
        snapshot.lower_flank_pressure,
        snapshot.flank_pressure_asymmetry()
    ));
    ui.label(format!(
        "flank fracture upper/lower: {:.0}%/{:.0}% (Δ {:.0}%)",
        snapshot.upper_flank_fracture_ratio * 100.0,
        snapshot.lower_flank_fracture_ratio * 100.0,
        snapshot.flank_fracture_asymmetry() * 100.0
    ));
}

fn suggested_motion_for_scenario(scenario: LabScenario) -> FormationMotion {
    match scenario {
        LabScenario::GuidedDemo
        | LabScenario::OffsetContact
        | LabScenario::WedgeVsLine
        | LabScenario::WedgeVsPhalanx
        | LabScenario::PhalanxVsCrowd => FormationMotion::ApproachRed,
        _ => FormationMotion::Static,
    }
}

fn formation_motion_label(motion: FormationMotion) -> &'static str {
    match motion {
        FormationMotion::Static => "static",
        FormationMotion::ApproachBoth => "both advance",
        FormationMotion::ApproachRed => "red advances only",
        FormationMotion::ApproachBlue => "blue advances only",
    }
}
