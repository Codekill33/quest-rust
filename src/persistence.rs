use crate::errors::AppError;
use crate::player::Player;

#[cfg(not(feature = "wasm"))]
pub fn save_player(player: &Player, path: &str) -> Result<(), AppError> {
    use std::fs;
    let json = player.to_json()?;
    fs::write(path, json).map_err(AppError::Io)?;
    Ok(())
}

#[cfg(not(feature = "wasm"))]
pub fn load_player(path: &str) -> Result<Option<Player>, AppError> {
    use std::fs;
    use std::path::Path;
    if !Path::new(path).exists() {
        return Ok(None);
    }
    let json = fs::read_to_string(path).map_err(AppError::Io)?;
    let player = Player::from_json(&json)?;
    Ok(Some(player))
}

#[cfg(feature = "wasm")]
pub fn save_player(player: &Player, path: &str) -> Result<(), AppError> {
    let window = web_sys::window().expect("no global window exists");
    let local_storage = window
        .local_storage()
        .expect("should have local storage")
        .expect("local storage is missing");
    let json = player.to_json()?;
    local_storage
        .set_item(path, &json)
        .map_err(|_| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, "localStorage error")))?;
    Ok(())
}

#[cfg(feature = "wasm")]
pub fn load_player(path: &str) -> Result<Option<Player>, AppError> {
    let window = web_sys::window().expect("no global window exists");
    let local_storage = window
        .local_storage()
        .expect("should have local storage")
        .expect("local storage is missing");
    
    if let Ok(Some(json)) = local_storage.get_item(path) {
        let player = Player::from_json(&json)?;
        Ok(Some(player))
    } else {
        Ok(None)
    }
}
