use bevy::prelude::*;
use bevy_sprinkles::prelude::*;

use crate::state::{DirtyState, EditorState};
use crate::ui::widgets::checkbox::{CheckboxCommitEvent, CheckboxProps, CheckboxState, checkbox};
use crate::ui::widgets::inspector_field::fields_row;
use crate::ui::widgets::text_edit::{TextEditProps, text_edit};

use super::draw_pass::MaterialSection;
use super::{InspectorSection, section_needs_setup};
use crate::ui::components::binding::{
    FieldBinding, FieldKind, get_inspecting_emitter, get_inspecting_emitter_mut,
    mark_dirty_and_restart,
};

#[derive(Component)]
struct FlipbookEnabledCheckbox;

#[derive(Component)]
struct FlipbookOptions;

pub fn plugin(app: &mut App) {
    app.add_observer(handle_flipbook_toggle).add_systems(
        Update,
        (
            setup_flipbook_options,
            toggle_flipbook_options,
            sync_flipbook_checkbox,
        )
            .after(super::update_inspected_emitter_tracker),
    );
}

fn setup_flipbook_options(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    sections: Query<(Entity, &InspectorSection), With<MaterialSection>>,
    existing: Query<Entity, With<FlipbookOptions>>,
) {
    let Some(entity) = section_needs_setup(&sections, &existing) else {
        return;
    };

    let flipbook = get_inspecting_emitter(&editor_state, &assets)
        .and_then(|(_, e)| e.draw_pass.flipbook.clone());
    let enabled = flipbook.is_some();
    let fb = flipbook.as_ref().cloned().unwrap_or_default();

    commands.entity(entity).with_children(|parent| {
        parent.spawn(fields_row()).with_child((
            FlipbookEnabledCheckbox,
            checkbox(
                CheckboxProps::new("Flipbook").checked(enabled),
                &asset_server,
            ),
        ));
    });

    let display = if enabled {
        Display::Flex
    } else {
        Display::None
    };

    let options = commands
        .spawn((
            FlipbookOptions,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                display,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(fields_row()).with_children(|row| {
                row.spawn((
                    FieldBinding::emitter_variant_field(
                        "draw_pass.flipbook",
                        "columns",
                        FieldKind::U32,
                    ),
                    text_edit(
                        TextEditProps::default()
                            .with_label("Columns")
                            .with_default_value(fb.columns.to_string())
                            .numeric_i32()
                            .with_min(1.0),
                    ),
                ));
                row.spawn((
                    FieldBinding::emitter_variant_field(
                        "draw_pass.flipbook",
                        "rows",
                        FieldKind::U32,
                    ),
                    text_edit(
                        TextEditProps::default()
                            .with_label("Rows")
                            .with_default_value(fb.rows.to_string())
                            .numeric_i32()
                            .with_min(1.0),
                    ),
                ));
            });

            parent.spawn(fields_row()).with_children(|row| {
                row.spawn((
                    FieldBinding::emitter_variant_field(
                        "draw_pass.flipbook",
                        "speed",
                        FieldKind::F32,
                    ),
                    text_edit(
                        TextEditProps::default()
                            .with_label("Speed")
                            .with_default_value(fb.speed.to_string())
                            .numeric_f32()
                            .with_min(0.0),
                    ),
                ));
            });
        })
        .id();

    let wrapper = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();
    commands.entity(wrapper).add_child(options);
    commands.entity(entity).add_child(wrapper);
}

fn toggle_flipbook_options(
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut options: Query<&mut Node, With<FlipbookOptions>>,
) {
    let Ok(mut node) = options.single_mut() else {
        return;
    };

    let enabled = get_inspecting_emitter(&editor_state, &assets)
        .map(|(_, e)| e.draw_pass.flipbook.is_some())
        .unwrap_or(false);

    super::set_display_visible(&mut node, enabled);
}

fn sync_flipbook_checkbox(
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut checkboxes: Query<&mut CheckboxState, With<FlipbookEnabledCheckbox>>,
    new_checkboxes: Query<Entity, Added<FlipbookEnabledCheckbox>>,
) {
    if !editor_state.is_changed() && !assets.is_changed() && new_checkboxes.is_empty() {
        return;
    }

    let enabled = get_inspecting_emitter(&editor_state, &assets)
        .map(|(_, e)| e.draw_pass.flipbook.is_some())
        .unwrap_or(false);

    for mut state in &mut checkboxes {
        if state.checked != enabled {
            state.checked = enabled;
        }
    }
}

fn handle_flipbook_toggle(
    trigger: On<CheckboxCommitEvent>,
    checkboxes: Query<(), With<FlipbookEnabledCheckbox>>,
    editor_state: Res<EditorState>,
    mut assets: ResMut<Assets<ParticlesAsset>>,
    mut dirty_state: ResMut<DirtyState>,
    mut emitter_runtimes: Query<&mut EmitterRuntime>,
) {
    if checkboxes.get(trigger.entity).is_err() {
        return;
    }

    let Some((_, emitter)) = get_inspecting_emitter_mut(&editor_state, &mut assets) else {
        return;
    };

    let currently_enabled = emitter.draw_pass.flipbook.is_some();
    if trigger.checked == currently_enabled {
        return;
    }

    emitter.draw_pass.flipbook = if trigger.checked {
        Some(Flipbook::default())
    } else {
        None
    };

    mark_dirty_and_restart(
        &mut dirty_state,
        &mut emitter_runtimes,
        emitter.time.fixed_seed,
    );
}
