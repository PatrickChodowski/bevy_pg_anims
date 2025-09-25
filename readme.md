# Animations management plugin for Bevy

(PG stands for PatchGames)

Control the animations of entity using one `Anim` component

Feel free to use the code it as a template, I dont plan to extend it massively unless I need it in my own project.


```

#[allow(dead_code)]
#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum ANIM {
    ROOT = 0,       // NECESSARY to match ANIM id with index
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
pub const MASK_LOWER_BODY: u32 = 1;
pub const MASK_UPPER_BODY: u32 = 2;


fn map_targets_to_masks() -> Vec<(String, Vec<u32>)> {
    let map: Vec<(String, Vec<u32>)> = vec![

        ("Armature/mixamorig:Hips/mixamorig:Spine".to_string(),
        vec![MASK_LOWER_BODY, MASK_ALL]),

        ("Armature/mixamorig:Hips/mixamorig:Spine/mixamorig:Spine1".to_string(),
        vec![MASK_UPPER_BODY, MASK_ALL]),
 
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
        .add_systems(RunOnce, init)
        .add_systems(PostUpdate, track_hands.in_set(PGAnimsSet::Anims))
        .add_observer(anim_start)    
        .add_observer(anim_end)
        ;
    }
}

fn init(){
    // Obtain GLTF handle...
    ... when ready:
    commands.spawn(AnimGraphInit{gltf_handle: animated_gltf_handle.clone()});
}

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


```



## Attach event later to clip


``` 
#[derive(Resource)]
struct RunOnce(bool);

impl Default for RunOnce {
    fn default() -> Self {
        RunOnce(false)
    }
}



fn attach_events_to_clips(
    player_graph:       Option<Res<PGAnimGraph>>,
    mut graphs:         ResMut<Assets<AnimationGraph>>,
    mut clips:          ResMut<Assets<AnimationClip>>,
    mut run_once:       Local<RunOnce>
){
    if run_once.0 {
        return;
    }

    if let Some(player_graph) = player_graph {
        if let Some(graph) = graphs.get_mut(&player_graph.graph){
            let anim_clip = get_clip(player_graph.animations[ANIM::Walking.get()], graph, &mut clips);
            info!(" [ANIMS] Attaching WalkEvent Event to AnimWalking duration: {}", anim_clip.duration());
            anim_clip.add_event(0.02, StepEvent);
            anim_clip.add_event(0.4, StepEvent);
            run_once.0 = true;
        }
    }
}

```