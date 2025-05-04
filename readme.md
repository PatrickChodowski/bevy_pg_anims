# Animations management plugin for Bevy

(PG stands for PatchGames)

Control the animations of entity using one `Anim` component

Feel free to use the code it as a template, I dont plan to extend it massively unless I need it in my own project.


```

#[allow(dead_code)]
#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum ANIM {
    ROOT = 0,
    BreathingIdle = 1,
    Dribble = 2,
    JogForward = 3,
    Running = 4
}
impl ANIM {
    pub fn get(self) -> usize {
        self as usize
    }
    pub fn from_usize(value: usize) -> Self {
        match value {
            0 => ANIM::ROOT,
            1 => ANIM::BreathingIdle,
            2 => ANIM::Dribble,
            3 => ANIM::JogForward,
            4 => ANIM::Running,
            _ => panic!("Invalid enum value: {}", value),
        }
    }
}

pub const MASK_ALL: u32 = 0;
pub const MASK_LEFT_HAND: u32 = 1;
pub const MASK_RIGHT_HAND: u32 = 2;
pub const MASK_LEFT_LEG: u32 = 3;
pub const MASK_RIGHT_LEG: u32 = 4;
pub const MASK_LOWER_BODY: u32 = 5;
pub const MASK_UPPER_BODY: u32 = 6;
pub const MASK_HANDS: u32 = 7;
pub const MASK_LEGS: u32 = 8;


fn map_targets_to_masks() -> Vec<(String, Vec<u32>)> {
    let map: Vec<(String, Vec<u32>)> = vec![

        ("Armature/mixamorig:Hips/mixamorig:Spine".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1".to_string(),
        vec![MASK_UPPER_BODY, MASK_ALL]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2".to_string(),
        vec![MASK_UPPER_BODY, MASK_ALL]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2/mixamorig:Neck".to_string(),
        vec![MASK_UPPER_BODY, MASK_ALL]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2/mixamorig:Neck/mixamorig:Head".to_string(),
        vec![MASK_UPPER_BODY, MASK_ALL]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2/mixamorig:LeftShoulder".to_string(), 
        vec![MASK_UPPER_BODY, MASK_ALL, MASK_LEFT_HAND, MASK_HANDS]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2/mixamorig:LeftShoulder/mixamorig:LeftArm".to_string(), 
        vec![MASK_UPPER_BODY, MASK_ALL, MASK_LEFT_HAND, MASK_HANDS]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2/mixamorig:LeftShoulder/mixamorig:LeftArm/mixamorig:LeftForeArm".to_string(), 
        vec![MASK_UPPER_BODY, MASK_ALL, MASK_LEFT_HAND, MASK_HANDS]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2/mixamorig:RightShoulder".to_string(), 
        vec![MASK_UPPER_BODY, MASK_ALL, MASK_RIGHT_HAND, MASK_HANDS]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2/mixamorig:RightShoulder/mixamorig:RightArm".to_string(), 
        vec![MASK_UPPER_BODY, MASK_ALL, MASK_RIGHT_HAND, MASK_HANDS]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1/mixamorig:Spine2/mixamorig:RightShoulder/mixamorig:RightArm/mixamorig:RightForeArm".to_string(), 
        vec![MASK_UPPER_BODY, MASK_ALL, MASK_RIGHT_HAND, MASK_HANDS]),

        ("Armature/mixamorig:Hips".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL]),

        ("Armature/mixamorig:Hips/mixamorig:LeftUpLeg".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL, MASK_LEFT_LEG, MASK_LEGS]),

        ("Armature/mixamorig:Hips/mixamorig:LeftUpLeg/mixamorig:LeftLeg".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL, MASK_LEFT_LEG, MASK_LEGS]),

        ("Armature/mixamorig:Hips/mixamorig:LeftUpLeg/mixamorig:LeftLeg/mixamorig:LeftFoot".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL, MASK_LEFT_LEG, MASK_LEGS]),

        ("Armature/mixamorig:Hips/mixamorig:LeftUpLeg/mixamorig:LeftLeg/mixamorig:LeftFoot/mixamorig:LeftToeBase".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL, MASK_LEFT_LEG, MASK_LEGS]),

        ("Armature/mixamorig:Hips/mixamorig:RightUpLeg".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL, MASK_RIGHT_LEG, MASK_LEGS]),

        ("Armature/mixamorig:Hips/mixamorig:RightUpLeg/mixamorig:RightLeg".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL, MASK_RIGHT_LEG, MASK_LEGS]),

        ("Armature/mixamorig:Hips/mixamorig:RightUpLeg/mixamorig:RightLeg/mixamorig:RightFoot".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL, MASK_RIGHT_LEG, MASK_LEGS]),

        ("Armature/mixamorig:Hips/mixamorig:RightUpLeg/mixamorig:RightLeg/mixamorig:RightFoot/mixamorig:RightToeBase".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL, MASK_RIGHT_LEG, MASK_LEGS]),
 
    ];

    return map;
}

use bevy_pg_anims::prelude::*;

pub struct AnimsPlugin;

impl Plugin for AnimsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(PGAnimsPlugin{
            targets_masks_mapping: map_targets_to_masks(),
            ..default()
        })
        .add_systems(PostUpdate, track_hands.in_set(PGAnimsSet::Anims))
        ;
    }
}

```