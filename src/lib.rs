use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::animation::{animate_targets, AnimationTargetId};


pub struct PGAnimsPlugin {
    pub anims_with_start_event: Vec<usize>,
    pub anims_with_end_event: Vec<usize>,
    pub targets_masks_mapping: Vec<(String, Vec<u32>)>
}

impl Default for PGAnimsPlugin{
    fn default() -> Self {
        PGAnimsPlugin {
            anims_with_start_event: Vec::new(), 
            anims_with_end_event: Vec::new(), 
            targets_masks_mapping: Vec::new()
        }
    }
}


#[derive(Resource)]
struct AnimsPluginConfig{
    anims_with_start_event: Vec<usize>,
    anims_with_end_event:   Vec<usize>,
    targets_masks_mapping:  Vec<(String, Vec<u32>)>
}


impl Plugin for PGAnimsPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<PlayerGraph>()
        .register_type::<Anim>()
        .register_type::<AnimsConf>()
        .register_type::<PGAnimatable>()
        .insert_resource(AnimsPluginConfig {
            anims_with_start_event: self.anims_with_start_event.clone(),
            anims_with_end_event:   self.anims_with_end_event.clone(),
            targets_masks_mapping:  self.targets_masks_mapping.clone()
        })
        .add_event::<AnimStartEvent>()
        .add_event::<AnimEndEvent>()
        .add_systems(Update, (
            (
                attach_animation_graphs, 
                update_animatable
            ).before(animate_targets),
            update_animation
        ).in_set(PGAnimsSet::Anims))
        .add_systems(PostUpdate, play_next_animation_after_finished)
        // .add_observer(anim_start)    
        // .add_observer(anim_end)
        ;
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PGAnimsSet {
    Anims
}


fn attach_animation_graphs(
    mut commands:       Commands,
    player_graph:       Res<PlayerGraph>,
    mut graphs:         ResMut<Assets<AnimationGraph>>,
    mut clips:          ResMut<Assets<AnimationClip>>,
    armatures:          Query<Entity, (Added<AnimationPlayer>, Without<PGAnimatable>)>,
    mut added_events:   Local<AnimsPostprocessDone>,
    anims_config:       Res<AnimsPluginConfig>
){

    for armature_entity in armatures.iter() {

        if added_events.0 == false {
            let graph = graphs.get_mut(&player_graph.graph).unwrap();

            // Masks
            for (names_string, masks) in anims_config.targets_masks_mapping.iter(){
                let names: Vec<Name> = names_string.split('/').map(|s| Name::new(s.to_string())).collect();
                let target_id = AnimationTargetId::from_names(names.iter());
                for mask in masks.iter(){
                    graph.add_target_to_mask_group(target_id, *mask);
                }
            }

            // Events
            for anim in anims_config.anims_with_start_event.iter(){
                let anim_clip = get_clip(player_graph.animations[*anim], graph, &mut clips);
                anim_clip.add_event(anim_clip.duration(), AnimStartEvent{anim: *anim});
            }

            for anim in anims_config.anims_with_end_event.iter(){
                let anim_clip = get_clip(player_graph.animations[*anim], graph, &mut clips);
                anim_clip.add_event(anim_clip.duration(), AnimEndEvent{anim: *anim});
            }

            added_events.0 = true;
        }

        commands.entity(armature_entity).insert(AnimationGraphHandle(player_graph.graph.clone()));
    }
}


fn update_animatable(
    mut commands:   Commands,
    armatures:      Query<Entity, Added<AnimationGraphHandle>>,
    parents:        Query<&ChildOf>,
    animatables:    Query<&PGAnimatable>
){

    for entity in armatures.iter(){
        let mut animatable_entity = Entity::PLACEHOLDER;
        let mut default_anim: usize = 0;
        for parent_entity in parents.iter_ancestors(entity){
            if let Ok(animatable) = animatables.get(parent_entity){
                animatable_entity = parent_entity;
                default_anim = animatable.default_anim;
                break;
            }
        }
        commands.entity(animatable_entity).insert(
            (
                PGAnimatable{armature: entity, default_anim},
                Anim::new(default_anim)
            )
        );
    }

}

fn get_clip<'a>(
    node: AnimationNodeIndex,
    graph: &AnimationGraph,
    clips: &'a mut Assets<AnimationClip>,
) -> &'a mut AnimationClip {
    let node = graph.get(node).unwrap();
    let clip = match &node.node_type {
        AnimationNodeType::Clip(handle) => clips.get_mut(handle),
        _ => unreachable!(),
    };
    clip.unwrap()
}

// The only way it works
struct AnimsPostprocessDone(bool);

impl Default for AnimsPostprocessDone {
    fn default() -> Self {
        AnimsPostprocessDone(false)
    }
}

fn update_animation(
    player_graph:     Res<PlayerGraph>,
    mut graphs:       ResMut<Assets<AnimationGraph>>,
    mut animatables:  Query<(&PGAnimatable, &mut Anim), Changed<Anim>>,
    mut players:      Query<(Entity, &mut AnimationPlayer)>
){
    for (animatable, anim) in animatables.iter_mut(){
        let Ok((_anim_player_entity, mut player)) = players.get_mut(animatable.armature) else {continue;};
        let Some(graph) = graphs.get_mut(&player_graph.graph) else {continue;};

        let current_anims = player.playing_animations().map(|(id, _aa)| id.index()).collect::<HashSet<_>>();
        let new_anims = anim.get();

        // No need to change anything
        if current_anims == new_anims {
            continue;
        }

        // Otherwise
        player.stop_all();

        for animconf in anim.anims.iter(){
            let node_id = player_graph.animations[animconf.index];
            let active_animation = player.play(node_id);

            if let Some(speed) = animconf.speed {
                active_animation.set_speed(speed);
            } else {
                active_animation.set_speed(1.0);
            }

            if anim.repeat {
                active_animation.repeat();
            };

            if let Some(mask) = animconf.mask {
                if let Some(animation_node) = graph.get_mut(node_id){
                    animation_node.mask = 0;
                    animation_node.add_mask_group(mask);
                }
            } else {
                if let Some(animation_node) = graph.get_mut(node_id){
                    animation_node.mask = 0;
                }
            }
        }
    }
}

fn play_next_animation_after_finished(
    changed_players:      Query<(Entity, &AnimationPlayer), Changed<AnimationPlayer>>,
    mut animatables:      Query<(&PGAnimatable, &mut Anim)>
){
    for (entity, player) in changed_players.iter(){
        if player.all_finished(){
            for (animatable, mut anim) in animatables.iter_mut(){
                if animatable.armature == entity {
                    if let Some(ref mut next) = anim.next{
                        if let Some(next_anims) = next.pop(){
                            anim.anims = next_anims;
                        } else {
                            anim.next = None;
                        }
                    } else {
                        anim.stop_all();
                    }
                    break; // Break from looking for animatables
                }
            }
        }
    }
}



/* 
// Example observer implementations
fn anim_start(
    trigger:        Trigger<AnimStartEvent>,
    mut commands:   Commands,
    parents:        Query<&ChildOf>
){

    for ancestor in parents.iter_ancestors(trigger.target()){
        match trigger.anim {
            _ => {}
        }
    }
}


fn anim_end(
    trigger:        Trigger<AnimEndEvent>,
    mut commands:   Commands,
    parents:        Query<&ChildOf>
){
    for ancestor in parents.iter_ancestors(trigger.target()){
        // if let Ok(mut state) = combats.get_mut(ancestor){
        //     match trigger.anim {
        //         _ => {}
        //     }
        // }
    }
}
*/

#[derive(Resource, Reflect)]
pub struct PlayerGraph {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph:      Handle<AnimationGraph>,
}

#[derive(Event, Clone)]
pub struct AnimStartEvent {
    pub anim: usize
}

#[derive(Event, Clone)]
pub struct AnimEndEvent {
    pub anim: usize
}


#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct PGAnimatable {
    pub armature: Entity,
    pub default_anim: usize
}
impl PGAnimatable {
    pub fn new(default_anim: usize) -> Self {
        PGAnimatable{
            armature: Entity::PLACEHOLDER, 
            default_anim
        }
    }
}


#[derive(Reflect, Debug)]
pub struct AnimsConf {
    index: usize,      // ID of animation
    mask:  Option<u32>, // Optional Mask
    speed: Option<f32>
}
impl Default for AnimsConf {
    fn default() -> Self {
        AnimsConf{
            index: 0, 
            mask: None, 
            speed: None
        }
    }
}

impl AnimsConf {
    pub fn new(index: usize) -> Self {
        AnimsConf{index, ..default()}
    }
    pub fn new_with_mask(index: usize, mask: u32) -> Self {
        return AnimsConf{index, mask: Some(mask), ..default()};
    }
}

pub type Anims = Vec<AnimsConf>;


#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Anim {
    anims:   Anims,
    next:    Option<Vec<Anims>>,
    repeat:  bool
}

impl Anim {
    pub fn new(
        anim_id: usize
    ) -> Self {
        return Anim{
            anims: vec![
                AnimsConf::new(anim_id)
            ],
            repeat: true, 
            next: None
        };
    }

    pub fn get(&self) -> HashSet<usize> {
        return self.anims.iter().map(|aconf| aconf.index).collect::<HashSet<usize>>();
    }

    pub fn set(&mut self, anims: Anims, repeat: bool){
        self.anims = anims;
        self.repeat = repeat;
    }

    pub fn set_loop(&mut self, anim_id: usize){
        self.anims = vec![
            AnimsConf::new(anim_id)
        ];
        self.repeat = true;
    }

    pub fn set_once(&mut self, anim_id: usize){
        self.anims = vec![
            AnimsConf::new(anim_id)
        ];
        self.repeat = false;
    }

    pub fn set_next(&mut self, next: Vec<Anims>){
        self.next = Some(next);
    }

    pub fn stop_all(&mut self){
        self.anims.clear();
    }
    
    // pub fn set_all_speed(&mut self, speed: f32){
    //     for anim in self.anims.iter_mut(){
    //         anim.speed = Some(speed);
    //     }
    // }

    // pub fn set_mask(&mut self, mask: u32){
    //     for anim in self.anims.iter_mut(){
    //         anim.mask = Some(mask);
    //     }
    // }
}

impl Default for Anim {
    fn default() -> Self {
        Anim{
            anims: Vec::new(),
            repeat: true, 
            next: None
        }
    }
}


pub mod prelude {
    pub use crate::{
        PGAnimsPlugin, 
        PGAnimsSet,
        AnimStartEvent, 
        AnimEndEvent,
        PGAnimatable,
        AnimsConf,
        Anims,
        Anim
    };
}

/*
// Example ANIM and Masks:

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

*/