use comet::prelude::*;

trait Stat: Component + std::fmt::Debug {
    fn name(&self) -> String;
    fn value(&self) -> u8;
}

#[derive(Component, Debug)]
#[require(Stats = Stats::player)]
struct Player;

#[derive(Component, Debug)]
#[query_as(Stat)]
struct Health(u8);

impl Stat for Health {
    fn name(&self) -> String {
        "health".into()
    }

    fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Component, Debug)]
#[query_as(Stat)]
struct Attack(u8);

impl Stat for Attack {
    fn name(&self) -> String {
        "attack".into()
    }

    fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Component, Debug)]
#[query_as(Stat)]
struct Defense(u8);

impl Stat for Defense {
    fn name(&self) -> String {
        "defense".into()
    }

    fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Component, Debug)]
#[query_as(Stat)]
struct Luck(u8);

impl Stat for Luck {
    fn name(&self) -> String {
        "luck".into()
    }

    fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Component, Debug)]
#[query_as(Stat)]
struct Arcane(u8);

impl Stat for Arcane {
    fn name(&self) -> String {
        "arcane".into()
    }

    fn value(&self) -> u8 {
        self.0
    }
}

bundle!(Stats {
    health: Health,
    attack: Attack,
    defense: Defense,
    luck: Luck,
    arcane: Arcane
});

impl Stats {
    pub fn player() -> Self {
        Self {
            health: Health(200),
            attack: Attack(100),
            defense: Defense(50),
            luck: Luck(10),
            arcane: Arcane(0),
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            health: Health(0),
            attack: Attack(0),
            defense: Defense(0),
            luck: Luck(0),
            arcane: Arcane(0),
        }
    }
}

bundle!(Character {
    player: Player,
    transform: Transform,
    stats: Stats,
});

impl Character {
    pub fn new() -> Self {
        Self {
            player: Player,
            transform: Transform::at(v3::new(1.0, 1.0, 1.0)),
            stats: Stats::player(),
        }
    }
}

fn setup(app: &mut App) {
    app.spawn(Character::new());
    app.spawn((Transform::at(v3::new(-1.0, 0.0, 0.0)), Stats::default()));

    app.query::<(Entity, Amount<&dyn Stat>, &dyn Stat), AtLeast<dyn Stat, 3>>()
        .for_each(|(entity, amount, stat)| {
            info!(
                "{entity:?} has {amount} stats, including {} ({})",
                stat.name(),
                stat.value()
            );
        });

    app.query::<(Entity, First<&dyn Stat>, Last<&dyn Stat>), With<Player>>()
        .for_each(|(entity, first, last)| {
            info!(
                "{entity:?} starts with {} and ends with {}",
                first.name(),
                last.name()
            );
        });
}

fn update(app: &mut App, _dt: f32) {
    app.query::<&mut Attack, With<Player>>()
        .for_each(|attack| attack.0 = attack.0.saturating_add(1));

    app.query::<(Entity, Amount<&dyn Stat>, Take<&dyn Stat, 3>), ChangedAtLeast<dyn Stat, 1>>()
        .for_each(|(entity, amount, stat)| {
            info!(
                "{entity:?} changed {amount} available stats; {} is now {}",
                stat.name(),
                stat.value()
            );
        });

    app.quit();
}

fn main() {
    App::with_preset(Headless).run(setup, update)
}
