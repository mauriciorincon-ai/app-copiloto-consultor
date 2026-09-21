//! Corpus del consultor — módulo **NO protegido**, y con razón.
//!
//! Aquí SÍ se abre disco: el corpus son los documentos PROPIOS del usuario, que se leen donde
//! están, y su índice vive en la carpeta de la app. Eso es exactamente lo que la regla del
//! efímero permite persistir. Por eso `corpus/` queda fuera de la lista de módulos protegidos
//! de `verify:ephemeral`: si estuviera dentro, el gate se volvería imposible de cumplir y la
//! tentación sería aflojarlo — y un gate aflojado deja de proteger lo que sí importa.
//!
//! La frontera es la del estándar 4-T: **lo del usuario** puede persistir; **lo de terceros**,
//! no. Nada de lo que entra aquí viene de la reunión.
//!
//! Se llena en la fase 4 del sprint 001 (ingesta → chunking por sección → BM25 con tantivy).
