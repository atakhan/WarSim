use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::scenario::{FormationScenarioQuery, LabScenario, apply_scenario};
use super::settings::LabSettings;

pub fn lab_ui(
    mut contexts: EguiContexts,
    mut settings: ResMut<LabSettings>,
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
                ui.label("Когда локальное давление выше прочности материала, слот ломается: он желтеет, проседает и смещается из строя.");
                ui.label("Сломанные слоты повышают усталость и могут заражать соседей потерей структуры.");
                ui.separator();
                ui.label("Цель этой версии: увидеть волны давления и первые fracture-состояния, ещё без Avian contact zone и индивидуальных ударов.");
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
                ui.label("Смещение назад показывает сжатие формации; боковой разъезд показывает потерю структуры.");
                ui.label("В сценарии Flank pressure верхний ряд синей линии получает дополнительную depth-волну через тот же contact pipeline.");
            });
        settings.show_legend = show_legend;
    }

    egui::Window::new("FMP Laboratory").show(ctx, |ui| {
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
            apply_scenario(settings.scenario, &mut formations);
        }

        ui.label(settings.scenario.description());
        ui.label(format!("Watch for: {}", settings.scenario.watch_for()));

        ui.add(
            egui::Slider::new(&mut settings.impact_strength, 0.0..=18.0)
                .text("boundary pressure"),
        );
        ui.add(
            egui::Slider::new(&mut settings.contact_distance, 0.5..=12.0)
                .text("contact distance"),
        );
        ui.add(egui::Slider::new(&mut settings.field_leak, 0.0..=0.3).text("field leak"));
        ui.add(egui::Slider::new(&mut settings.pulse_period, 0.4..=4.0).text("pulse period"));
        ui.checkbox(&mut settings.show_fractures, "show fractures");

        ui.separator();
        ui.label("Layer 1: pressure waves inside formation material");
        ui.label("Watch the bright wave enter from the contact edge and break weak slots into yellow fracture markers.");
        ui.label("Use Reset scenario after changing parameters if you want a clean comparison.");

        for (name, formation, mut material, field) in &mut formations {
            ui.collapsing(name.as_str(), |ui| {
                let metrics = field.metrics();
                let snapshot = field.snapshot();

                ui.label(format!("profile: {}", formation.profile.label()));
                ui.add(egui::Slider::new(&mut material.stiffness, 1.0..=12.0).text("stiffness"));
                ui.add(egui::Slider::new(&mut material.forward_multiplier, 0.1..=3.0).text("fwd multiplier"));
                ui.add(egui::Slider::new(&mut material.lateral_multiplier, 0.1..=3.0).text("lat multiplier"));
                ui.add(
                    egui::Slider::new(&mut material.yield_strength, 1.0..=10.0)
                        .text("yield strength"),
                );
                ui.add(egui::Slider::new(&mut material.viscosity, 0.0..=3.0).text("viscosity"));
                ui.add(egui::Slider::new(&mut material.morale, 0.1..=1.0).text("morale"));
                ui.add(egui::Slider::new(&mut material.fatigue, 0.0..=1.0).text("fatigue"));

                ui.separator();
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
                        .desired_width(180.0),
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
            });
        }
    });
}
