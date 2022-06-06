mod cbc_game;

fn main() -> anyhow::Result<()> {
    let mut cbc_interactive = cbc_game::CBCGame::new();
    cbc_interactive.start()?;

    Ok(())
}
