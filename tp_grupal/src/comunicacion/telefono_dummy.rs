use crate::comunicacion::telefono::Telefono;

#[derive(Default)]
pub struct TelefonoDummy {}

impl Telefono for TelefonoDummy {
    fn atender_llamada(&mut self) -> Result<(), super::telefono::ErrorTelefono> {
        Ok(())
    }

    fn llamar(&mut self, _usuario: &str) -> Result<(), super::telefono::ErrorTelefono> {
        Ok(())
    }

    fn rechazar_llamada(&mut self) -> Result<(), super::telefono::ErrorTelefono> {
        Ok(())
    }
}
