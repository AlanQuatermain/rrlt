use crate::prelude::*;

#[system]
#[read_component(Player)]
#[read_component(Point)]
#[write_component(Pools)]
#[read_component(Attributes)]
#[read_component(FieldOfView)]
#[read_component(Item)]
#[read_component(Name)]
#[write_component(HungerClock)]
#[read_component(ProvidesFood)]
#[read_component(ProvidesDungeonMap)]
#[read_component(TownPortal)]
#[read_component(Damage)]
#[read_component(AreaOfEffect)]
#[read_component(ProvidesHealing)]
#[write_component(Confusion)]
#[read_component(TeleportTo)]
#[read_component(Hidden)]
#[read_component(Carried)]
#[read_component(Equipped)]
#[read_component(Consumable)]
#[read_component(SingleActivation)]
#[read_component(MagicItem)]
#[read_component(SpawnParticleLine)]
#[read_component(SpawnParticleBurst)]
pub fn effects(
    ecs: &mut SubWorld,
    #[resource] map: &mut Map,
    #[resource] particle_builder: &mut ParticleBuilder,
    #[resource] gamelog: &mut Gamelog,
    #[resource] turn_state: &mut TurnState,
    #[resource] dm: &mut MasterDungeonMap,
    commands: &mut CommandBuffer,
) {
    run_effects_queue(
        ecs,
        map,
        particle_builder,
        gamelog,
        turn_state,
        dm,
        commands,
    );
}
