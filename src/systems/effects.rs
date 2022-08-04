use crate::prelude::*;

#[system]
#[read_component(Player)]
#[read_component(Point)]
#[write_component(Pools)]
#[write_component(Attributes)]
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
#[write_component(Consumable)]
#[read_component(SingleActivation)]
#[read_component(MagicItem)]
#[read_component(SpawnParticleLine)]
#[read_component(SpawnParticleBurst)]
#[read_component(ProvidesRemoveCurse)]
#[read_component(ProvidesIdentify)]
#[read_component(Duration)]
#[read_component(StatusEffect)]
#[read_component(AttributeBonus)]
#[read_component(SpellTemplate)]
#[read_component(ProvidesMana)]
#[read_component(TeachSpell)]
#[write_component(KnownSpells)]
#[read_component(Slow)]
#[read_component(DamageOverTime)]
#[read_component(TileSize)]
#[write_component(Skills)]
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
