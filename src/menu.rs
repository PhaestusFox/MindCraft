use bevy::prelude::*;

use crate::{GameState, world::Map, settings::ViewDistance};
use strum::IntoEnumIterator;
use belly::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_belly)
        .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
        .add_systems(OnExit(GameState::MainMenu), cleanup_menu)
        .add_systems(Update, change_seed.run_if(in_state(GameState::MainMenu)))
        .add_systems(OnEnter(GameState::EscapeMenu), escape_menu)
        .add_systems(Update, update_view_distance)
        .add_systems(OnExit(GameState::EscapeMenu), cleanup_menu)
        .add_systems(Update, toggle_escape_menu)
        .add_systems(OnEnter(GameState::GenWorld), spwan_block_menu);
    }
}

fn setup_belly(mut commands: Commands) {
    commands.add(StyleSheet::load("main.ess"));
}

fn spawn_main_menu(
    mut commands: Commands
) {
    commands.add(eml!{
        <div c:menu>
            <button on:press=run!(|ctx| {
                ctx.add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::GenWorld);
                });
            })><label value="Play"/></button>
            <button on:press=run!(|ctx| {
                ctx.add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::EscapeMenu);
                });
            })><label value="Settings"/></button>
            <textinput id="Seed"/>
        </div>
    });
}

fn cleanup_menu(mut elements: Elements, mut focused: ResMut<belly::core::input::Focused>) {
    elements.select(".menu").remove();
    *focused = belly::core::input::Focused::default();
}

fn change_seed(query: Query<&TextInput, Changed<Element>>, mut elements: Elements, map: Res<Map>) {
    for entity in elements.select("#Seed").entities() {
        if let Ok(input) = query.get(entity) {
            let seed = if input.value.len() == 0 {
                use rand::Rng;
                rand::thread_rng().gen()
            } else {
                match input.value.parse() {
                    Ok(yes) => yes,
                    Err(_) => {
                        use std::hash::{BuildHasher, Hash, Hasher};
                        let mut hasher = bevy::utils::FixedState.build_hasher();
                        input.value.hash(&mut hasher);
                        hasher.finish()
                    }
                }
            };
            map.set_seed(seed);
        }
    }
}

#[derive(Debug, Component, Default)]
struct TempRender(f32);
fn escape_menu(
    mut commands: Commands,
    view_distance: Res<ViewDistance>,
) {
    let slider = commands.spawn(TempRender(view_distance.get())).id();
    commands.add(eml! {
        <div c:menu>
            <span c:view>
            <label bind:value=from!(ViewDistance:0|fmt.c("View Distance: {c}"))/>
            <slider {slider}
            // minimum=5
            // maximum=25
            bind:value=to!(slider, TempRender:0)
            bind:value=from!(slider, TempRender:0)
            />
            </span>
            <button on:press=run!(|ctx| {
                ctx.add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::MainMenu);
                });
            })><label value="Main Menu"/></button>
            <button on:press=run!(|ctx| {
                ctx.add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::Playing);
                });
            })><label value="Close"/></button>
        </div>
    });
}

fn update_view_distance(
    query: Query<&TempRender, Changed<TempRender>>,
    mut view_distance: ResMut<ViewDistance>,
) {
    for dis in &query {
        if view_distance.get() != dis.0 {
            view_distance.set(dis.0);
        }
    }
}

#[derive(Resource, Default)]
struct ToggleEscape(GameState);

fn toggle_escape_menu(
    input: Res<Input<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut old: Local<ToggleEscape>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if *state.get() == GameState::EscapeMenu {
            next.set(old.0);
        } else {
            old.0 = *state.get();
            next.set(GameState::EscapeMenu)
        }
    }
}

fn spwan_block_menu(
    mut commands: Commands,
) {
    commands.add(eml!(
        <buttongroup c:block_bar bind:value=from!(crate::player_controller::SelectedBlock:get()|fmt.c("{c:?}"))>
            <for block in = crate::blocks::BlockType::iter()>
                <button value=format!("{:?}", block) on:press=run!(|ctx| {
                    ctx.commands().add(move |world: &mut World| {
                        world.resource_mut::<crate::player_controller::SelectedBlock>().set(block);
                    });
                })>
                    <img src=block.get_icon_paths()/>
                </button>
            </for>
        </buttongroup>
    ));
}