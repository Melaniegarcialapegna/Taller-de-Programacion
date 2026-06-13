//! Comunicacion - Objetos que ejecutan operaciones usando al servidor.
pub mod comunicador;
#[cfg(test)]
pub mod comunicador_fake;
#[cfg(test)]
pub mod comunicador_fake_y_mock;
#[cfg(test)]
pub mod comunicador_mock;
#[cfg(test)]
pub mod comunicador_stub;
pub mod comunicador_tcp;
pub mod lobby;
pub mod telefono;
#[cfg(test)]
pub mod telefono_dummy;
#[cfg(test)]
pub mod telefono_mock;
#[cfg(test)]
pub mod telefono_stub;
