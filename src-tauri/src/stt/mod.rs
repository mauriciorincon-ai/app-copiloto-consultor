//! Transcripción local — **MÓDULO PROTEGIDO** (regla del efímero verificable).
//!
//! Mismas reglas que `capture/`: RAM y nada más. El audio no sale del equipo para volverse
//! texto (regla de nada crudo fuera), y el texto que produce vive en una ventana deslizante de
//! turnos que muere al cerrar la sesión.
//!
//! Se llena en la fase 3 del sprint 001 (SpeechAnalyzer con respaldo whisper-rs; el motor se
//! elige por ADR con medición, no por preferencia).
