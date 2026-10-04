//! EL ENSAYO (C18, sprint 004, ADR 019) — practicar la reunión antes de tenerla.
//!
//! Eliges el cliente y tu propuesta; la app te hace las preguntas que ese cliente probablemente hará,
//! te escucha responder y te dice qué evidencia de tu corpus usaste. **Código primero**: el banco sale
//! de reglas publicadas (`data/ensayo/`) y el modelo solo suma, si lo enciendes, un puñado de preguntas
//! fundadas en tus secciones.
//!
//! **MÓDULO PROTEGIDO** (`verify:ephemeral`): aquí se arma lo que se pregunta y, desde la fase 3, se
//! oye tu respuesta. Ni disco ni red: los documentos los lee `lib.rs` (con `corpus`) y se los pasa ya
//! troceados; lo que se guarda de un ensayo lo escribe otro módulo, con la llave de tus notas.

pub mod banco;
pub mod enriquecer;
