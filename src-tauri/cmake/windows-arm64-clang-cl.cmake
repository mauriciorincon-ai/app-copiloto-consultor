# whisper.cpp en Windows ARM64 (sprint 009, ADR 022 enmienda 3).
#
# ggml no compila con MSVC en ARM («MSVC is not supported for ARM, use clang», ggml/src/ggml-cpu/CMakeLists.txt), así
# que en este target —y solo en este— la parte C/C++ de `whisper-rs-sys` se compila con clang-cl (LLVM) y Ninja. Lo
# activa `src-tauri/.cargo/config.toml` con variables que llevan el target en el nombre: x64 y macOS no lo ven.
# Nada se descarga: clang-cl es el de LLVM (o el que traen las Build Tools) y Ninja, el de las Build Tools.

set(_pf "ProgramFiles(x86)")
execute_process(
  COMMAND "$ENV{${_pf}}/Microsoft Visual Studio/Installer/vswhere.exe" -latest -products * -property installationPath
  OUTPUT_VARIABLE _vs
  OUTPUT_STRIP_TRAILING_WHITESPACE)

find_program(AG_CLANG_CL clang-cl HINTS "$ENV{ProgramFiles}/LLVM/bin" "${_vs}/VC/Tools/Llvm/ARM64/bin" REQUIRED)
set(CMAKE_C_COMPILER "${AG_CLANG_CL}")
set(CMAKE_CXX_COMPILER "${AG_CLANG_CL}")

# El Ninja de las Build Tools (componente «C++ CMake tools for Windows»): no hace falta instalar otro.
if(NOT CMAKE_MAKE_PROGRAM)
  find_program(CMAKE_MAKE_PROGRAM ninja HINTS "${_vs}/Common7/IDE/CommonExtensions/Microsoft/CMake/Ninja" REQUIRED)
endif()
