use crate::app::{AppError, json_output_error, load_config_from_current_dir};
use crate::cli::prime_output;
use crate::output::output_message;

pub(crate) fn prime_command(json: bool) -> Result<Option<String>, AppError> {
    let Some((_, config)) = load_config_from_current_dir()? else {
        return Ok(None);
    };

    let output = prime_output(&config);
    if json {
        Ok(Some(output_message(output).map_err(json_output_error)?))
    } else {
        Ok(Some(output))
    }
}
