pub mod intro;
pub mod title;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Intro,
    Title,
    Game,
}
