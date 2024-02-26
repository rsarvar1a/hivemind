use hivemind::prelude::*;

pub fn setup() -> Result<()>
{
    env_logger::try_init().map_err(|_| Error::new(Kind::InternalError, "Could not initialize logger.".into()))
}
