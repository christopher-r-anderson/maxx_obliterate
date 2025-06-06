use std::time::Duration;

use bevy::prelude::*;

use crate::{
    app_state::AppState,
    gameplay::{
        enemy::{Enemy, EnemyBundle, EnemyDestroyedEvent, EnemyDestruction},
        explosion::Explosion,
        game_run::{GameRun, LevelStatus},
        player::{PlayerDestroyedEvent, player_bundle},
        stage::{spawn_level_info_panel, spawn_stage},
    },
    menus::level_complete::spawn_level_complete_menu,
};

const LEAD_OUT_TIME_SUCCESS: Duration = Duration::from_secs(1);
const LEAD_OUT_TIME_FAIL: Duration = Duration::from_secs(3);

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<LevelState>()
            .register_type::<Level>()
            .init_resource::<LeadOutTimer>()
            .add_observer(on_enemy_destroyed)
            .add_observer(on_player_destroyed)
            .add_systems(
                FixedUpdate,
                check_level_complete.run_if(in_state(LevelState::Playing)),
            )
            .add_systems(OnEnter(LevelState::Loading), spawn_level)
            .add_systems(OnEnter(LevelState::Complete), on_level_complete)
            .add_systems(
                Update,
                update_lead_out_timer.run_if(in_state(LevelState::Complete)),
            )
            .add_systems(
                FixedUpdate,
                check_load_status.run_if(in_state(LevelState::Loading)),
            );
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Level;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct LevelStats {
    pub enemies_destroyed: u32,
    pub success: Option<bool>,
    pub total_enemies: u32,
}

impl LevelStats {
    fn new(total_enemies: u32) -> Self {
        Self {
            enemies_destroyed: 0,
            success: None,
            total_enemies,
        }
    }
}

pub struct LevelConfig {
    pub enemies: Vec<EnemyBundle>,
    pub name: &'static str,
    pub notes: &'static str,
    pub start_position: Vec2,
}

#[derive(SubStates, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(AppState = AppState::Gameplay)]
#[states(scoped_entities)]
pub enum LevelState {
    #[default]
    Loading,
    Ready,
    Playing,
    Complete,
}

fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>, game_run: Single<&GameRun>) {
    let level_config = game_run.current_level_config(&asset_server);
    commands.spawn((
        StateScoped(AppState::Gameplay),
        LevelStats::new(
            level_config
                .enemies
                .iter()
                .filter(|enemy| enemy.requires_destruction())
                .collect::<Vec<_>>()
                .len()
                .try_into()
                .expect("There shouldn't be more enemies than u32"),
        ),
    ));
    spawn_level_info_panel(&mut commands, &asset_server, &level_config, &game_run);
    spawn_stage(&mut commands, &asset_server);
    commands.spawn(player_bundle(&asset_server, level_config.start_position));
    commands.spawn_batch(level_config.enemies);
}

fn check_load_status(mut next_state: ResMut<NextState<LevelState>>) {
    next_state.set(LevelState::Ready);
}

fn on_enemy_destroyed(
    _trigger: Trigger<EnemyDestroyedEvent>,
    mut level_stats: Single<&mut LevelStats>,
) {
    level_stats.enemies_destroyed += 1;
}

fn check_level_complete(
    mut next_state: ResMut<NextState<LevelState>>,
    mut level_stats: Single<&mut LevelStats>,
    enemies_q: Query<&EnemyDestruction, With<Enemy>>,
    explosions_q: Query<(), With<Explosion>>,
) {
    if enemies_q
        .iter()
        .filter(|destruction| destruction == &&EnemyDestruction::Required)
        .collect::<Vec<_>>()
        .is_empty()
        && explosions_q.is_empty()
    {
        level_stats.success = Some(true);
        next_state.set(LevelState::Complete);
    }
}

fn on_player_destroyed(
    _trigger: Trigger<PlayerDestroyedEvent>,
    mut next_state: ResMut<NextState<LevelState>>,
    mut level_stats: Single<&mut LevelStats>,
) {
    level_stats.success = Some(false);
    next_state.set(LevelState::Complete);
}

#[derive(Resource, Debug, Default, Deref, DerefMut)]
struct LeadOutTimer(Timer);

fn on_level_complete(
    mut lead_out_timer: ResMut<LeadOutTimer>,
    level_stats: Single<&LevelStats>,
    mut game_run: Single<&mut GameRun>,
) {
    if level_stats.success == Some(true) {
        game_run.set_current_level_status(LevelStatus::Completed);
        lead_out_timer.set_duration(LEAD_OUT_TIME_SUCCESS);
    } else {
        game_run.set_current_level_status(LevelStatus::Tried);
        lead_out_timer.set_duration(LEAD_OUT_TIME_FAIL);
    }
    lead_out_timer.reset();
}

fn update_lead_out_timer(
    commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut lead_out_timer: ResMut<LeadOutTimer>,
    level_stats: Single<&LevelStats>,
    game_run: Single<&mut GameRun>,
) {
    if !lead_out_timer.finished() {
        lead_out_timer.tick(time.delta());
        if lead_out_timer.just_finished() {
            spawn_level_complete_menu(commands, &asset_server, &level_stats, &game_run);
        }
    }
}
