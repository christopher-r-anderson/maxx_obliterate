use bevy::prelude::*;

use crate::gameplay::{
    enemy::{DefenderClass, ENEMY_DEFENDER_SIZE, EnemyBundle},
    level::LevelConfig,
    stage::{STAGE_HEIGHT, STAGE_WIDTH},
};

pub fn get_config(asset_server: &AssetServer) -> LevelConfig {
    let hh = 260.;
    let hw = 30.;
    let wall_height = STAGE_HEIGHT / 2. - hh;

    let mut enemies = vec![
        EnemyBundle::new_base(asset_server, vec2(0., 330.)),
        EnemyBundle::new_shadow(asset_server, vec2(0., 100.)),
        EnemyBundle::new_wall(
            asset_server,
            vec2(0., -hh - wall_height / 2.),
            vec2(STAGE_WIDTH, wall_height),
        ),
        EnemyBundle::new_land(
            asset_server,
            vec2(0., -hh + 30. * 2. / 2.),
            vec2(STAGE_WIDTH, 30. * 2.),
        ),
    ];

    let counts = [
        (&DefenderClass::Two, 8),
        (&DefenderClass::Three, 8),
        (&DefenderClass::One, 2),
    ];
    let x = hw + ENEMY_DEFENDER_SIZE.x / 2.;
    let mut y = hh;
    for (class, count) in counts {
        for _ in 0..count {
            enemies.push(EnemyBundle::new_defender(asset_server, vec2(-x, y), class));
            enemies.push(EnemyBundle::new_defender(asset_server, vec2(x, y), class));
            y -= 30.;
        }
    }

    LevelConfig {
        name: "Corridor Shooter 2",
        notes: "",
        start_position: vec2(0., -hh + 20.),
        enemies,
    }
}
