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

# `AG_LLVM_BIN` (opcional) manda sobre lo demás: la CI lo pone si tuvo que sacar un LLVM ARM64 aparte.
find_program(AG_CLANG_CL clang-cl
  HINTS "$ENV{AG_LLVM_BIN}" "$ENV{ProgramFiles}/LLVM/bin" "${_vs}/VC/Tools/Llvm/ARM64/bin" REQUIRED)
set(CMAKE_C_COMPILER "${AG_CLANG_CL}")
set(CMAKE_CXX_COMPILER "${AG_CLANG_CL}")
# Aunque el clang-cl que aparezca sea el x64 (emulado), el objeto sale ARM64.
set(CMAKE_C_COMPILER_TARGET aarch64-pc-windows-msvc)
set(CMAKE_CXX_COMPILER_TARGET aarch64-pc-windows-msvc)

# El Ninja de las Build Tools (componente «C++ CMake tools for Windows»), o el que haya en el PATH.
if(NOT CMAKE_MAKE_PROGRAM)
  find_program(CMAKE_MAKE_PROGRAM ninja HINTS "${_vs}/Common7/IDE/CommonExtensions/Microsoft/CMake/Ninja" REQUIRED)
endif()

# **Un binario que corre en todos los equipos ARM64 de Windows que lo pueden correr, no solo en el que lo compiló.**
# Con el valor por defecto (`GGML_NATIVE=ON`) ggml compila con `-mcpu=native` y PRUEBA las instrucciones en la máquina
# que compila: el instalador que sube la CI saldría afinado para la CPU del runner y no tendría por qué arrancar en la
# Surface. Se fija la base `armv8.2-a` con producto escalar y aritmética vectorial de media precisión (lo que usan los
# modelos de whisper, que van en f16); `i8mm` y SVE solo ayudan a los modelos cuantizados, que la app no usa.
set(GGML_NATIVE OFF CACHE BOOL "binario portable, sin -mcpu=native" FORCE)
set(GGML_CPU_ARM_ARCH "armv8.2-a+dotprod+fp16" CACHE STRING "base ARM64 portable de Windows" FORCE)
