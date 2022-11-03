pub mod intro;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Intro,
    Title,
    Game,
}
