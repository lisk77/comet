use comet::prelude::*;

trait Stat: Component {
    fn value(&self) -> u8;
}

#[derive(Component, Debug)]
#[query_as(Stat)]
struct Health(u8);

impl Stat for Health {
    fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Component, Debug)]
#[query_as(Stat)]
struct Attack(u8);

impl Stat for Attack {
    fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Component, Debug)]
#[query_as(Stat)]
struct Defense(u8);

impl Stat for Defense {
    fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Component)]
struct Player;

fn setup(app: &mut App) {
    app.spawn((Player, Health(250), Attack(100), Defense(50)));

    app.spawn((Player, Health(1), Attack(2), Defense(3)));
}

fn update(app: &mut App, _dt: f32) {
    app.query::<&dyn Stat, With<Player>>()
        .for_each(|stat| info!("Stat: {}", stat.value()));
    app.quit();
}

fn main() {
    App::with_preset(Headless).run(setup, update);
}
